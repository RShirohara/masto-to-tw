mod api;
mod cache;

use std::error::Error;

use api::mastodon::{Api as MastodonApi, Status};
use worker::{
  event, Context, Env, Request, Response, Result as WorkerResult, ScheduleContext, ScheduledEvent,
};

#[event(fetch)]
pub async fn fetch(_req: Request, env: Env, ctx: Context) -> WorkerResult<Response> {
  match sync_posts(&env, &ctx).await {
    Ok(statuses) => Response::ok(serde_json::to_string(&statuses)?),
    Err(error) => Response::error(error.to_string(), 500),
  }
}

#[event(scheduled)]
pub async fn scheduled(_event: ScheduledEvent, env: Env, _ctx: ScheduleContext) {}

async fn sync_posts(env: &Env, ctx: &Context) -> Result<Vec<Status>, Box<dyn Error>> {
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

  // If account cache is empty, save mastodon account to kv.
  if mastodon_account_cache.is_ok() && mastodon_account_cache.unwrap().is_none() {
    let _ = cache::save_target_account(env, ctx, &mastodon_account);
  }

  // Get mastodon statuses.
  let mastodon_statuses = mastodon_api.get_account_status(&mastodon_account).await?;

  Ok(mastodon_statuses)
}
