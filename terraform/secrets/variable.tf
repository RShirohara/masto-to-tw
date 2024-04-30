variable "cloudflare_account_id" {
  type        = string
  description = "The cloudflare account identifier to target for the resource."
}

variable "cloudflare_api_token" {
  type        = string
  description = "The API Token for operation to cloudflare."
}

variable "mastodon_instance_url" {
  type        = string
  description = "The URL of the Mastodon instance from which to retrieve the post."
}

variable "mastodon_account_acct" {
  type        = string
  description = "The ID of the Mastodon account from which to retrieve the post."
}

variable "mastodon_access_token" {
  type        = string
  description = "The API Token for operation to mastodon instance."
}

variable "twitter_consumer_key" {
  type        = string
  description = "The App Key for operation to twitter."
}

variable "twitter_consumer_secret" {
  type        = string
  description = "The App Secret for operation to twitter."
}

variable "twitter_access_token" {
  type        = string
  description = "The Access Token for operation to twitter."
}

variable "twitter_access_secret" {
  type        = string
  description = "The Access Token Secret for operation to twitter."
}
