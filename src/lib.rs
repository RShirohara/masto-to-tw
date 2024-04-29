mod mastodon;
mod twitter;

use worker::{event, Context, Env, Request, Response, Result as WorkerResult, Router};

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> WorkerResult<Response> {
  let router = Router::new();

  router
    .get_async("/mastodon", |_req, ctx| async move {
      let env = match mastodon::ClientEnv::from_ctx(&ctx) {
        Ok(env) => env,
        Err(error) => {
          return Response::error(format!("Failed to retrieve environments: {error:#?}"), 500)
        }
      };

      let account = match mastodon::lookup_account(
        &env,
        ctx.secret("MASTODON_ACCOUNT_ACCT")?.to_string().as_str(),
      )
      .await
      {
        Ok(account) => account,
        Err(error) => return Response::error(format!("Failed to lookup account: {error:#?}"), 500),
      };

      let statuses = match mastodon::retrieve_status(&env, &account.id).await {
        Ok(statuses) => statuses,
        Err(error) => {
          return Response::error(format!("Failed to retrieve statuses: {error:#?}"), 500)
        }
      };

      Response::from_json(&statuses)
    })
    .get_async("/twitter/post", |_req, ctx| async move {
      let auth = match twitter::create_auth(&ctx) {
        Ok(auth) => auth,
        Err(error) => {
          return Response::error(
            format!("Falied to retrieve authentications: {error:#?}"),
            500,
          )
        }
      };

      match twitter::post_tweet(&auth, "This is test tweet from api.").await {
        Ok(id) => Response::ok(id),
        Err(error) => Response::error(format!("Failed to post tweet: {error:#?}"), 500),
      }
    })
    .run(req, env)
    .await
}
