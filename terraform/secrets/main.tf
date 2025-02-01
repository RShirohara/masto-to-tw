terraform {
  required_providers {
    cloudflare = {
      source = "cloudflare/cloudflare"
      version = "~> 5.0.0"
    }
  }
}

provider "cloudflare" {
  api_token = var.cloudflare_api_token
}

resource "cloudflare_worker_secret" "mastodon_instance_url" {
  account_id  = var.cloudflare_account_id
  name        = "MASTODON_INSTANCE_URL"
  script_name = "masto-to-tw"
  secret_text = var.mastodon_instance_url
}

resource "cloudflare_worker_secret" "mastodon_account_acct" {
  account_id  = var.cloudflare_account_id
  name        = "MASTODON_ACCOUNT_ACCT"
  script_name = "masto-to-tw"
  secret_text = var.mastodon_account_acct
}

resource "cloudflare_worker_secret" "mastodon_access_token" {
  account_id  = var.cloudflare_account_id
  name        = "MASTODON_ACCESS_TOKEN"
  script_name = "masto-to-tw"
  secret_text = var.mastodon_access_token
}

resource "cloudflare_worker_secret" "twitter_consumer_key" {
  account_id  = var.cloudflare_account_id
  name        = "TWITTER_CONSUMER_KEY"
  script_name = "masto-to-tw"
  secret_text = var.twitter_consumer_key
}

resource "cloudflare_worker_secret" "twitter_consumer_secret" {
  account_id  = var.cloudflare_account_id
  name        = "TWITTER_CONSUMER_SECRET"
  script_name = "masto-to-tw"
  secret_text = var.twitter_consumer_secret
}

resource "cloudflare_worker_secret" "twitter_access_token" {
  account_id  = var.cloudflare_account_id
  name        = "TWITTER_ACCESS_TOKEN"
  script_name = "masto-to-tw"
  secret_text = var.twitter_access_token
}

resource "cloudflare_worker_secret" "twitter_access_secret" {
  account_id  = var.cloudflare_account_id
  name        = "TWITTER_ACCESS_SECRET"
  script_name = "masto-to-tw"
  secret_text = var.twitter_access_secret
}
