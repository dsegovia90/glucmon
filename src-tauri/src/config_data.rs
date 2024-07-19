use std::fs::{create_dir_all, OpenOptions};
use std::io::Read;

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::Storage;

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

    pub fn initialize(&mut self, app: tauri::AppHandle) -> anyhow::Result<()> {
        let data_dir = app
            .path_resolver()
            .app_data_dir()
            .expect("Could not create/open data_dir folder.");
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

    pub fn save_to_disk(self, app: tauri::AppHandle) -> anyhow::Result<()> {
        let data_dir = app
            .path_resolver()
            .app_data_dir()
            .expect("Could not create/open data_dir folder.");
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

#[tauri::command]
pub fn set_glucmon_config(
    form_config_values: GlucmonConfigStore,
    glucmon_config_store_mutex: State<'_, Storage>,
    app: tauri::AppHandle,
) -> Result<GlucmonConfigStore, String> {
    dbg!("Setting glucmon config to state.", &form_config_values);
    let mut state = glucmon_config_store_mutex
        .config
        .lock()
        .expect("Could not lock glucmon_config_store_mutex.");

    state.update_config(
        form_config_values.nightscout_url,
        form_config_values.nightscout_api_token,
        form_config_values.is_mmmol,
    );
    dbg!(&state);

    state.clone().save_to_disk(app).unwrap();

    dbg!(&state);

    Ok(state.clone())
}

#[tauri::command]
pub fn get_glucmon_config(
    glucmon_config_store_mutex: State<'_, Storage>,
) -> Result<GlucmonConfigStore, String> {
    dbg!("Getting glucmon config from state.");
    let state = glucmon_config_store_mutex
        .config
        .lock()
        .expect("Could not lock glucmon_config_store_mutex.");
    Ok(state.clone())
}
