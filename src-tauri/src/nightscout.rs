use serde::Deserialize;
use std::time::{SystemTime, UNIX_EPOCH};

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
    direction: String,
    r#type: String,
    filtered: u32,
    unfiltered: u32,
    rssi: u32,
    noise: u32,
    sys_time: String,
    utc_offset: i32,
    mills: u128,
}

pub fn get_glucose_data() -> anyhow::Result<String> {
    let reqwest = reqwest::blocking::Client::new();
    let nightscout_url = std::env::var("NIGHTSCOUT_URL").expect("NIGHTSCOUT_URL must be set.");
    let nightscout_api_token =
        std::env::var("NIGHTSCOUT_API_TOKEN").expect("NIGHTSCOUT_URL must be set.");
    let is_mmmol = std::env::var("IS_MMMOL").unwrap_or("true".to_string());

    let mut data = reqwest
        .get(format!("{nightscout_url}/api/v1/entries"))
        .header("accept", "application/json")
        .header("api-secret", nightscout_api_token)
        .send()?
        .json::<Vec<NightscoutEntry>>()?;

    data.sort_by(|a, b| b.date.cmp(&a.date));

    let last_entry = data.first().unwrap();
    let divider = if is_mmmol == *"true" { 18.0 } else { 1.0 };
    let glucose_value = last_entry.sgv / divider;
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");

    let mins_ago = (since_the_epoch.as_millis() - last_entry.date) / 60000;

    let str = format!("{glucose_value:.1} - {mins_ago} mins ago.");
    Ok(str)
}
