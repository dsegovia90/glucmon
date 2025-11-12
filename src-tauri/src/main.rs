// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod config_data;
mod error;
mod nightscout;
mod utils;

static TRAY_MENU_ITEM_GLUCOSE_ENTRY: &str = "tray_menu_item_glucose_entry";
static TRAY_MENU_ITEM_OPEN_SETTINGS: &str = "tray_menu_item_settings";
static TRAY_MENU_ITEM_OPEN_SETTINGS_DISPLAY: &str = "Settings";
static TRAY_MENU_ITEM_QUIT_ENTRY: &str = "tray_menu_item_quit_entry";
static TRAY_MENU_ITEM_QUIT_DISPLAY: &str = "Quit Glucmon Completely";

use commands::{get_glucmon_config, set_glucmon_config};
use config_data::GlucmonConfigStore;
use nightscout::{format_glucose_display, get_glucose_data, Direction};
use std::path::PathBuf;
use std::{sync::Mutex, thread};
use tauri::{
    CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem,
    UserAttentionType,
};
use utils::{create_icon_from_path, get_error_icon, get_icon_path_from_direction};

#[derive(Debug)]
struct Storage {
    config: Mutex<GlucmonConfigStore>,
    last_timestamp: Mutex<Option<u128>>,
    glucose_value: Mutex<Option<f32>>,
    direction: Mutex<Direction>,
    last_display_text: Mutex<Option<String>>,
    last_icon_path: Mutex<Option<PathBuf>>,
}

const POLL_INTERVAL_MS: u64 = 5 * 1000; // 15 seconds when expecting new data
const DATA_EXPECTED_AFTER_MS: u128 = 5 * 60 * 1000; // Expect new data after 5 minutes

