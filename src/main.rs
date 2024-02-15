use chrono::TimeZone;
use clap::{Parser, Subcommand};
use std::{ffi::OsStr, fs, io::Read, path::PathBuf};
use zip::ZipArchive;
use serde::{Serialize, Deserialize};
use chrono::prelude::*;

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
                let file = fs::File::open(path).unwrap_or_else(|e| panic!("could not open {path:?}: {e}"));
                let bufreader = std::io::BufReader::new(file);
                let mut archive = ZipArchive::new(bufreader).unwrap();
                let mut records_file = archive.by_name("Takeout/Location History (Timeline)/Records.json").unwrap_or_else(|e| panic!("could not extract data from Records.json: {e}"));
                let mut buf: Vec<u8> = Vec::new();
                records_file.read_to_end(&mut buf).unwrap_or_else(|e| panic!("could not read Records.json: {e}"));
                std::str::from_utf8(&buf).unwrap_or_else(|e| panic!("could not read data from Records.json as utf-8 string: {e}")).to_string()
            } else if path.extension() == Some(OsStr::new("json")) {
                // read file to string
                std::fs::read_to_string(path).unwrap_or_else(|e| panic!("could not read file {path:?}: {e}"))

            } else {
                let ext = path.extension().unwrap().to_str().unwrap();
                panic!(
                    "could not handle unknown filetype, must be one of {{.zip, .json}}: {ext}"
                );
            };

            let document: JsonDocument = serde_json::from_str(&json_str).unwrap_or_else(|e| panic!("could not deserialize json: {e}"));
        }
        None => {}
    }
}


#[derive(Deserialize)]
struct JsonDocument {
    locations: Vec<JsonRecord>
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

struct Record {
    latitude: i64,
    longitude: i64,
    timestamp: DateTime<Utc>
}