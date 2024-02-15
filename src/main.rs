mod core;

use crate::core::data::*;
use crate::core::json::*;
use chrono::prelude::*;
use chrono::TimeDelta;
use clap::{Parser, Subcommand};
use country_boundaries::LatLon;
use country_boundaries::{CountryBoundaries, BOUNDARIES_ODBL_360X180};
use std::collections::btree_map::Range;
use std::collections::HashSet;
use std::{ffi::OsStr, fs, io::Read, path::PathBuf};
use zip::ZipArchive;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// lists every time the location crosses a recognized border
    BorderCrossings {
        #[arg(
            short('p'),
            long,
            required(true),
            help("The .zip or .json file that will be read to produce the command's output")
        )]
        path: PathBuf,
        #[arg(short('e'), long, required(false), value_name("SOURCE"), help("Excludes a certain data source from the results; can be specified multiple times to exclude multiple sources"))]
        exclude_source: Vec<Source>,
        #[arg(short('s'), long, required(false), help("Ignores border crossings between subregions such as US states, Canadian provinces, etc"))]
        ignore_subregions: bool,
        #[arg(short('m'), long, required(false), help("Does not treat missing data as its own region and instead assumes that the region remains the same for the duration of missing data"))]
        ignore_missing_data: bool,
    },
}

fn main() {
    run_cli();
}

fn run_cli() {
    let cli = Cli::parse();
    match &cli.command {
        Some(Commands::BorderCrossings {
            path,
            exclude_source,
            ignore_subregions,
            ignore_missing_data,
        }) => {
            // read file to Vec<Record>
            let mut records: Vec<Record> = read_records_from_file(path);
            // sort records by timestamp in ascending order (should already be sorted, but just in case)
            records.sort_unstable_by_key(|r| r.timestamp);
            // convert Record to BorderCrossing
            let crossings = records_to_border_crossings(&records);
            let s = display_border_crossings(&crossings);
            println!("{s}");
        }
        None => {}
    }
}

fn read_records_from_file(path: &PathBuf) -> Vec<Record> {
    // extract json string from Records.json
    let json_str = if path.extension() == Some(OsStr::new("zip")) {
        // if .zip
        // extract the data of Records.json from within the zip as a &str
        let file = fs::File::open(path).unwrap_or_else(|e| panic!("could not open {path:?}: {e}"));
        let bufreader = std::io::BufReader::new(file);
        let mut archive = ZipArchive::new(bufreader).unwrap();
        // find Records.json within the zip archive
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
        // if .json
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

    // convert to Vec<Record>
    document
        .locations
        .iter()
        .flat_map(|r| Record::from_json(r))
        .collect()
}

fn border_crossing_to_string(
    crossing: &BorderCrossing,
    next_crossing: &Option<&BorderCrossing>,
) -> String {
    let timestamp_str = crossing.timestamp.to_rfc2822();
    let region_strings: String = crossing
        .new_regions
        .iter()
        .map(|r| format!("    | {r}"))
        .collect::<Vec<String>>()
        .join("\n");
    let duration_string = match next_crossing {
        Some(next) => {
            let days = (next.timestamp - crossing.timestamp).num_days();
            format!("    | Duration: {days} Days")
        }
        None => "    | Duration Unknown".to_string(),
    };
    let complete_string = vec![
        &timestamp_str,
        "    |",
        &region_strings,
        &duration_string,
        "    |\n",
    ]
    .join("\n");
    complete_string
}

fn display_border_crossings(crossings: &Vec<BorderCrossing>) -> String {
    let mut string: String = "".to_string();
    for i in 0..crossings.len() {
        let crossing = crossings.get(i).unwrap();
        let maybe_next = crossings.get(i + 1);
        string += &border_crossing_to_string(crossing, &maybe_next);
    }

    string
}

/// requires records to be sorted by timestamp
fn records_to_border_crossings(records: &Vec<Record>) -> Vec<BorderCrossing> {
    // create a vector to track border crossings
    let mut crossings: Vec<BorderCrossing> = vec![];
    let mut maybe_prev: Option<Record> = None;
    for record in records.iter() {
        if let Some(prev) = maybe_prev {
            // if we have a previous record, check before adding a new crossing
            // check if we have a data gap of more than one day
            let interval = record.timestamp - prev.timestamp;
            if interval >= TimeDelta::days(1) {
                // if we have a gap of more than one day, add a missing data border crossing
                crossings.push(BorderCrossing {
                    timestamp: prev.timestamp + TimeDelta::days(1), // timestamp is +1 day from previous record
                    new_regions: vec![Region::MissingData].into_iter().collect(),
                })
            }

            // add crossing if we've changed locations (or if the last crossing was MissingData)
            let location_diff = &record.regions() - &prev.regions();
            if location_diff.len() > 0
                || crossings
                    .last()
                    .is_some_and(|c| c.new_regions.contains(&Region::MissingData))
            {
                crossings.push(BorderCrossing::from(record))
            }
        } else {
            // if there is no previous record, we unconditionally make a border crossing
            crossings.push(BorderCrossing::from(record))
        }

        // update previous record
        maybe_prev = Some(*record);
    }
    crossings
}
