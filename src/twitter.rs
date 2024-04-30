use std::{
  error::Error,
  io::{Cursor, Error as IoError, ErrorKind as IoErrorKind},
};

use futures::stream::TryStreamExt;
use reqwest::{header, Client};
use tokio::io::AsyncReadExt;
use tokio_util::io::StreamReader;
use twapi_v2::{
  api::{
    post_2_tweets::{Api as PostApi, Body as PostBody, Media, Reply},
    Authentication,
  },
  oauth10a::OAuthAuthentication,
  upload::{
    post_media_metadata_create::{AltText, Api as MediaMetadataApi, Body as MediaMetadataBody},
    post_media_upload_append::{Api as MediaUploadApi, Data as MediaUploadData},
    post_media_upload_finalize::{Api as MediaFinalizeApi, Data as MediaFinalizeData},
    post_media_upload_init::{Api as MediaInitApi, Data as MediaInitData},
  },
};
use worker::Env;

pub fn create_authentication(env: &Env) -> Result<impl Authentication, Box<dyn Error>> {
  Ok(OAuthAuthentication::new(
    env.secret("TWITTER_CONSUMER_KEY")?.to_string(),
    env.secret("TWITTER_CONSUMER_SECRET")?.to_string(),
    env.secret("TWITTER_ACCESS_TOKEN")?.to_string(),
    env.secret("TWITTER_ACCESS_SECRET")?.to_string(),
  ))
}

pub async fn post_tweet(
  auth: &impl Authentication,
  text: &str,
  reply_to: &Option<&str>,
  media_ids: &Option<Vec<String>>,
) -> Result<String, Box<dyn Error>> {
  let body = PostBody {
    text: Some(text.to_string()),
    reply: match reply_to {
      Some(id) => Some(Reply {
        in_reply_to_tweet_id: id.to_string(),
        ..Default::default()
      }),
      None => None,
    },
    media: match media_ids {
      Some(ids) => Some(Media {
        media_ids: ids.to_owned(),
        ..Default::default()
      }),
      None => None,
    },
    ..Default::default()
  };
  let (response, _) = PostApi::new(body).execute(auth).await?;
  let id = match response.data {
    Some(data) => data.id.unwrap_or("".to_string()),
    None => "".to_string(),
  };
  Ok(id)
}

pub async fn upload_image(
  auth: &impl Authentication,
  url: &str,
  alt: &Option<String>,
) -> Result<String, Box<dyn Error>> {
  // Retrieve source
  let client = Client::new();
  let source_response = client
    .get(url)
    .header(header::USER_AGENT, "Curl")
    .send()
    .await?;
  let content_type = source_response
    .headers()
    .get("Content-Type")
    .unwrap()
    .to_str()?;
  let content_size = source_response
    .headers()
    .get("Content-Length")
    .unwrap()
    .to_str()?
    .parse::<u64>()?;

  // Init
  let data = MediaInitData {
    total_bytes: content_size,
    media_type: content_type.to_string(),
    ..Default::default()
  };
  let (response, _) = MediaInitApi::new(data).execute(auth).await?;
  let media_id = response.media_id_string;

  // Append
  let stream = source_response.bytes_stream();
  let mut reader = StreamReader::new(stream.map_err(|e| IoError::new(IoErrorKind::Other, e)));
  let mut segment_index = 0;
  while segment_index * 1000000 < content_size {
    let read_size: usize = if (segment_index + 1) * 1000000 < content_size {
      1000000
    } else {
      (content_size - segment_index * 1000000) as usize
    };
    let mut cursor = Cursor::new(vec![0; read_size]);
    reader.read_exact(cursor.get_mut()).await?;
    let data = MediaUploadData {
      media_id: media_id.to_owned(),
      segment_index,
      cursor,
    };
    let _ = MediaUploadApi::new(data).execute(auth).await?;
    segment_index += 1;
  }

  // Finalize
  let data = MediaFinalizeData {
    media_id: media_id.to_owned(),
  };
  let _ = MediaFinalizeApi::new(data).execute(auth).await?;

  // Add alt text
  if let Some(text) = alt {
    let body = MediaMetadataBody {
      media_id: media_id.to_owned(),
      alt_text: AltText {
        text: text.to_string(),
      },
    };
    let _ = MediaMetadataApi::new(body).execute(auth).await?;
  }

  Ok(media_id)
}
