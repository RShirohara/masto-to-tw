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
    .filter(|status| !sync_status.contains_key(&status.id))
    .collect();
  if sync_target.is_empty() {
    return Ok(sync_status);
  }

  let twitter_auth = twitter::create_authentication(&env)?;
  for status in sync_target.iter() {
    let tweet_id = match twitter::post_tweet(&twitter_auth, &status.text).await {
      Ok(id) => id,
      Err(error) => return Err(error),
    };
    sync_status.insert(status.id.clone(), tweet_id);
  }

  kv::save_sync_status(&env, &sync_status).await?;

  Ok(sync_status)
}
