// this file contains definitions for the JSON types encountered in the Google Takeout data

use serde::Deserialize;

#[derive(Deserialize)]
pub struct JsonDocument {
    pub locations: Vec<JsonRecord>,
}

#[derive(Deserialize)]
pub struct JsonRecord {
    #[serde(rename(deserialize = "latitudeE7"))]
    pub latitude: Option<i64>,
    #[serde(rename(deserialize = "longitudeE7"))]
    pub longitude: Option<i64>,
    pub accuracy: Option<i64>,
    #[serde(rename(deserialize = "verticalAccuracy"))]
    pub vertical_accuracy: Option<i64>,
    pub source: Option<JsonSource>,
    pub timestamp: String,
}

#[derive(Deserialize, PartialEq, Eq, Hash, Copy, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum JsonSource {
    Wifi,
    Unknown,
    #[serde(rename = "GPS")]
    GPS,
    Cell,
    VisitDeparture,
    VisitArrival,
    Manual,
}
