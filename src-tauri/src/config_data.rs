use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::fs::{create_dir_all, OpenOptions};
use std::io::Read;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GlucmonConfigStore {
    pub nightscout_url: String,
    pub nightscout_api_token: String,
    pub is_mmmol: bool,
    #[serde(skip_serializing, default = "GlucmonConfigStore::set_to_true")]
    pub is_set: bool,
}

impl Default for GlucmonConfigStore {
    fn default() -> Self {
        Self::new()
    }
}

impl GlucmonConfigStore {
    fn new() -> Self {
        Self {
            nightscout_url: "".to_string(),
            nightscout_api_token: "".to_string(),
            is_mmmol: true,
            is_set: false,
        }
    }

    pub fn initialize(&mut self, app: tauri::AppHandle) -> Result<()> {
        let data_dir = app
            .path_resolver()
            .app_data_dir()
            .ok_or(Error::custom("Could not resolve app data dir."))?;
        create_dir_all(&data_dir)?;

        let data_file = data_dir.join("data.json");

        dbg!("Reading/writing to json file: {}", &data_file);

        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(data_file)?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let json = match serde_json::from_str(&contents) {
            Ok(json) => json,
            Err(err) => {
                dbg!("Failed to load json, creating from scratch. Error: {}", err);
                Self::new()
            }
        };

        *self = json;

        Ok(())
    }

    pub fn update_config(
        &mut self,
        nightscout_url: String,
        nightscout_api_token: String,
        is_mmol: bool,
    ) {
        self.nightscout_url = nightscout_url;
        self.nightscout_api_token = nightscout_api_token;
        self.is_mmmol = is_mmol;
        self.is_set = true;
    }

    pub fn save_to_disk(self, app: tauri::AppHandle) -> Result<()> {
        let data_dir = app
            .path_resolver()
            .app_data_dir()
            .ok_or(Error::custom("Could not resolve app data dir."))?;
        create_dir_all(&data_dir)?;

        let data_file = data_dir.join("data.json");

        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(data_file)?;

        serde_json::to_writer(file, &self)?;
        Ok(())
    }

    fn set_to_true() -> bool {
        true
    }
}
