use std::error::Error;
use twapi_v2::api::post_2_tweets;
use twapi_v2::oauth10a::OAuthAuthentication;
use worker::Env;

pub fn create_auth(env: &Env) -> Result<OAuthAuthentication, Box<dyn Error>> {
  Result::Ok(OAuthAuthentication::new(
    env.secret("TWITTER_CONSUMER_KEY")?.to_string(),
    env.secret("TWITTER_CONSUMER_SECRET")?.to_string(),
    env.secret("TWITTER_ACCESS_TOKEN")?.to_string(),
    env.secret("TWITTER_ACCESS_SECRET")?.to_string(),
  ))
}

pub async fn post_tweet(auth: &OAuthAuthentication, text: &str) -> Result<String, Box<dyn Error>> {
  let body = post_2_tweets::Body {
    text: Some(text.to_string()),
    ..Default::default()
  };
  let (response, _header) = post_2_tweets::Api::new(body).execute(auth).await?;
  Ok(response.data.unwrap().id.unwrap_or_default())
}
