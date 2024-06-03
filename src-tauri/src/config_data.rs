use std::fs::{create_dir_all, OpenOptions};
use std::io::Read;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GlucmonConfigStore {
    pub nightscout_url: String,
    pub nightscout_api_token: String,
    pub is_mmmol: bool,
}

impl GlucmonConfigStore {
    fn new() -> Self {
        Self {
            nightscout_url: "".to_string(),
            nightscout_api_token: "".to_string(),
            is_mmmol: true,
        }
    }

    pub fn default(app: tauri::AppHandle) -> anyhow::Result<Self> {
        let data_dir = app
            .path_resolver()
            .app_data_dir()
            .expect("Could not create/open data_dir folder.");
        create_dir_all(&data_dir)?;

        let data_file = data_dir.join("data.json");

        dbg!("Reading/writing to json file: ", &data_file);

        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(data_file)?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let json = match serde_json::from_str(&contents) {
            Ok(json) => json,
            Err(err) => {
                println!("Failed to load json, creating from scratch. Error: {}", err);
                Self::new()
            }
        };

        dbg!(&json);
        // file.write_all(b"hello world")
        //     .expect("Could not write to file.");

        Ok(json)
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
            .open(data_file)?;

        serde_json::to_writer(file, &self)?;
        Ok(())
    }
}

#[tauri::command]
pub fn set_glucmon_config(
    form_config_values: GlucmonConfigStore,
    glucmon_config_store_mutex: State<'_, Mutex<GlucmonConfigStore>>,
    app: tauri::AppHandle,
) -> Result<GlucmonConfigStore, String> {
    dbg!("Setting glucmon config to state.", &form_config_values);
    let mut state = glucmon_config_store_mutex
        .lock()
        .expect("Could not lock glucmon_config_store_mutex.");
    state.nightscout_url = form_config_values.nightscout_url;
    state.nightscout_api_token = form_config_values.nightscout_api_token;
    state.is_mmmol = form_config_values.is_mmmol;

    dbg!(&state);
    state.clone().save_to_disk(app).unwrap();

    Ok(state.clone())
}

#[tauri::command]
pub fn get_glucmon_config(
    glucmon_config_store_mutex: State<'_, Mutex<GlucmonConfigStore>>,
) -> Result<GlucmonConfigStore, String> {
    dbg!("Getting glucmon config from state.");
    let state = glucmon_config_store_mutex
        .lock()
        .expect("Could not lock glucmon_config_store_mutex.");
    Ok(state.clone())
}
