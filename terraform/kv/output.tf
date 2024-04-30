output "kv_namespace_id" {
  value     = cloudflare_workers_kv_namespace.masto_to_tw.id
  sensitive = true
}
