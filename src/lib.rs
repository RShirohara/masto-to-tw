mod cache;
mod mastodon;
mod twitter;

use std::{collections::HashMap, error::Error};

use mastodon::Status;
use worker::{event, Env, ScheduleContext, ScheduledEvent};

#[event(scheduled)]
pub async fn scheduled(_event: ScheduledEvent, env: Env, ctx: ScheduleContext) {
  let _ = sync_statuses(&env, &ctx).await;
}

async fn sync_statuses(
  env: &Env,
  ctx: &ScheduleContext,
) -> Result<HashMap<String, String>, Box<dyn Error>> {
  // Retrieve mastodon statuses
  let statuses = mastodon::retrieve_statuses(env, ctx).await?;

  // Retrieve sync status
  let mut sync_status: HashMap<String, String> = cache::retrieve_sync_status(env).await?;

  // If sync status is empty, initialize and finish process
  if sync_status.is_empty() {
    for status in statuses {
      sync_status.insert(status.id.to_owned(), "".to_string());
    }
    let _ = cache::save_sync_status(env, ctx, &sync_status);
    return Ok(sync_status);
  }

  // Extract sync target
  let sync_target: Vec<&Status> = statuses
    .iter()
    .rev()
    .filter(|status| !sync_status.contains_key(&status.id))
    .filter(|status| {
      status.in_reply_to_account_id.is_none()
        || status.in_reply_to_account_id.to_owned().unwrap() == status.account.id
    })
    .collect();

  // If sync target is empty, finish process
  if sync_target.is_empty() {
    return Ok(sync_status);
  }

  // Post tweet
  let twitter_auth = twitter::create_authentication(env)?;
  for status in sync_target {
    let tweet_id = twitter::post_tweet_from_status(&twitter_auth, status, &sync_status)
      .await
      .unwrap_or_else(|_| "".to_string());
    sync_status.insert(status.id.to_owned(), tweet_id);
  }

  // Save sync status
  let _ = cache::save_sync_status(env, ctx, &sync_status);

  Ok(sync_status)
}
