use chrono::prelude::*;
use chrono::TimeDelta;
use chrono::TimeZone;
use clap::{Parser, Subcommand};
use country_boundaries::LatLon;
use country_boundaries::{CountryBoundaries, BOUNDARIES_ODBL_360X180};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt;
use std::fmt::Display;
use std::{ffi::OsStr, fs, io::Read, path::PathBuf, str::FromStr};
use zip::ZipArchive;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    List {
        #[arg(short, long)]
        path: PathBuf,
    },
}

fn main() {
    run_cli();
}

fn run_cli() {
    let cli = Cli::parse();
    match &cli.command {
        Some(Commands::List { path }) => {
            let json_str = if path.extension() == Some(OsStr::new("zip")) {
                // extract the data of Records.json from within the zip as a &str
                let file =
                    fs::File::open(path).unwrap_or_else(|e| panic!("could not open {path:?}: {e}"));
                let bufreader = std::io::BufReader::new(file);
                let mut archive = ZipArchive::new(bufreader).unwrap();
                let mut records_file = archive
                    .by_name("Takeout/Location History (Timeline)/Records.json")
                    .unwrap_or_else(|e| panic!("could not extract data from Records.json: {e}"));
                let mut buf: Vec<u8> = Vec::new();
                records_file
                    .read_to_end(&mut buf)
                    .unwrap_or_else(|e| panic!("could not read Records.json: {e}"));
                std::str::from_utf8(&buf)
                    .unwrap_or_else(|e| {
                        panic!("could not read data from Records.json as utf-8 string: {e}")
                    })
                    .to_string()
            } else if path.extension() == Some(OsStr::new("json")) {
                // read file to string
                std::fs::read_to_string(path)
                    .unwrap_or_else(|e| panic!("could not read file {path:?}: {e}"))
            } else {
                let ext = path.extension().unwrap().to_str().unwrap();
                panic!("could not handle unknown filetype, must be one of {{.zip, .json}}: {ext}");
            };
            // deserialize the document to rust struct
            let document: JsonDocument = serde_json::from_str(&json_str)
                .unwrap_or_else(|e| panic!("could not deserialize json: {e}"));
            // convert to workable data types
            let mut records: Vec<Record> = document
                .locations
                .iter()
                .flat_map(|r| Record::from_json(r))
                .collect();
            // sort records by timestamp in ascending order (should already be sorted, but just in case)
            records.sort_unstable_by_key(|r| r.timestamp);
            // get country boundaries
            let boundaries = CountryBoundaries::from_reader(BOUNDARIES_ODBL_360X180)
                .unwrap_or_else(|e| panic!("could not read boundaries: {e}"));

            let mut maybe_prev: Option<Record> = None;
            for record in records.iter() {
                // find the time interval between this record and the previous record
                let maybe_interval = maybe_prev.as_ref().map(|p| record.timestamp - p.timestamp);

                // print a line if we have a gap in data >= 1 day
                if let Some(interval) = maybe_interval {
                    if interval >= TimeDelta::days(1) {
                        let gap_days = interval.num_days();
                        println!("data gap of {gap_days} days")
                    }
                }

                if let Some(prev) = maybe_prev {
                    let ids: HashSet<&str> = boundaries
                        .ids(LatLon::new(record.latitude, record.longitude).unwrap())
                        .iter()
                        .map(|s| *s)
                        .collect();

                    let prev_ids: HashSet<&str> = boundaries
                        .ids(LatLon::new(prev.latitude, prev.longitude).unwrap())
                        .iter()
                        .map(|s| *s)
                        .collect();

                    let diff = &ids - &prev_ids;
                    if diff.len() > 0 {
                        let time_str = record.timestamp.to_rfc2822();
                        let zones: Vec<Region> =
                            diff.iter().map(|code| Region::from_code(code)).collect();
                        let zones_str = zones
                            .iter()
                            .map(|z| format!("    {z}"))
                            .collect::<Vec<String>>()
                            .join("\n");
                        println!("time: {time_str}");
                        println!("{zones_str}");
                    }
                }

                // update the previous record
                maybe_prev = Some(record.to_owned());
            }
        }
        None => {}
    }
}

#[derive(Deserialize)]
struct JsonDocument {
    locations: Vec<JsonRecord>,
}

#[derive(Deserialize)]
struct JsonRecord {
    #[serde(rename(deserialize = "latitudeE7"))]
    latitude: Option<i64>,
    #[serde(rename(deserialize = "longitudeE7"))]
    longitude: Option<i64>,
    accuracy: Option<i64>,
    #[serde(rename(deserialize = "verticalAccuracy"))]
    vertical_accuracy: Option<i64>,
    source: Option<Source>,
    timestamp: String,
}

#[derive(Deserialize)]
enum Source {
    WIFI,
    UNKNOWN,
    GPS,
    CELL,
}

#[derive(Clone, Copy, Debug)]
struct Record {
    latitude: f64,
    longitude: f64,
    timestamp: DateTime<Utc>,
}

impl Record {
    fn from_json(json: &JsonRecord) -> Option<Self> {
        if let (Some(latitude), Some(longitude), timestamp) =
            (json.latitude, json.longitude, &json.timestamp)
        {
            Some(Record {
                latitude: latitude as f64 / 1E7,
                longitude: longitude as f64 / 1E7,
                timestamp: DateTime::from_str(&timestamp).unwrap(),
            })
        } else {
            None
        }
    }
}

enum Region {
    CountryCode(rust_iso3166::CountryCode),
    Subdivision(rust_iso3166::iso3166_2::Subdivision),
    Obsolete(rust_iso3166::iso3166_3::CountryCode3),
    Unknown(String),
}

impl Region {
    fn from_code(code: &str) -> Self {
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
            Self::Unknown(code.to_owned())
        }
    }
}

impl Display for Region {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = match self {
            Region::CountryCode(c) => c.name,
            Region::Subdivision(s) => s.name,
            Region::Obsolete(o) => o.name,
            Region::Unknown(u) => u,
        };
        write!(f, "{str}")
    }
}
