use std::{collections::HashMap, error::Error};

use worker::{Context, Env};

use crate::api::mastodon::Status;

use super::{save_to_kv, KV_BINDING_NAME};

pub async fn get_sync_status(env: &Env) -> Result<HashMap<String, String>, Box<dyn Error>> {
  let kv = env.kv(KV_BINDING_NAME)?;
  let sync_status: HashMap<String, String> = kv
    .get("sync_status")
    .json()
    .await?
    .unwrap_or_else(HashMap::new);

  Ok(sync_status)
}

pub fn save_sync_status(
  env: &Env,
  ctx: &Context,
  status: &HashMap<String, String>,
) -> Result<(), Box<dyn Error>> {
  let kv = env.kv(KV_BINDING_NAME)?;
  let status_encoded = serde_json::to_string(status)?;

  ctx.wait_until(save_to_kv(kv, "sync_status", status_encoded, 604800));

  Ok(())
}

pub fn init_sync_status_from_statuses(statuses: &Vec<Status>) -> HashMap<String, String> {
  let mut sync_status: HashMap<String, String> = HashMap::new();
  for status in statuses {
    sync_status.insert(status.id.to_owned(), "".to_string());
  }

  sync_status
}
