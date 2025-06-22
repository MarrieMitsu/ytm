use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::{BufRead, BufReader, Seek, SeekFrom},
    path::PathBuf,
};

use anyhow::{Result, bail};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;

use crate::utils::{DateTimeUtility, is_buffer_contains_keywords, is_json_file};

mod v1;

/// default_page
pub fn default_page() -> usize {
    1
}

/// default_limit
pub fn default_limit() -> usize {
    20
}

/// default_order
pub fn default_order() -> Order {
    Order::Latest
}

/// Pagination
#[derive(Debug, Serialize)]
pub struct Pagination {
    pub current_page: usize,
    pub prev_page: Option<usize>,
    pub next_page: Option<usize>,
    pub total_page: usize,
    pub limit: usize,
    pub page_range: Vec<usize>,
}

impl Pagination {
    pub fn new(current_page: usize, total_page: usize, limit: usize) -> Self {
        let prev_page = if current_page > 1 {
            Some(current_page - 1)
        } else {
            None
        };

        let next_page = if current_page < total_page {
            Some(current_page + 1)
        } else {
            None
        };

        let min = 1;
        let max = total_page;

        let (start, end) = if max <= 5 {
            (min, max)
        } else {
            let mut s = current_page.saturating_sub(2).max(min);
            let mut e = (current_page + 2).max(5);

            if e > max {
                let diff = e - max;
                s = s.saturating_sub(diff).max(min);
                e = max;
            }

            (s, e)
        };

        let page_range = (start..=end).collect();

        Self {
            current_page,
            prev_page,
            next_page,
            total_page,
            limit,
            page_range,
        }
    }
}

/// Order
#[derive(Debug, Serialize, Deserialize, Clone, strum::Display, strum::EnumIter, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum Order {
    Latest,
    Oldest,
    MostWatched,
    LeastWatched,
}

impl Order {
    pub fn to_string_label(&self) -> String {
        match self {
            Self::Latest => String::from("Latest"),
            Self::Oldest => String::from("Oldest"),
            Self::MostWatched => String::from("Most Watched"),
            Self::LeastWatched => String::from("Least Watched"),
        }
    }

    pub fn collect_key_label_pair() -> Vec<(String, String)> {
        Self::iter()
            .map(|v| (v.to_string(), v.to_string_label()))
            .collect()
    }
}

/// MetadataFilter
#[derive(Debug, Serialize, Deserialize)]
pub struct MetadataFilter {
    pub id: Option<String>,
    pub title: Option<String>,
    pub channel_name: Option<String>,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,

    #[serde(default = "default_order")]
    pub order: Order,

    #[serde(default = "default_page")]
    pub page: usize,

    #[serde(default = "default_limit")]
    pub limit: usize,
}

impl MetadataFilter {
    /// Check if all fields are `None` to pass filtering
    pub fn skip(&self) -> bool {
        self.id.is_none()
            && self.title.is_none()
            && self.channel_name.is_none()
            && self.from.is_none()
            && self.to.is_none()
    }
}

/// MetadataTable
#[derive(Debug)]
pub struct MetadataTable {
    total_count_raw: usize,
    total_count: usize,
    watch_timeline: Vec<DateTime<Utc>>,
    data: Vec<Metadata>,
}

impl MetadataTable {
    pub fn total_count_raw(&self) -> usize {
        self.total_count_raw
    }

    pub fn total_count(&self) -> usize {
        self.total_count
    }

    pub fn watch_timeline(&self) -> Vec<DateTime<Utc>> {
        self.watch_timeline.clone()
    }

