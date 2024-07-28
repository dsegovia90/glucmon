use crate::{config_data::GlucmonConfigStore, Storage};
use tauri::State;

#[tauri::command]
pub fn set_glucmon_config(
    form_config_values: GlucmonConfigStore,
    glucmon_config_store_mutex: State<'_, Storage>,
    app: tauri::AppHandle,
) -> std::result::Result<GlucmonConfigStore, String> {
    dbg!("Setting glucmon config to state.", &form_config_values);
    let mut state = glucmon_config_store_mutex
        .config
        .lock()
        .map_err(|e| e.to_string())?;

    state.update_config(
        form_config_values.nightscout_url,
        form_config_values.nightscout_api_token,
        form_config_values.is_mmmol,
    );

    state.clone().save_to_disk(app).map_err(|e| e.to_string())?;

    Ok(state.clone())
}

#[tauri::command]
pub fn get_glucmon_config(
    glucmon_config_store_mutex: State<'_, Storage>,
) -> std::result::Result<GlucmonConfigStore, String> {
    dbg!("Getting glucmon config from state.");
    let state = glucmon_config_store_mutex
        .config
        .lock()
        .map_err(|e| e.to_string())?;
    Ok(state.clone())
}
