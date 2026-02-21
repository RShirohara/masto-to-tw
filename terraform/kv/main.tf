terraform {
  required_providers {
    cloudflare = {
      source = "cloudflare/cloudflare"
      version = "~> 5.17.0"
    }
  }
}

provider "cloudflare" {
  api_token = var.cloudflare_api_token
}

resource "cloudflare_workers_kv_namespace" "masto_to_tw" {
  account_id = var.cloudflare_account_id
  title      = "masto-to-tw"
}
