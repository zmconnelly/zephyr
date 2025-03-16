#![allow(unused)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bang {
    pub id: String,
    pub name: String,
    pub search_url: String,
    pub home_url: String,
    pub category: String,
    pub is_custom: bool,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct BangCache {
    pub bangs: HashMap<String, Bang>,
    #[serde(with = "timestamp_seconds")]
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct DuckDuckGoBang {
    #[serde(rename = "c", default)]
    pub category: Option<String>,

    #[serde(rename = "d", default)]
    pub domain: Option<String>,

    #[serde(rename = "s", default)]
    pub name: Option<String>,

    #[serde(rename = "sc", default)]
    pub subcategory: Option<String>,

    #[serde(rename = "t", default)]
    pub trigger: Option<String>,

    #[serde(rename = "u", default)]
    pub url: Option<String>,

    // Optional fields that we don't use but are present in the data
    #[serde(rename = "r", default)]
    pub rank: Option<i32>,
}

impl DuckDuckGoBang {
    pub fn is_valid(&self) -> bool {
        self.category.is_some()
            && self.domain.is_some()
            && self.name.is_some()
            && self.subcategory.is_some()
            && self.trigger.is_some()
            && self.url.is_some()
    }

    pub fn to_bang(&self) -> Option<(String, Bang)> {
        if !self.is_valid() {
            return None;
        }

        let trigger = self.trigger.as_ref().unwrap().clone();

        let bang = Bang {
            id: trigger.clone(),
            name: self.name.as_ref().unwrap().clone(),
            search_url: self.url.as_ref().unwrap().clone(),
            home_url: format!("https://{}", self.domain.as_ref().unwrap()),
            category: format!(
                "{} - {}",
                self.category.as_ref().unwrap(),
                self.subcategory.as_ref().unwrap()
            ),
            is_custom: false,
        };

        Some((trigger, bang))
    }
}

/// Serialization helper for DateTime<Utc>
pub(crate) mod timestamp_seconds {
    use super::*;

    pub fn serialize<S>(dt: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let timestamp = dt.timestamp();
        serializer.serialize_i64(timestamp)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let timestamp = i64::deserialize(deserializer)?;
        Ok(DateTime::<Utc>::from_timestamp(timestamp, 0).unwrap_or_else(|| Utc::now()))
    }
}
