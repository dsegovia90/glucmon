// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

static UPDATE_GLUCOSE_EVENT_ID: &str = "update_glucose";
static TRAY_MENU_ITEM_GLUCOSE_ENTRY: &str = "tray_menu_item_glucose_entry";
static TRAY_MENU_ITEM_QUIT_ENTRY: &str = "tray_menu_item_quit_entry";
static TRAY_MENU_ITEM_QUIT_DISPLAY: &str = "Quit Glucmon Completely";

use config_data::initialize_config_data;
use icon_utils::{create_icon_from_path, get_icon_path_from_direction};
use nightscout::get_glucose_data;
use std::thread;
use tauri::{
    CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem,
};

mod config_data;
mod icon_utils;
mod nightscout;

fn main() {
    let (glucose_value_str, direction) = get_glucose_data().unwrap();
    let tray_menu_glucose_data_item =
        CustomMenuItem::new(TRAY_MENU_ITEM_GLUCOSE_ENTRY, glucose_value_str);
    let tray_menu_quite_item =
        CustomMenuItem::new(TRAY_MENU_ITEM_QUIT_ENTRY, TRAY_MENU_ITEM_QUIT_DISPLAY);
    let tray_menu = SystemTrayMenu::new()
        .add_item(tray_menu_glucose_data_item)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(tray_menu_quite_item);
    let tray = SystemTray::new().with_menu(tray_menu);

    let global_app = tauri::Builder::default()
        .system_tray(tray)
        .setup(move |app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let handle = app.handle();
            initialize_config_data(app).unwrap();

            let icon_path = get_icon_path_from_direction(direction);
            let icon = create_icon_from_path(&icon_path).unwrap();
            handle.tray_handle().set_icon(icon).unwrap();

            app.listen_global(UPDATE_GLUCOSE_EVENT_ID, move |_| {
                let item_handle = handle.tray_handle().get_item(TRAY_MENU_ITEM_GLUCOSE_ENTRY);
                let (glucose_value_str, direction) = get_glucose_data().unwrap();
                let icon_path = get_icon_path_from_direction(direction);
                let icon = create_icon_from_path(&icon_path).unwrap();
                handle.tray_handle().set_icon(icon).unwrap();
                item_handle.set_title(glucose_value_str).unwrap();
            });

            let handle = app.handle();

            thread::spawn(move || loop {
                std::thread::sleep(std::time::Duration::from_millis(10000));
                handle.trigger_global(UPDATE_GLUCOSE_EVENT_ID, None);
            });
            Ok(())
        })
        .on_system_tray_event(|app, event| {
            if let SystemTrayEvent::MenuItemClick { id, .. } = event {
                if TRAY_MENU_ITEM_QUIT_ENTRY == id.as_str() {
                    app.exit(0)
                }
            }
        });

    global_app
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
