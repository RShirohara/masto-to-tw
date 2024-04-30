use std::error::Error;

use reqwest::{header, Client};
use serde::Deserialize;
use worker::Env;

const USER_AGENT: &str = "MastoToTw";

pub async fn retrieve_statuses(env: &Env) -> Result<Vec<Status>, Box<dyn Error>> {
  let mastodon_env = MastodonEnv::from_worker_env(&env)?;
  let account = lookup_account(
    &mastodon_env,
    env.secret("MASTODON_ACCOUNT_ACCT")?.to_string().as_str(),
  )
  .await?;
  let statuses = retrieve_account_statuses(&mastodon_env, &account).await?;

  Ok(statuses)
}

// Account
async fn lookup_account(env: &MastodonEnv, acct: &str) -> Result<Account, Box<dyn Error>> {
  let client = Client::new();
  let response = client
    .get(format!("{}/api/v1/accounts/lookup", env.domain).as_str())
    .query(&[("acct", acct)])
    .header(header::USER_AGENT, USER_AGENT)
    .bearer_auth(env.access_token.as_str())
    .send()
    .await?;

  let account: Account = serde_json::from_str(response.text().await?.as_str())?;

  Ok(account)
}

#[derive(Clone, Deserialize)]
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
    .get(format!("{}/api/v1/accounts/{}/statuses", env.domain, account.id).as_str())
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
  pub id: String,
  pub text: String,
  pub account: Account,
  pub in_reply_to_id: Option<String>,
  pub in_reply_to_account_id: Option<String>,
  pub media_attachments: Vec<MediaAttachment>,
}

#[derive(Clone, Deserialize)]
pub struct MediaAttachment {
  pub id: String,
  pub url: String,
  pub description: String,
}

// Environment
struct MastodonEnv {
  domain: String,
  access_token: String,
}

impl MastodonEnv {
  pub fn from_worker_env(env: &Env) -> Result<Self, Box<dyn Error>> {
    Ok(MastodonEnv {
      domain: env.secret("MASTODON_INSTANCE_URL")?.to_string(),
      access_token: env.secret("MASTODON_ACCESS_TOKEN")?.to_string(),
    })
  }
}
