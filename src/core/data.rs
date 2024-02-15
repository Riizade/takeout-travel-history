// this file contains type/data definitions for internal use

use std::fmt::Display;
use std::{fmt, str::FromStr};

use chrono::{DateTime, Utc};
use clap::ValueEnum;
use serde::Deserialize;

use crate::{JsonRecord, JsonSource};

/// this is a cleaner, more usable version of the raw JSON JsonRecord type from Google Takeout (in json.rs)
#[derive(Clone, Copy, Debug)]
pub struct Record {
    pub latitude: f64,
    pub longitude: f64,
    pub timestamp: DateTime<Utc>,
    pub source: Source,
}

impl Record {
    pub fn from_json(json: &JsonRecord) -> Option<Self> {
        if let (Some(latitude), Some(longitude), timestamp) =
            (json.latitude, json.longitude, &json.timestamp)
        {
            Some(Record {
                latitude: latitude as f64 / 1E7,
                longitude: longitude as f64 / 1E7,
                timestamp: DateTime::from_str(&timestamp).unwrap(),
                source: Source::from_json_source(&json.source),
            })
        } else {
            None
        }
    }
}

/// defines the source for a location record
#[derive(Deserialize, PartialEq, Eq, Hash, Copy, Clone, ValueEnum, Debug)]
pub enum Source {
    /// a wifi access point
    WIFI,
    /// gps satellite signal
    GPS,
    /// cell tower signal
    CELL,
    /// unknown source (as recorded in Google Takeout)
    UNKNOWN,
    /// no source was recorded in Google Takeout
    NONE,
}

impl Source {
    pub fn from_json_source(json: &Option<JsonSource>) -> Self {
        match json {
            Some(s) => match s {
                JsonSource::CELL => Source::CELL,
                JsonSource::UNKNOWN => Source::UNKNOWN,
                JsonSource::GPS => Source::GPS,
                JsonSource::WIFI => Source::WIFI,
            },
            None => Source::NONE,
        }
    }
}

pub enum Region {
    CountryCode(rust_iso3166::CountryCode),
    Subdivision(rust_iso3166::iso3166_2::Subdivision),
    Obsolete(rust_iso3166::iso3166_3::CountryCode3),
    UnknownCode(String),
    MissingData,
}

impl Region {
    pub fn from_code(code: &str) -> Self {
        // decode all versions
        let opt_cc = rust_iso3166::from_alpha2(code);
        let opt_sub = rust_iso3166::iso3166_2::from_code(code);
        let opt_obs = rust_iso3166::iso3166_3::from_code(code);

        if let Some(cc) = opt_cc {
            Self::CountryCode(cc)
        } else if let Some(sub) = opt_sub {
            Self::Subdivision(sub)
        } else if let Some(obs) = opt_obs {
            Self::Obsolete(obs)
        } else {
            Self::UnknownCode(code.to_owned())
        }
    }
}

impl Display for Region {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = match self {
            Region::CountryCode(c) => c.name,
            Region::Subdivision(s) => s.name,
            Region::Obsolete(o) => o.name,
            Region::UnknownCode(u) => u,
            Region::MissingData => "Missing Data",
        };
        write!(f, "{str}")
    }
}

pub struct BorderCrossing {
    pub timestamp: DateTime<Utc>,
    pub new_regions: Vec<Region>,
}

impl Display for BorderCrossing {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let timestamp_str = self.timestamp.to_rfc2822();
        let region_strings: String = self
            .new_regions
            .iter()
            .map(|r| format!("    | {r}"))
            .collect::<Vec<String>>()
            .join("\n");
        let complete_string = format!("{timestamp_str}\n    |\n{region_strings}\n    |");
        write!(f, "{complete_string}")
    }
}
