use std::error::Error;

use reqwest::{header, Client, Response};
use serde::{Deserialize, Serialize};
use worker::{Env, ScheduleContext};

const USER_AGENT: &str = "MastoToTw";

pub async fn retrieve_statuses(
  env: &Env,
  ctx: &ScheduleContext,
) -> Result<Vec<Status>, Box<dyn Error>> {
  let mastodon_env = MastodonEnv::from_worker_env(env)?;

  // Retrieve account
  let account_cache = crate::cache::retrieve_account(env).await?;
  let account = match account_cache.to_owned() {
    Some(account) => account,
    None => {
      lookup_account(
        &mastodon_env,
        env.secret("MASTODON_ACCOUNT_ACCT")?.to_string().as_str(),
      )
      .await?
    }
  };
  if account_cache.is_none() {
    let _ = crate::cache::save_account(env, ctx, &account);
  }

  // Retrieve statuses
  let statuses = retrieve_account_statuses(&mastodon_env, &account).await?;

  Ok(statuses)
}

// Account
async fn lookup_account(env: &MastodonEnv, acct: &str) -> Result<Account, Box<dyn Error>> {
  let client = Client::new();
  let response = client
    .get(format!(
      "{domain}/api/v1/accounts/lookup",
      domain = env.domain
    ))
    .query(&[("acct", acct)])
    .header(header::USER_AGENT, USER_AGENT)
    .bearer_auth(env.access_token.as_str())
    .send()
    .await?;

  let account: Account = serde_json::from_str(response.text().await?.as_str())?;

  Ok(account)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Account {
  pub id: String,
}

// Status
async fn retrieve_account_statuses(
  env: &MastodonEnv,
  account: &Account,
) -> Result<Vec<Status>, Box<dyn Error>> {
  let client = Client::new();
  let response = client
    .get(format!(
      "{domain}/api/v1/accounts/{account_id}/statuses",
      domain = env.domain,
      account_id = account.id
    ))
    .query(&[("exclude_reblogs", true), ("only_public", true)])
    .header(header::USER_AGENT, USER_AGENT)
    .bearer_auth(env.access_token.as_str())
    .send()
    .await?;

  let statuses: Vec<Status> = serde_json::from_str(response.text().await?.as_str())?;

  Ok(statuses)
}

#[derive(Clone, Deserialize)]
pub struct Status {
  pub account: Account,
  pub id: String,
  pub in_reply_to_account_id: Option<String>,
  pub in_reply_to_id: Option<String>,
  pub media_attachments: Vec<MediaAttachment>,
  pub spoiler_text: String,
  pub text: String,
  pub url: String,
}

#[derive(Clone, Deserialize)]
pub struct MediaAttachment {
  pub description: Option<String>,
  pub id: String,
  pub url: String,
}

// Media
pub async fn retrieve_media_attachment(url: &str) -> Result<Media, Box<dyn Error>> {
  let client = Client::new();
  let response = client
    .get(url)
    .header(header::USER_AGENT, USER_AGENT)
    .send()
    .await?;

  Ok(Media {
    content_size: response
      .headers()
      .get(header::CONTENT_LENGTH)
      .unwrap()
      .to_str()?
      .parse::<u64>()?,
    content_type: response
      .headers()
      .get(header::CONTENT_TYPE)
      .unwrap()
      .to_str()?
      .to_string(),
    response,
  })
}

pub struct Media {
  pub content_type: String,
  pub content_size: u64,
  pub response: Response,
}

// Environment
struct MastodonEnv {
  access_token: String,
  domain: String,
}

impl MastodonEnv {
  pub fn from_worker_env(env: &Env) -> Result<Self, Box<dyn Error>> {
    Ok(MastodonEnv {
      domain: env.secret("MASTODON_INSTANCE_URL")?.to_string(),
      access_token: env.secret("MASTODON_ACCESS_TOKEN")?.to_string(),
    })
  }
}
