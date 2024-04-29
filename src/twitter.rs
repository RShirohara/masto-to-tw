use std::error::Error;

use twapi_v2::{
  api::post_2_tweets::{Api as PostApi, Body as PostBody},
  oauth10a::OAuthAuthentication,
};
use worker::Env;

pub fn create_authentication(env: &Env) -> Result<OAuthAuthentication, Box<dyn Error>> {
  Ok(OAuthAuthentication::new(
    env.secret("TWITTER_CONSUMER_KEY")?.to_string(),
    env.secret("TWITTER_CONSUMER_SECRET")?.to_string(),
    env.secret("TWITTER_ACCESS_TOKEN")?.to_string(),
    env.secret("TWITTER_ACCESS_SECRET")?.to_string(),
  ))
}

pub async fn post_tweet(auth: &OAuthAuthentication, text: &str) -> Result<String, Box<dyn Error>> {
  let body = PostBody {
    text: Some(text.to_string()),
    ..Default::default()
  };
  let (response, _) = PostApi::new(body).execute(auth).await?;
  let id = match response.data {
    Some(data) => data.id.unwrap_or("".to_string()),
    None => "".to_string(),
  };
  Ok(id)
}
