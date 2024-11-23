mod api;

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
  // Get mastodon account
  let mastodon_api = MastodonApi::new(env)?;
  let mastodon_account = mastodon_api
    .lookup_account(env.secret("MASTODON_ACCOUNT_ACCT")?.to_string().as_ref())
    .await?;

  // Get mastodon statuses
  let mastodon_statuses = mastodon_api.get_account_status(&mastodon_account).await?;

  Ok(mastodon_statuses)
}
