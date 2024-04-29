mod kv;
mod mastodon;
mod twitter;

use mastodon::{ClientEnv, Status};
use std::collections::HashMap;
use worker::{event, Context, Env, Request, Response, Result as WorkerResult};

#[event(fetch)]
pub async fn main(_req: Request, env: Env, _ctx: Context) -> WorkerResult<Response> {
  let mastodon_env = match ClientEnv::from_ctx(&env) {
    Ok(env) => env,
    Err(error) => {
      return Response::error(format!("Falid to retrieve environments: {error:#?}"), 500)
    }
  };

  let mastodon_account = match mastodon::lookup_account(
    &mastodon_env,
    env.secret("MASTODON_ACCOUNT_ACCT")?.to_string().as_str(),
  )
  .await
  {
    Ok(account) => account,
    Err(error) => return Response::error(format!("Failed to lookup account: {error:#?}"), 500),
  };

  let mastodon_statuses = match mastodon::retrieve_status(&mastodon_env, &mastodon_account.id).await
  {
    Ok(statuses) => statuses,
    Err(error) => return Response::error(format!("Failed to retrieve statuses: {error:#?}"), 500),
  };

  let mut synced_statuses = match kv::retrieve_synced_statuses(&env).await {
    Ok(statuses) => statuses,
    Err(error) => {
      return Response::error(format!("Failed to retrieve synced status: {error:#?}"), 500)
    }
  };

  if synced_statuses.is_empty() {
    let synced_statuses = initialize_synced_statuses(&mastodon_statuses);
    match kv::save_synced_statuses(&env, &synced_statuses).await {
      Ok(_) => return Response::from_json(&synced_statuses),
      Err(error) => {
        return Response::error(format!("Failed to save synced statuses: {error:#?}"), 500)
      }
    }
  }

  let sync_target = retrieve_post_target(&mastodon_statuses, &synced_statuses);

  if sync_target.is_empty() {
    return Response::from_json(&synced_statuses);
  }

  let twitter_auth = match twitter::create_auth(&env) {
    Ok(auth) => auth,
    Err(error) => {
      return Response::error(format!("Failed to retrieve environments: {error:#?}"), 500)
    }
  };
  for status in sync_target.iter() {
    let id = match twitter::post_tweet(&twitter_auth, &status.text).await {
      Ok(id) => id,
      Err(_) => continue,
    };
    synced_statuses.insert(status.id.clone(), id);
  }

  match kv::save_synced_statuses(&env, &synced_statuses).await {
    Ok(_) => Response::from_json(&synced_statuses),
    Err(error) => Response::error(format!("Failed to save synced posts: {error:#?}"), 500),
  }
}

fn initialize_synced_statuses(statuses: &Vec<Status>) -> HashMap<String, String> {
  let mut map: HashMap<String, String> = HashMap::new();
  for status in statuses.iter() {
    map.insert(status.id.clone(), "".to_string());
  }
  map
}

fn retrieve_post_target(
  statuses: &Vec<Status>,
  synced_statuses: &HashMap<String, String>,
) -> Vec<Status> {
  statuses
    .into_iter()
    .filter(|status| !synced_statuses.contains_key(&status.id))
    .cloned()
    .collect()
}