    pub fn get_collection(&mut self, filter: &MetadataFilter) -> (Pagination, Vec<Metadata>) {
        let mut filtered = self
            .data
            .iter()
            .filter(|x| {
                if filter.skip() {
                    return true;
                }

                let id = if let Some(v) = &filter.id {
                    x.id == *v
                } else {
                    true
                };

                let title = if let Some(v) = &filter.title {
                    x.title.to_lowercase().contains(&v.to_lowercase())
                } else {
                    true
                };

                let channel_name = if let Some(v) = &filter.channel_name {
                    x.channel.name.to_lowercase().contains(&v.to_lowercase())
                } else {
                    true
                };

                let from = if let Some(v) = &filter.from {
                    x.watched_at > *v
                } else {
                    true
                };

                let to = if let Some(v) = &filter.to {
                    x.watched_at < *v
                } else {
                    true
                };

                id && title && channel_name && from && to
            })
            .cloned()
            .collect::<Vec<Metadata>>();

        match filter.order {
            Order::Oldest => {
                filtered.reverse();
            }
            Order::MostWatched => {
                filtered.sort_by_key(|v| v.watch_count);
                filtered.reverse();
            }
            Order::LeastWatched => {
                filtered.sort_by_key(|v| v.watch_count);
            }
            _ => {}
        }

        let total_item = filtered.len();
        let total_page = (total_item as f64 / filter.limit as f64).ceil() as usize;

        let page_offset = filter.page * filter.limit;
        let limit_offset = filter.limit;

        let right = page_offset.min(total_item);

        let left = if page_offset < total_item {
            page_offset.saturating_sub(limit_offset)
        } else {
            (total_item - (total_item % limit_offset)).max(0)
        };

        filtered.drain(right..);
        filtered.drain(..left);

        (
            Pagination::new(filter.page, total_page, filter.limit),
            filtered,
        )
    }
}

/// Channel
#[derive(Clone, Debug, Serialize)]
pub struct Channel {
    pub id: String,
    pub name: String,
}

/// Metadata
#[derive(Clone, Debug, Serialize)]
pub struct Metadata {
    pub id: String,
    pub title: String,
    pub channel: Channel,
    pub watched_at: DateTime<Utc>,
    pub watch_count: usize,
    pub watch_timeline: Vec<DateTime<Utc>>,
}

impl Metadata {
    pub fn to_datetime_local(&self) -> String {
        self.watched_at.to_datetime_string()
    }
}

impl PartialOrd for Metadata {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Metadata {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.watched_at.cmp(&other.watched_at)
    }
}

impl PartialEq for Metadata {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Metadata {}

/// Is version 1 json structure
fn is_v1<R: BufRead>(reader: R) -> bool {
    let keys: HashSet<&str> = [
        "\"header\"",
        "\"title\"",
        "\"titleUrl\"",
        "\"subtitles\"",
        "\"name\"",
        "\"url\"",
        "\"time\"",
        "\"products\"",
    ]
    .into();

    is_buffer_contains_keywords(reader, &keys)
}

/// Load version 1 schema
fn load_v1<R: BufRead>(reader: R) -> Result<MetadataTable> {
    log::debug!("Match schema version: 1");

    let raw: Vec<v1::Schema> = serde_json::from_reader(reader)?;
    let mut total_count_raw: usize = 0;
    let mut watch_timeline: Vec<DateTime<Utc>> = Vec::new();
    let mut map: HashMap<String, Metadata> = HashMap::new();

    for r in raw {
        total_count_raw += 1;
        watch_timeline.push(r.time);

        if map.contains_key(&r.id) {
            if let Some(m) = map.get_mut(&r.id) {
                // watched_at always the earliest
                if r.time < m.watched_at {
                    m.watched_at = r.time;
                }

                m.watch_count += 1;
                m.watch_timeline.push(r.time);
                m.watch_timeline.sort();
            }
        } else {
            let m = Metadata {
                id: r.id.clone(),
                title: r.title,
                channel: Channel {
                    id: r.channel.id,
                    name: r.channel.name,
                },
                watched_at: r.time,
                watch_count: 1,
                watch_timeline: vec![r.time],
            };

            map.insert(r.id.clone(), m);
        }
    }

    let mut data = map.into_values().collect::<Vec<Metadata>>();
    data.sort_by(|a, b| b.watched_at.cmp(&a.watched_at));
    watch_timeline.sort();

    Ok(MetadataTable {
        total_count_raw,
        total_count: data.len(),
        watch_timeline,
        data,
    })
}

/// Load metadata from a json file
pub fn load_metadata_from_file(path: &PathBuf) -> Result<MetadataTable> {
    log::debug!("Loading metadata from file...");

    if !is_json_file(path) {
        bail!("Unsupported file format. Please use valid JSON file");
    }

    let file = File::open(path)?;
    let mut rdr = BufReader::new(file);

    if is_v1(&mut rdr) {
        let _ = rdr.seek(SeekFrom::Start(0))?;
        let metadata_table = load_v1(rdr)?;

        Ok(metadata_table)
    } else {
        bail!("Unrecognized JSON structure. The JSON structure does not match any defined schema");
    }
}
