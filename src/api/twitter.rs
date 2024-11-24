use std::{
  collections::HashMap,
  error::Error,
  io::{Cursor, Error as IoError, ErrorKind as IoErrorKind},
};

use futures::stream::TryStreamExt;
use tokio::io::AsyncReadExt;
use tokio_util::io::StreamReader;
use twapi_v2::{
  api::post_2_tweets::{
    Api as TweetApi, Body as TweetBody, Media as TweetMedia, Reply as TweetReply,
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

use super::mastodon::{Media as MastodonMedia, Status as MastodonStatus};

pub struct Api {
  auth: OAuthAuthentication,
}

impl Api {
  pub fn new(env: &Env) -> Result<Self, Box<dyn Error>> {
    Ok(Self {
      auth: OAuthAuthentication::new(
        env.secret("TWITTER_CONSUMER_KEY")?.to_string(),
        env.secret("TWITTER_CONSUMER_SECRET")?.to_string(),
        env.secret("TWITTER_ACCESS_TOKEN")?.to_string(),
        env.secret("TWITTER_ACCESS_SECRET")?.to_string(),
      ),
    })
  }
}

// Tweet
impl Api {
  pub fn build_body(
    &self,
    mastodon_status: &MastodonStatus,
    sync_status: &HashMap<String, String>,
    media_ids: Option<&Vec<String>>,
  ) -> Result<TweetBody, Box<dyn Error>> {
    // Get tweet_id for reply.
    let reply_to = mastodon_status.in_reply_to_id.as_ref().map(|id| {
      sync_status
        .get(id.as_str())
        .unwrap_or(&"".to_string())
        .to_owned()
    });

    // Build text.
    let mut is_cw = false;

    if !mastodon_status.spoiler_text.is_empty() {
      is_cw = true;
    }

    let text = match is_cw {
      true => format!(
        "CW: {spoiler_text}\n\n{url}",
        spoiler_text = mastodon_status.spoiler_text,
        url = mastodon_status.url
      ),
      false => mastodon_status.text.to_owned(),
    };

    // Build body.
    let body = TweetBody {
      text: Some(text.to_string()),
      reply: reply_to.map(|id| TweetReply {
        in_reply_to_tweet_id: id.to_owned(),
        ..Default::default()
      }),
      media: media_ids.map(|ids| TweetMedia {
        media_ids: ids.to_owned(),
        ..Default::default()
      }),
      ..Default::default()
    };

    Ok(body)
  }

  pub async fn post_tweet(&self, body: TweetBody) -> Result<String, Box<dyn Error>> {
    let (response, _) = TweetApi::new(body).execute(&self.auth).await?;
    let id = match response.data {
      Some(data) => data.id.unwrap_or("".to_string()),
      None => "".to_string(),
    };

    Ok(id)
  }
}

// Media
impl Api {
  pub async fn upload_media(
    &self,
    mastodon_media: MastodonMedia,
    description: &Option<String>,
  ) -> Result<String, Box<dyn Error>> {
    // Init
    let data = MediaInitData {
      total_bytes: mastodon_media.content_size,
      media_type: mastodon_media.content_type,
      ..Default::default()
    };
    let (response, _) = MediaInitApi::new(data).execute(&self.auth).await?;
    let media_id = response.media_id_string;

    // Append
    const MEDIA_SPLIT_SIZE: u64 = 1000000;
    let stream = mastodon_media.response.bytes_stream();
    let mut reader = StreamReader::new(stream.map_err(|e| IoError::new(IoErrorKind::Other, e)));
    let mut segment_index = 0;

    while segment_index * MEDIA_SPLIT_SIZE < mastodon_media.content_size {
      let read_size = match (segment_index + 1) * MEDIA_SPLIT_SIZE < mastodon_media.content_size {
        true => MEDIA_SPLIT_SIZE,
        false => mastodon_media.content_size - segment_index * MEDIA_SPLIT_SIZE,
      };
      let mut cursor = Cursor::new(vec![0; read_size as usize]);

      reader.read_exact(cursor.get_mut()).await?;
      let data = MediaAppendData {
        media_id: media_id.to_owned(),
        segment_index,
        cursor,
      };
      let _ = MediaAppendApi::new(data).execute(&self.auth).await?;

      segment_index += 1;
    }

    // Finalize
    let data = MediaFinalizeData {
      media_id: media_id.to_owned(),
    };
    let _ = MediaFinalizeApi::new(data).execute(&self.auth).await?;

    // Add description.
    if let Some(text) = description {
      let body = MediaMetadataBody {
        media_id: media_id.to_owned(),
        alt_text: MediaMetadataAltText {
          text: text.to_owned(),
        },
      };
      let _ = MediaMetadataApi::new(body).execute(&self.auth).await?;
    }

    Ok(media_id)
  }
}
