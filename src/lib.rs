mod kv;
mod mastodon;
mod twitter;

use std::{collections::HashMap, error::Error};

use mastodon::Status;
use worker::{event, Context, Env, Request, Response, Result as WorkerResult};

#[event(fetch)]
pub async fn fetch(_req: Request, env: Env, _ctx: Context) -> WorkerResult<Response> {
  match sync_statuses(&env).await {
    Ok(sync_status) => Response::from_json(&sync_status),
    Err(error) => Response::error(format!("Failed to sync status: {error:#?}"), 500),
  }
}

async fn sync_statuses(env: &Env) -> Result<HashMap<String, String>, Box<dyn Error>> {
  // Retrieve mastodon statuses
  let statuses = mastodon::retrieve_statuses(&env).await?;

  // Retrieve status for sync
  let mut sync_status = kv::retrieve_sync_status(&env).await?;
  if sync_status.is_empty() {
    for status in statuses.iter() {
      sync_status.insert(status.id.to_owned(), "".to_string());
    }
    kv::save_sync_status(&env, &sync_status).await?;
    return Ok(sync_status);
  }

  // Extract sync target
  let sync_target: Vec<Status> = statuses
    .to_owned()
    .into_iter()
    .rev()
    .filter(|status| !sync_status.contains_key(&status.id))
    .filter(|status| {
      status.in_reply_to_account_id.is_none()
        || status.in_reply_to_account_id.to_owned().unwrap() == status.account.id
    })
    .collect();

  // Post tweet
  let twitter_auth = twitter::create_authentication(&env)?;
  for status in sync_target.iter() {
    // Upload media
    let media_ids = match status.media_attachments.is_empty() {
      true => None,
      false => {
        let mut ids: Vec<String> = Vec::new();
        for attachment in status.media_attachments.to_owned() {
          let id = match twitter::upload_image(
            &twitter_auth,
            attachment.url.as_str(),
            match attachment.description.is_empty() {
              true => None,
              false => Some(attachment.description.as_str()),
            },
          )
          .await
          {
            Ok(id) => id,
            Err(_) => continue,
          };
          ids.push(id)
        }
        match ids.is_empty() {
          true => None,
          false => Some(ids),
        }
      }
    };

    // Retrieve tweet_id for reply
    let reply_to = match status.in_reply_to_id.to_owned() {
      Some(id) => match sync_status.contains_key(id.as_str()) {
        true => Some(sync_status.get(id.as_str()).unwrap().as_str()),
        false => None,
      },
      None => None,
    };

    // Post
    let tweet_id = twitter::post_tweet(&twitter_auth, &status.text, reply_to, media_ids).await?;

    sync_status.insert(status.id.to_owned(), tweet_id);
  }

  // Save status for sync
  kv::save_sync_status(&env, &sync_status).await?;

  Ok(sync_status)
}
