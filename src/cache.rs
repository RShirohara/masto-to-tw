use worker::kv::KvStore;

mod target_account;

pub use target_account::{get_target_account, save_target_account};

const KV_BINDING_NAME: &str = "cache";

async fn save_to_kv(store: KvStore, name: &str, value: String, expiration_ttl: u64) {
  let _ = store
    .put(name, value)
    .unwrap()
    .expiration_ttl(expiration_ttl)
    .execute()
    .await;
}
