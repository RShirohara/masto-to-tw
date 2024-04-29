use std::{collections::HashMap, error::Error};

use worker::Env;

pub async fn retrieve_sync_status(env: &Env) -> Result<HashMap<String, String>, Box<dyn Error>> {
  let kv = env.kv("MASTO_TO_TW")?;
  let status: HashMap<String, String> = match kv.get("sync_status").json().await? {
    Some(status) => status,
    None => HashMap::new(),
  };
  Ok(status)
}

pub async fn save_sync_status(
  env: &Env,
  status: &HashMap<String, String>,
) -> Result<(), Box<dyn Error>> {
  let kv = env.kv("MASTO_TO_TW")?;
  let status_encoded = serde_json::to_string(&status)?;
  kv.put("sync_status", status_encoded)?.execute().await?;
  Ok(())
}
