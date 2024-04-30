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
  let statuses = mastodon::retrieve_statuses(&env).await?;

  let mut sync_status = kv::retrieve_sync_status(env).await?;
  if sync_status.is_empty() {
    for status in statuses.iter() {
      sync_status.insert(status.id.clone(), "".to_string());
    }
    kv::save_sync_status(&env, &sync_status).await?;
    return Ok(sync_status);
  }

  let sync_target: Vec<Status> = statuses
    .clone()
    .into_iter()
    .rev()
    .filter(|status| !sync_status.contains_key(&status.id))
    .filter(|status| {
      status.in_reply_to_account_id.is_none()
        || status.in_reply_to_account_id.clone().unwrap() == status.account.id
    })
    .collect();
  if sync_target.is_empty() {
    return Ok(sync_status);
  }

  let twitter_auth = twitter::create_authentication(&env)?;
  for status in sync_target.iter() {
    let media_ids = if !status.media_attachments.is_empty() {
      let mut ids: Vec<String> = Vec::new();
      for attachment in status.media_attachments.clone() {
        let id = match twitter::upload_image(
          &twitter_auth,
          attachment.url.as_str(),
          &if !attachment.description.is_empty() {
            Some(attachment.description)
          } else {
            None
          },
        )
        .await {
          Ok(id) => id,
          Err(_) => continue
        };
        ids.push(id)
      }
      match !ids.is_empty() {
          true => Some(ids),
          false => None
      }
    } else {
      None
    };

    let reply_to = match &status.in_reply_to_id {
      Some(id) => match sync_status.contains_key(id.as_str()) {
        true => Some(sync_status.get(id.as_str()).unwrap().as_str()),
        false => None,
      },
      None => None,
    };

    let tweet_id =
      match twitter::post_tweet(&twitter_auth, &status.text, &reply_to, &media_ids).await {
        Ok(id) => id,
        Err(error) => return Err(error),
      };
    sync_status.insert(status.id.clone(), tweet_id);
  }

  kv::save_sync_status(&env, &sync_status).await?;

  Ok(sync_status)
}
