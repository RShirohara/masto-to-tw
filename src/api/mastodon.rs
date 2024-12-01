use std::error::Error;

use reqwest::{header, Client, Response};
use serde::{Deserialize, Serialize};
use worker::Env;

const USER_AGENT: &str = "masto-to-tw";

pub struct Api {
    auth: Auth,
    client: Client,
}

impl Api {
    pub fn new(env: &Env) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            auth: Auth::from_worker_env(env)?,
            client: Client::new(),
        })
    }
}

// Authentication
struct Auth {
    access_token: String,
    domain: String,
}

impl Auth {
    pub fn from_worker_env(env: &Env) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            access_token: env.secret("MASTODON_ACCESS_TOKEN")?.to_string(),
            domain: env.secret("MASTODON_INSTANCE_URL")?.to_string(),
        })
    }
}

// Account
#[derive(Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: String,
}

impl Api {
    pub async fn lookup_account(&self, acct: &str) -> Result<Account, Box<dyn Error>> {
        let response = self
            .client
            .get(format!(
                "{domain}/api/v1/accounts/lookup",
                domain = self.auth.domain,
            ))
            .query(&[("acct", acct)])
            .header(header::USER_AGENT, USER_AGENT)
            .bearer_auth(self.auth.access_token.as_str())
            .send()
            .await?;
        let account: Account = serde_json::from_str(response.text().await?.as_str())?;

        Ok(account)
    }
}

// Status
#[derive(Clone, Serialize, Deserialize)]
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

#[derive(Clone, Serialize, Deserialize)]
pub struct MediaAttachment {
    pub description: Option<String>,
    pub r#type: String,
    pub url: String,
}

impl Api {
    pub async fn get_account_status(
        &self,
        account: &Account,
    ) -> Result<Vec<Status>, Box<dyn Error>> {
        let response = self
            .client
            .get(format!(
                "{domain}/api/v1/accounts/{account_id}/statuses",
                domain = self.auth.domain,
                account_id = account.id
            ))
            .query(&[("exclude_reblogs", true), ("only_public", true)])
            .header(header::USER_AGENT, USER_AGENT)
            .bearer_auth(self.auth.access_token.as_str())
            .send()
            .await?;
        let statuses: Vec<Status> = serde_json::from_str(response.text().await?.as_str())?;

        Ok(statuses)
    }
}

// Media
pub struct Media {
    pub content_type: String,
    pub content_size: u64,
    pub response: Response,
}

impl Api {
    pub async fn get_media_attachment(&self, url: &str) -> Result<Media, Box<dyn Error>> {
        let response = self
            .client
            .get(url)
            .header(header::USER_AGENT, USER_AGENT)
            .send()
            .await?;

        Ok(Media {
            content_type: response
                .headers()
                .get(header::CONTENT_TYPE)
                .unwrap()
                .to_str()?
                .to_string(),
            content_size: response
                .headers()
                .get(header::CONTENT_LENGTH)
                .unwrap()
                .to_str()?
                .parse::<u64>()?,
            response,
        })
    }
}
