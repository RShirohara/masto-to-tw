use std::{
  collections::HashMap,
  error::Error,
  io::{Cursor, Error as IoError, ErrorKind as IoErrorKind},
};

use futures::stream::TryStreamExt;
use tokio::io::AsyncReadExt;
use tokio_util::io::StreamReader;
use twapi_v2::{
  api::{
    post_2_tweets::{Api as TweetApi, Body as TweetBody, Media as TweetMedia, Reply as TweetReply},
    Authentication,
  },
  oauth10a::OAuthAuthentication,
  upload::{
    post_media_metadata_create::{
      AltText as MediaMetadataAltText, Api as MediaMetadataApi, Body as MediaMetadataBody,
    },
    post_media_upload_append::{Api as MediaAppendApi, Data as MediaAppendData},
    post_media_upload_finalize::{Api as MediaFinalizeApi, Data as MediaFinalizeData},
    post_media_upload_init::{Api as MediaInitApi, Data as MediaInitData},
  },
};
use worker::Env;

use crate::mastodon::{Media as MastodonMedia, Status as MastodonStatus};

pub fn create_authentication(env: &Env) -> Result<impl Authentication, Box<dyn Error>> {
  Ok(OAuthAuthentication::new(
    env.secret("TWITTER_CONSUMER_KEY")?.to_string(),
    env.secret("TWITTER_CONSUMER_SECRET")?.to_string(),
    env.secret("TWITTER_ACCESS_TOKEN")?.to_string(),
    env.secret("TWITTER_ACCESS_SECRET")?.to_string(),
  ))
}

pub async fn post_tweet_from_status(
  auth: &impl Authentication,
  status: &MastodonStatus,
  sync_status: &HashMap<String, String>,
) -> Result<String, Box<dyn Error>> {
  // Upload media
  let media_ids = match status.media_attachments.is_empty() {
    true => None,
    false => {
      let mut ids: Vec<String> = Vec::new();
      for attachment in &status.media_attachments {
        let media = crate::mastodon::retrieve_media_attachment(&attachment.url).await?;
        let id = match upload_media(auth, media, attachment.description.as_deref()).await {
          Ok(id) => id,
          Err(_) => continue,
        };
        ids.push(id);
      }
      match ids.is_empty() {
        true => None,
        false => Some(ids),
      }
    }
  };

  // Retrieve tweet_id for reply
  let reply_to = status
    .in_reply_to_id
    .as_ref()
    .map(|id| sync_status.get(id.as_str()).unwrap().as_str());

  // Build text
  let heading_info: Vec<String> = [match status.spoiler_text.is_empty() {
    true => "".to_string(),
    false => format!("CW: {}", status.spoiler_text),
  }]
  .iter()
  .filter(|text| !text.is_empty())
  .cloned()
  .collect();
  let text = [
    heading_info.join(match heading_info.is_empty() {
      true => "",
      false => "\n",
    }),
    status.text.to_owned(),
  ]
  .join(match heading_info.is_empty() {
    true => "",
    false => "\n\n",
  });

  // Post
  let tweet_id = post_tweet(auth, &text, reply_to, media_ids).await?;

  Ok(tweet_id)
}

async fn post_tweet(
  auth: &impl Authentication,
  text: &str,
  reply_to: Option<&str>,
  media_ids: Option<Vec<String>>,
) -> Result<String, Box<dyn Error>> {
  let body = TweetBody {
    text: Some(text.to_string()),
    reply: reply_to.map(|id| TweetReply {
      in_reply_to_tweet_id: id.to_string(),
      ..Default::default()
    }),
    media: media_ids.map(|ids| TweetMedia {
      media_ids: ids.to_vec(),
      ..Default::default()
    }),
    ..Default::default()
  };
  let (response, _) = TweetApi::new(body).execute(auth).await?;
  let id = match response.data {
    Some(data) => data.id.unwrap(),
    None => "".to_string(),
  };

  Ok(id)
}

async fn upload_media(
  auth: &impl Authentication,
  media: MastodonMedia,
  description: Option<&str>,
) -> Result<String, Box<dyn Error>> {
  // Init
  let data = MediaInitData {
    total_bytes: media.content_size,
    media_type: media.content_type,
    ..Default::default()
  };
  let (response, _) = MediaInitApi::new(data).execute(auth).await?;
  let media_id = response.media_id_string;

  // Append
  const MEDIA_SPLIT_SIZE: u64 = 1000000;
  let stream = media.response.bytes_stream();
  let mut reader = StreamReader::new(stream.map_err(|e| IoError::new(IoErrorKind::Other, e)));
  let mut segment_index = 0;

  while segment_index * MEDIA_SPLIT_SIZE < media.content_size {
    let read_size: usize = if (segment_index + 1) * MEDIA_SPLIT_SIZE < media.content_size {
      MEDIA_SPLIT_SIZE.try_into().unwrap()
    } else {
      (media.content_size - segment_index * MEDIA_SPLIT_SIZE) as usize
    };
    let mut cursor = Cursor::new(vec![0; read_size]);

    reader.read_exact(cursor.get_mut()).await?;
    let data = MediaAppendData {
      media_id: media_id.to_owned(),
      segment_index,
      cursor,
    };
    let _ = MediaAppendApi::new(data).execute(auth).await?;

    segment_index += 1;
  }

  // Finalize
  let data = MediaFinalizeData {
    media_id: media_id.to_owned(),
  };
  let _ = MediaFinalizeApi::new(data).execute(auth).await?;

  // Add description
  if let Some(text) = description {
    let body = MediaMetadataBody {
      media_id: media_id.to_owned(),
      alt_text: MediaMetadataAltText {
        text: text.to_string(),
      },
    };
    let _ = MediaMetadataApi::new(body).execute(auth).await?;
  }

  Ok(media_id)
}