fn main() {
    let tray_menu_glucose_data_item = CustomMenuItem::new(TRAY_MENU_ITEM_GLUCOSE_ENTRY, "--");
    let tray_menu_settings_item = CustomMenuItem::new(
        TRAY_MENU_ITEM_OPEN_SETTINGS,
        TRAY_MENU_ITEM_OPEN_SETTINGS_DISPLAY,
    );
    let tray_menu_quit_item =
        CustomMenuItem::new(TRAY_MENU_ITEM_QUIT_ENTRY, TRAY_MENU_ITEM_QUIT_DISPLAY);
    let tray_menu = SystemTrayMenu::new()
        .add_item(tray_menu_glucose_data_item)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(tray_menu_settings_item)
        .add_item(tray_menu_quit_item);
    let tray = SystemTray::new().with_menu(tray_menu);

    let global_app = tauri::Builder::default()
        .system_tray(tray)
        .manage(Storage {
            config: Mutex::new(GlucmonConfigStore {
                ..Default::default()
            }),
            last_timestamp: Mutex::new(None),
            glucose_value: Mutex::new(None),
            direction: Mutex::new(Direction::None),
            last_display_text: Mutex::new(None),
            last_icon_path: Mutex::new(None),
        })
        .setup(move |app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let handle = app.handle();

            let icon_path = match get_icon_path_from_direction(&handle, &Direction::None, "--") {
                Ok(path) => path,
                Err(_) => get_error_icon(&handle),
            };
            let icon = create_icon_from_path(&icon_path).unwrap();
            handle.tray_handle().set_icon(icon).unwrap();

            let binding = handle.state::<Storage>();
            let mut config = binding.config.lock().unwrap();
            config.initialize(handle.app_handle()).unwrap();

            let handle = app.handle();
            thread::spawn(move || {
                let mut last_fetch_time: Option<u128> = None;

                loop {
                    let binding = handle.state::<Storage>();
                    let is_set = binding.config.lock().unwrap().is_set;

                    if is_set {
                        let item_handle =
                            handle.tray_handle().get_item(TRAY_MENU_ITEM_GLUCOSE_ENTRY);

                        // Determine if we should fetch new data
                        let should_fetch = {
                            let last_timestamp = binding.last_timestamp.lock().unwrap();

                            if last_timestamp.is_none() {
                                // First time, fetch immediately
                                true
                            } else {
                                let timestamp = last_timestamp.unwrap();
                                let current_time = std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap()
                                    .as_millis();
                                let time_since_data = current_time - timestamp;

                                // Fetch if data is older than 5 minutes and we haven't fetched recently
                                if time_since_data > DATA_EXPECTED_AFTER_MS {
                                    // Check if enough time passed since last fetch attempt
                                    if let Some(last_fetch) = last_fetch_time {
                                        (current_time - last_fetch) >= POLL_INTERVAL_MS as u128
                                    } else {
                                        true
                                    }
                                } else {
                                    // Check if it's time for the next expected data
                                    time_since_data >= DATA_EXPECTED_AFTER_MS
                                }
                            }
                        };

                        // Fetch new data if needed
                        if should_fetch {
                            let current_time = std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap()
                                .as_millis();
                            last_fetch_time = Some(current_time);

                            match get_glucose_data(handle.app_handle()) {
                                Ok((glucose_value, direction, timestamp)) => {
                                    let binding = handle.state::<Storage>();
                                    let is_new = {
                                        let last_timestamp = binding.last_timestamp.lock().unwrap();
                                        last_timestamp.is_none()
                                            || last_timestamp.unwrap() != timestamp
                                    };

                                    if is_new {
                                        // Store new values
                                        {
                                            let mut state = binding.last_timestamp.lock().unwrap();
                                            *state = Some(timestamp);
                                        }
                                        {
                                            let mut state = binding.glucose_value.lock().unwrap();
                                            *state = Some(glucose_value);
                                        }
                                        {
                                            let mut state = binding.direction.lock().unwrap();
                                            *state = direction;
                                        }

                                        // Update icon only if path changed
                                        let glucose_display =
                                            format_glucose_display(glucose_value, timestamp);
                                        let icon_path = match get_icon_path_from_direction(
                                            &handle,
                                            &direction,
                                            &glucose_display,
                                        ) {
                                            Ok(path) => path,
                                            Err(_) => get_error_icon(&handle),
                                        };

                                        // Only create and set icon if path changed
                                        let should_update_icon = {
                                            let last_icon = binding.last_icon_path.lock().unwrap();
                                            last_icon.as_ref() != Some(&icon_path)
                                        };

                                        if should_update_icon {
                                            let icon = create_icon_from_path(&icon_path).unwrap();
                                            handle.tray_handle().set_icon(icon).unwrap();
                                            let mut last_icon =
                                                binding.last_icon_path.lock().unwrap();
                                            *last_icon = Some(icon_path);
                                        }
                                    }
                                }
                                Err(_e) => {
                                    item_handle.set_title("-- Lost connection --").unwrap();
                                }
                            }
                        }

                        // Update display with current "X mins ago" only if it changed
                        let binding = handle.state::<Storage>();
                        let last_timestamp = binding.last_timestamp.lock().unwrap();
                        let glucose_value = binding.glucose_value.lock().unwrap();

                        if let (Some(timestamp), Some(value)) = (*last_timestamp, *glucose_value) {
                            let glucose_display = format_glucose_display(value, timestamp);

                            // Only update if the display text actually changed
                            let should_update = {
                                let last_display = binding.last_display_text.lock().unwrap();
                                last_display.as_ref() != Some(&glucose_display)
                            };

                            if should_update {
                                item_handle.set_title(&glucose_display).unwrap();
                                let mut last_display = binding.last_display_text.lock().unwrap();
                                *last_display = Some(glucose_display);
                            }

                            // Calculate how old the data is and determine sleep time
                            let current_time = std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap()
                                .as_millis();
                            let time_since_data = current_time - timestamp;

                            if time_since_data > DATA_EXPECTED_AFTER_MS {
                                // Data is stale, poll frequently
                                std::thread::sleep(std::time::Duration::from_millis(
                                    POLL_INTERVAL_MS,
                                ));
                            } else {
                                // Data is fresh, wait until next minute boundary for display update
                                let seconds_into_current_minute = (time_since_data % 60000) / 1000;
                                let seconds_until_next_minute = 60 - seconds_into_current_minute;

                                // Wait until the next minute boundary (when "X mins ago" will change)
                                std::thread::sleep(std::time::Duration::from_secs(
                                    seconds_until_next_minute as u64,
                                ));
                            }
                        } else {
                            // No data yet, check again in 1 second
                            std::thread::sleep(std::time::Duration::from_secs(1));
                        }
                    } else {
                        // Config not set, wait a bit before checking again
                        std::thread::sleep(std::time::Duration::from_millis(2000));
                    }
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            set_glucmon_config,
            get_glucmon_config
        ])
        .on_system_tray_event(|app, event| {
            if let SystemTrayEvent::MenuItemClick { id, .. } = event {
                if TRAY_MENU_ITEM_QUIT_ENTRY == id.as_str() {
                    app.exit(0)
                }
                if TRAY_MENU_ITEM_OPEN_SETTINGS == id.as_str() {
                    if let Some(window) = app.get_window("settings") {
                        window.center().unwrap();
                        window.show().unwrap();
                        window.set_focus().unwrap();
                        window
                            .request_user_attention(Some(UserAttentionType::Informational))
                            .unwrap();
                    } else {
                        tauri::WindowBuilder::new(
                            app,
                            "settings",
                            tauri::WindowUrl::App("index.html".into()),
                        )
                        .inner_size(400.0, 400.0)
                        .title("Glucmon | Settings")
                        .maximizable(false)
                        .minimizable(false)
                        .resizable(false)
                        .build()
                        .expect("Could not create settings window.");
                    }
                }
            }
        });

    global_app
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|_app_handle, event| {
            if let tauri::RunEvent::ExitRequested { api, .. } = event {
                api.prevent_exit();
            }
        })
}
