use std::error::Error;

use worker::{Env, ScheduleContext};

use crate::api::mastodon::Account;

use super::{save_to_kv, KV_BINDING_NAME};

pub async fn get_target_account(env: &Env) -> Result<Option<Account>, Box<dyn Error>> {
  let kv = env.kv(KV_BINDING_NAME)?;
  let account: Option<Account> = kv.get("target_account").json().await?;

  Ok(account)
}

pub fn save_target_account(
  env: &Env,
  ctx: &ScheduleContext,
  account: &Account,
) -> Result<(), Box<dyn Error>> {
  let kv = env.kv(KV_BINDING_NAME)?;
  let account_encoded = serde_json::to_string(account)?;

  ctx.wait_until(save_to_kv(kv, "target_account", account_encoded, 86400));

  Ok(())
}
