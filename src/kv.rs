use std::{collections::HashMap, error::Error};
use worker::Env;

pub async fn retrieve_synced_statuses(
  env: &Env,
) -> Result<HashMap<String, String>, Box<dyn Error>> {
  let kv = env.kv("MASTO_TO_TW")?;
  let statuses: HashMap<String, String> = match kv.get("synced_statuses").json().await? {
    Some(statuses) => statuses,
    None => HashMap::new(),
  };
  Ok(statuses)
}

pub async fn save_synced_statuses(
  env: &Env,
  statuses: &HashMap<String, String>,
) -> Result<(), Box<dyn Error>> {
  let kv = env.kv("MASTO_TO_TW")?;
  let encoded_statuses = serde_json::to_string(&statuses)?;
  kv.put("synced_statuses", encoded_statuses)?
    .execute()
    .await?;
  Ok(())
}
