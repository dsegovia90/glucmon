// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config_data;
mod error;
mod nightscout;
mod utils;

static UPDATE_GLUCOSE_EVENT_ID: &str = "update_glucose";
static TRAY_MENU_ITEM_GLUCOSE_ENTRY: &str = "tray_menu_item_glucose_entry";
static TRAY_MENU_ITEM_OPEN_SETTINGS: &str = "tray_menu_item_settings";
static TRAY_MENU_ITEM_OPEN_SETTINGS_DISPLAY: &str = "Settings";
static TRAY_MENU_ITEM_QUIT_ENTRY: &str = "tray_menu_item_quit_entry";
static TRAY_MENU_ITEM_QUIT_DISPLAY: &str = "Quit Glucmon Completely";

use config_data::{get_glucmon_config, set_glucmon_config, GlucmonConfigStore};
use nightscout::{get_glucose_data, Direction};
use std::{sync::Mutex, thread};
use tauri::{
    CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem,
    UserAttentionType,
};
use utils::{create_icon_from_path, get_icon_path_from_direction};

#[derive(Debug)]
struct Storage {
    config: Mutex<GlucmonConfigStore>,
}

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
        })
        .setup(move |app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let handle = app.handle();

            let icon_path = get_icon_path_from_direction(&handle, &Direction::None, "--");
            let icon = create_icon_from_path(&icon_path).unwrap();
            handle.tray_handle().set_icon(icon).unwrap();

            let binding = handle.state::<Storage>();
            let mut config = binding.config.lock().unwrap();
            config.initialize(handle.app_handle()).unwrap();

            let handle = app.handle();
            thread::spawn(move || loop {
                let binding = handle.state::<Storage>();
                let is_set = binding.config.lock().unwrap().is_set;
                if is_set {
                    handle.trigger_global(UPDATE_GLUCOSE_EVENT_ID, None);
                }
                std::thread::sleep(std::time::Duration::from_millis(2000));
            });

            let handle = app.handle();

            app.listen_global(UPDATE_GLUCOSE_EVENT_ID, move |_| {
                let item_handle = handle.tray_handle().get_item(TRAY_MENU_ITEM_GLUCOSE_ENTRY);
                let (glucose_value_str, direction) = get_glucose_data(handle.app_handle()).unwrap();
                let icon_path =
                    get_icon_path_from_direction(&handle, &direction, &glucose_value_str);
                let icon = create_icon_from_path(&icon_path).unwrap();
                handle.tray_handle().set_icon(icon).unwrap();
                item_handle.set_title(glucose_value_str).unwrap();
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
