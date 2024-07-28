use serde::Deserialize;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::Manager;
use url::Url;

use crate::{
    error::{Error, Result},
    Storage,
};

#[derive(Debug, Deserialize, Clone, Copy)]
pub enum Direction {
    Flat,
    FortyFiveUp,
    FortyFiveDown,
    SingleUp,
    SingleDown,
    DoubleUp,
    DoubleDown,
    TripleUp,
    TripleDown,
    #[serde(rename = "RATE OUT OF RANGE")]
    RateOutOfRange,
    #[serde(rename = "NOT COMPUTABLE")]
    NotComputable,
    #[serde(rename = "NONE")]
    None,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct NightscoutEntry {
    #[serde(rename = "_id")]
    _id: String,
    device: String,
    date: u128,
    date_string: String,
    sgv: f32,
    delta: f32,
    direction: Direction,
    r#type: String,
    filtered: u32,
    unfiltered: u32,
    rssi: u32,
    noise: u32,
    sys_time: String,
    utc_offset: i32,
    mills: u128,
}

pub fn get_glucose_data(app: tauri::AppHandle) -> Result<(String, Direction)> {
    let reqwest = reqwest::blocking::Client::new();
    let glucmon_config_store = &app.state::<Storage>().config;
    let glucmon_config = glucmon_config_store.lock().map_err(Error::custom)?;
    let nightscout_url = Url::parse(&glucmon_config.nightscout_url)?.join("/api/v1/entries")?;
    let nightscout_api_token = &glucmon_config.nightscout_api_token;
    let is_mmmol = glucmon_config.is_mmmol;

    let response = reqwest
        .get(nightscout_url)
        .header("accept", "application/json")
        .header("api-secret", nightscout_api_token)
        .query(&[("count", "1")])
        .send()?;

    let mut data = response.json::<Vec<NightscoutEntry>>()?;

    data.sort_by(|a, b| b.date.cmp(&a.date));

    let last_entry = data
        .first()
        .ok_or(Error::custom("Could not extract data.first()."))?;
    let divider = if is_mmmol { 18.0 } else { 1.0 };
    let glucose_value = last_entry.sgv / divider;
    let direction = last_entry.direction;
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");

    let mins_ago = (since_the_epoch.as_millis() - last_entry.date) / 60000;

    let str = format!("{glucose_value:.1} - {mins_ago} mins ago.");

    Ok((str, direction))
}
