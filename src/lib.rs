mod api;
mod cache;

use std::{collections::HashMap, error::Error};

use api::mastodon::{Api as MastodonApi, Status as MastodonStatus};
use api::twitter::Api as TwitterApi;
use worker::{console_error, console_log, event, Env, ScheduleContext, ScheduledEvent};

#[event(scheduled)]
pub async fn scheduled(_event: ScheduledEvent, env: Env, ctx: ScheduleContext) {
  console_log!("Processing start.");

  let result = sync_posts(&env, &ctx).await;
  if result.is_err() {
    console_error!("Error occurred: {error:#}", error = result.unwrap_err());
  }

  console_log!("Processing finished.");
}

async fn sync_posts(
  env: &Env,
  ctx: &ScheduleContext,
) -> Result<HashMap<String, String>, Box<dyn Error>> {
  let mastodon_api = MastodonApi::new(env)?;

  // Get target mastodon account.
  let mastodon_account_acct = env.secret("MASTODON_ACCOUNT_ACCT")?.to_string();
  let mastodon_account_cache = cache::get_target_account(env).await;
  let mastodon_account = match &mastodon_account_cache {
    Ok(account) => match account {
      Some(account) => account.to_owned(),
      None => mastodon_api.lookup_account(&mastodon_account_acct).await?,
    },
    Err(_) => mastodon_api.lookup_account(&mastodon_account_acct).await?,
  };

  console_log!(
    "Target mastodon account: {acct} ({id})",
    acct = &mastodon_account_acct,
    id = &mastodon_account.id
  );

  // If account cache is empty, save mastodon account to kv.
  if mastodon_account_cache.is_ok() && mastodon_account_cache.unwrap().is_none() {
    let _ = cache::save_target_account(env, ctx, &mastodon_account);
  }

  // Get mastodon statuses.
  let mastodon_statuses = mastodon_api.get_account_status(&mastodon_account).await?;

  // Get sync status.
  let mut sync_status: HashMap<String, String> = cache::get_sync_status(env).await?;

  // If sync status is empty, initialize status and finish process.
  if sync_status.is_empty() {
    console_log!("Initialize sync status.");

    sync_status = cache::init_sync_status_from_statuses(&mastodon_statuses);
    let _ = cache::save_sync_status(env, ctx, &sync_status);

    return Ok(sync_status);
  }

  // Get sync target.
  let sync_target: Vec<&MastodonStatus> = mastodon_statuses
    .iter()
    .rev()
    .filter(|status| !sync_status.contains_key(&status.id))
    .filter(|status| {
      status.in_reply_to_account_id.is_none()
        || status
          .in_reply_to_account_id
          .to_owned()
          .unwrap_or("".to_string())
          == status.account.id
    })
    .collect();

  console_log!(
    "Target mastodon status: {statuses:?}",
    statuses = &sync_target
      .iter()
      .map(|&status| status.id.to_string())
      .collect::<Vec<String>>()
  );

  // If sync target is empty, finish process.
  if sync_target.is_empty() {
    return Ok(sync_status);
  }

  // Post tweet.
  let twitter_api = TwitterApi::new(env)?;
  for status in sync_target {
    // Upload media.
    let mut media_ids: Option<Vec<String>> = None;
    if !status.media_attachments.is_empty() && status.spoiler_text.is_empty() {
      let mut ids: Vec<String> = Vec::new();

      for attachment in &status.media_attachments {
        // If content type is "video", skip upload.
        if attachment.r#type == "video" {
          continue;
        }
        let media = match mastodon_api.get_media_attachment(&attachment.url).await {
          Ok(media) => media,
          Err(error) => {
            console_error!(
              "Media upload failed: {url} ({error:#})",
              url = &attachment.url,
              error = &error
            );
            continue;
          }
        };
        let media_id = match twitter_api
          .upload_media(media, &attachment.description)
          .await
        {
          Ok(id) => id,
          Err(error) => {
            console_error!(
              "Media upload failed: {url} ({error:#})",
              url = &attachment.url,
              error = &error
            );
            continue;
          }
        };
        console_log!(
          "Media upload completed: {url} -> {media_id}",
          url = &attachment.url,
          media_id = &media_id
        );
        ids.push(media_id);
      }

      media_ids = match ids.is_empty() {
        true => None,
        false => Some(ids),
      };
    }

    // Bulid tweet body.
    let body = twitter_api.build_body(status, &sync_status, media_ids.as_ref())?;

    // Post.
    let tweet_id = match twitter_api.post_tweet(body).await {
      Ok(tweet_id) => {
        console_log!(
          "Sync completed: {status_id} -> {tweet_id}",
          status_id = &status.id,
          tweet_id = &tweet_id
        );
        tweet_id
      }
      Err(error) => {
        console_error!(
          "Sync failed: {status_id} ({error:#})",
          status_id = &status.id,
          error = error
        );
        "".to_string()
      }
    };

    sync_status.insert(status.id.to_owned(), tweet_id);
  }

  // Save sync status to kv.
  let _ = cache::save_sync_status(env, ctx, &sync_status);

  Ok(sync_status)
}
