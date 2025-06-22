use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize, de::Visitor};

use crate::utils::{extract_youtube_channel_id, extract_youtube_video_id};

/// Video ID deserializer
///
/// Extract video ID from URL
fn video_id_de<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(extract_youtube_video_id(s))
}

/// Video title deserializer
///
/// By default title prefixed with "Watched " keyword, this function will get rid of that prefix
fn video_title_de<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;

    if let Some(val) = s.strip_prefix("Watched ") {
        Ok(val.to_owned())
    } else {
        Ok(s)
    }
}

/// Channel deserializer
///
/// Extract channel object from array sequences
fn channel_de<'de, D>(deserializer: D) -> Result<Channel, D::Error>
where
    D: Deserializer<'de>,
{
    struct FirstVisitor;

    impl<'de> Visitor<'de> for FirstVisitor {
        type Value = Channel;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a nonempty sequence of objects")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::SeqAccess<'de>,
        {
            let mut is_first = false;
            let mut value = Channel::default();

            while let Some(val) = seq.next_element::<Channel>()? {
                if !is_first {
                    value = val;
                    is_first = true;
                }
            }

            Ok(value)
        }
    }

    deserializer.deserialize_seq(FirstVisitor)
}

/// Channel ID deserializer
///
/// Extract channel ID from URL
fn channel_id_de<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(extract_youtube_channel_id(s))
}

/// Channel
#[derive(Deserialize, Serialize, Debug)]
pub struct Channel {
    #[serde(
        rename(deserialize = "url"),
        default,
        deserialize_with = "channel_id_de"
    )]
    pub id: String,

    #[serde(default)]
    pub name: String,
}

impl Default for Channel {
    fn default() -> Self {
        Channel {
            id: "-".to_owned(),
            name: "-".to_owned(),
        }
    }
}

/// Schema version 1 based on the JSON structures
#[derive(Deserialize, Serialize, Debug)]
pub struct Schema {
    #[serde(
        rename(deserialize = "titleUrl"),
        default,
        deserialize_with = "video_id_de"
    )]
    pub id: String,

    #[serde(default, deserialize_with = "video_title_de")]
    pub title: String,

    pub time: DateTime<Utc>,

    #[serde(
        rename(deserialize = "subtitles"),
        default,
        deserialize_with = "channel_de"
    )]
    pub channel: Channel,
}
