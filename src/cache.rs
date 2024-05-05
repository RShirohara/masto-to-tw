use std::{collections::HashMap, error::Error};

use worker::{kv::KvStore, Env, ScheduleContext};

use crate::mastodon::Account;

const KV_BINDING_NAME: &str = "cache";

pub async fn retrieve_sync_status(env: &Env) -> Result<HashMap<String, String>, Box<dyn Error>> {
  let kv = env.kv(KV_BINDING_NAME)?;
  let status: HashMap<String, String> = kv
    .get("sync_status")
    .json()
    .await?
    .unwrap_or_else(HashMap::new);
  Ok(status)
}

pub fn save_sync_status(
  env: &Env,
  ctx: &ScheduleContext,
  status: &HashMap<String, String>,
) -> Result<(), Box<dyn Error>> {
  let kv = env.kv(KV_BINDING_NAME)?;
  let status_encoded = serde_json::to_string(&status)?;
  ctx.wait_until(save_cache(kv, "sync_status", status_encoded));
  Ok(())
}

pub async fn retrieve_account(env: &Env) -> Result<Option<Account>, Box<dyn Error>> {
  let kv = env.kv(KV_BINDING_NAME)?;
  let account: Option<Account> = kv.get("account").json().await?;
  Ok(account)
}

pub fn save_account(
  env: &Env,
  ctx: &ScheduleContext,
  account: &Account,
) -> Result<(), Box<dyn Error>> {
  let kv = env.kv(KV_BINDING_NAME)?;
  let account_encoded = serde_json::to_string(&account)?;
  ctx.wait_until(save_cache(kv, "account", account_encoded));
  Ok(())
}

async fn save_cache(store: KvStore, name: &str, value: String) {
  let _ = store
    .put(name, value)
    .unwrap()
    .expiration_ttl(86400)
    .execute()
    .await;
}
