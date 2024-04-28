mod mastodon;
mod twitter;

use worker::*;

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
  let router = Router::new();

  router
    .get_async("/mastodon", |_req, ctx| async move {
      let env = match mastodon::ClientEnv::from_ctx(&ctx) {
        std::result::Result::Ok(env) => env,
        std::result::Result::Err(error) => return Response::error(error.to_string(), 500),
      };
      let account = match mastodon::lookup_account(
        &env,
        ctx.secret("MASTODON_ACCOUNT_ACCT")?.to_string().as_str(),
      )
      .await
      {
        std::result::Result::Ok(account) => account,
        std::result::Result::Err(error) => return Response::error(error.to_string(), 501),
      };
      let statuses = match mastodon::retrieve_status(&env, &account.id).await {
        std::result::Result::Ok(statuses) => statuses,
        std::result::Result::Err(error) => return Response::error(error.to_string(), 502),
      };

      match serde_json::to_string(&statuses) {
        std::result::Result::Ok(json) => Response::ok(json),
        std::result::Result::Err(error) => Response::error(error.to_string(), 503),
      }
    })
    .get_async("/twitter/post", |_req, ctx| async move {
      let auth = twitter::create_auth(&ctx)?;
      let result = twitter::post_tweet(&auth, "This is test tweet from api.").await;

      match result {
        std::result::Result::Ok(id) => Response::ok(id),
        std::result::Result::Err(error) => Response::error(error.to_string(), 503),
      }
    })
    .run(req, env)
    .await
}
