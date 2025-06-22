use std::{collections::HashSet, ffi::OsStr, io::BufRead, path::PathBuf};

use chrono::{DateTime, Local, TimeZone};
use once_cell::sync::Lazy;
use regex::Regex;

/// Simple checking json file
pub fn is_json_file(path: &PathBuf) -> bool {
    let is_json = match path.extension().and_then(OsStr::to_str) {
        Some("json") => true,
        _ => false,
    };

    if path.is_file() && is_json {
        true
    } else {
        false
    }
}

/// Check keywords through buffer
pub fn is_buffer_contains_keywords<R: BufRead>(reader: R, keys: &HashSet<&str>) -> bool {
    let mut found_keys = HashSet::new();

    for line in reader.lines() {
        if let Ok(line) = line {
            for key in keys {
                if !found_keys.contains(key) && line.contains(key) {
                    found_keys.insert(*key);
                }
            }

            if found_keys.len() == keys.len() {
                return true;
            }
        }
    }

    false
}

/// Extract youtube video id from url
pub fn extract_youtube_video_id(haystack: String) -> String {
    static RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"(?:https?://(?:www\.|music\.)?youtube\.com/(?:channel|c|user)/|https?://(?:www\.|music\.)?youtube\.com/watch\?v=|https?://youtu\.be/)([a-zA-Z0-9_-]{11})").unwrap()
    });

    if let Some(val) = RE.captures(&haystack) {
        val[1].to_string()
    } else {
        haystack
    }
}

/// Extract youtube channel id from url
pub fn extract_youtube_channel_id(haystack: String) -> String {
    static RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"(?:https?://(?:www\.)?youtube\.com/(?:channel|c|user)/)([a-zA-Z0-9_-]+)")
            .unwrap()
    });

    if let Some(val) = RE.captures(&haystack) {
        val[1].to_string()
    } else {
        haystack
    }
}

/// DateTimeUtility
///
/// DateTime utility from chrono
pub trait DateTimeUtility {
    fn to_datetime_string(&self) -> String;
}

impl<Tz: TimeZone> DateTimeUtility for DateTime<Tz> {
    fn to_datetime_string(&self) -> String {
        self.with_timezone(&Local)
            .format("%Y-%m-%d %H:%M:%S")
            .to_string()
    }
}

/// Fetch some url
pub async fn fetch_url(url: &str) -> anyhow::Result<bytes::Bytes> {
    log::debug!("Fetch: {}", url);

    let res = reqwest::get(url).await?;

    if !res.status().is_success() {
        anyhow::bail!("request failed: {}", res.status());
    }

    let bytes = res.bytes().await?;

    Ok(bytes)
}
