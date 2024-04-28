use reqwest::{header, Client};
use serde::{Deserialize, Serialize};
use std::error::Error;
use worker::RouteContext;

pub struct ClientEnv {
  domain: String,
  access_token: String,
  user_agent: String,
}

impl ClientEnv {
  pub fn from_ctx(context: &RouteContext<()>) -> Result<Self, Box<dyn Error>> {
    Result::Ok(ClientEnv {
      domain: context.secret("MASTODON_INSTANCE_URL")?.to_string(),
      access_token: context.secret("MASTODON_ACCESS_TOKEN")?.to_string(),
      user_agent: "MastoToTw".to_string(),
    })
  }
}

pub async fn lookup_account(env: &ClientEnv, acct: &str) -> Result<Account, Box<dyn Error>> {
  let url = format!("{}/api/v1/accounts/lookup", env.domain);
  let query = format!("?acct={acct}");

  let client = Client::new();
  let response = client
    .get(format!("{url}{query}"))
    .header(
      header::AUTHORIZATION,
      format!("Bearer {}", env.access_token),
    )
    .header(header::USER_AGENT, env.user_agent.clone())
    .send()
    .await?;

  let account: Account = serde_json::from_str(response.text().await?.as_str())?;
  Ok(account)
}

pub async fn retrieve_status(env: &ClientEnv, id: &str) -> Result<Vec<Status>, Box<dyn Error>> {
  let url = format!("{}/api/v1/accounts/{id}/statuses", env.domain);
  let query = "?exclude_reblogs=true&only_public=true";

  let client = Client::new();
  let response = client
    .get(format!("{url}{query}"))
    .header(
      header::AUTHORIZATION,
      format!("Bearer {}", env.access_token),
    )
    .header(header::USER_AGENT, env.user_agent.clone())
    .send()
    .await?;

  let statuses: Vec<Status> = serde_json::from_str(response.text().await?.as_str())?;
  Ok(statuses)
}

#[derive(Serialize, Deserialize)]
pub struct Account {
  pub id: String,
}

#[derive(Serialize, Deserialize)]
pub struct Status {
  pub id: String,
  pub text: String,
  pub media_attachments: Vec<MediaAttachment>,
}

#[derive(Serialize, Deserialize)]
pub struct MediaAttachment {
  pub id: String,
  pub url: String,
  pub description: String,
}
