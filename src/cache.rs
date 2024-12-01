use worker::kv::KvStore;

mod sync_status;
mod target_account;

pub use sync_status::{get_sync_status, init_sync_status_from_statuses, save_sync_status};
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
