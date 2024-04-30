# `masto-to-tw`

[![LICENSE][license-badge]][license]

Sync Mastodon posts to Twitter.

## Requirements

- `cargo`: used to compile wasm.
- `pnpm`: used to install dependencies and run tasks.
- `terraform`: used to deploy Worker KV and Secrets.

## Setup

1. Generate credentials in the Twitter Developer Portal.
   - API Key and Secret
   - Access Token and Secret: required "Read and Write" permissions.
2. Generate access token in the Mastodon instance.
   - required `read` scopes.
3. Go to `terraform/kv` directory, and deploy Workers KV namespace.
4. Go to root directory for repository, and deploy Workers script.

   ```shell
   pnpm deploy
   ```

5. Go to `terraform/secrets` directory, and deploy Workers Secrets.

## LICENSE

[MIT][license]

<!-- Link definitions -->

[license-badge]: https://img.shields.io/github/license/RShirohara/masto-to-tw
[license]: ./LICENSE.md
