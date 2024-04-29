use std::error::Error;
use twapi_v2::api::post_2_tweets;
use twapi_v2::oauth10a::OAuthAuthentication;
use worker::RouteContext;

pub fn create_auth(context: &RouteContext<()>) -> Result<OAuthAuthentication, Box<dyn Error>> {
  Result::Ok(OAuthAuthentication::new(
    context.secret("TWITTER_CONSUMER_KEY")?.to_string(),
    context.secret("TWITTER_CONSUMER_SECRET")?.to_string(),
    context.secret("TWITTER_ACCESS_TOKEN")?.to_string(),
    context.secret("TWITTER_ACCESS_SECRET")?.to_string(),
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
