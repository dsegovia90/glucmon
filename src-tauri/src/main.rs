// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use dotenv::dotenv;
use nightscout::get_glucose_data;
use tauri::{CustomMenuItem, SystemTray, SystemTrayMenu};

mod nightscout;

fn main() {
    dotenv().ok();

    let tray_menu_glucose_data_item =
        CustomMenuItem::new("entry".to_string(), get_glucose_data().unwrap());
    let tray_menu = SystemTrayMenu::new().add_item(tray_menu_glucose_data_item);
    let tray = SystemTray::new().with_menu(tray_menu);
    tauri::Builder::default()
        .system_tray(tray)
        .invoke_handler(tauri::generate_handler![])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
