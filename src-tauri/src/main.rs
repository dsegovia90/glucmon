// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{os::unix::process, thread};

use config_data::initialize_config_data;
use nightscout::get_glucose_data;
use tauri::{
    CustomMenuItem, Manager, MenuItem, SystemTray, SystemTrayEvent, SystemTrayMenu,
    SystemTrayMenuItem,
};

mod config_data;
mod nightscout;

#[derive(Clone, serde::Serialize)]
struct Payload {
    message: String,
}

fn main() {
    let tray_menu_glucose_data_item = CustomMenuItem::new("entry", get_glucose_data().unwrap());
    let tray_menu_quite_item = CustomMenuItem::new("quit", "Quit Glucmon Completely");
    let tray_menu = SystemTrayMenu::new()
        .add_item(tray_menu_glucose_data_item)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(tray_menu_quite_item);
    let tray = SystemTray::new().with_menu(tray_menu);

    let global_app = tauri::Builder::default()
        .system_tray(tray)
        .setup(|app| {
            let handle = app.handle();
            initialize_config_data(app).unwrap();

            app.listen_global("update_glucose", move |_| {
                let item_handle = handle.tray_handle().get_item("entry");
                item_handle.set_title(get_glucose_data().unwrap()).unwrap();
            });

            let handle = app.handle();

            thread::spawn(move || loop {
                std::thread::sleep(std::time::Duration::from_millis(10000));
                handle.trigger_global("update_glucose", None);
            });
            Ok(())
        })
        .on_system_tray_event(|app, event| {
            if let SystemTrayEvent::MenuItemClick { id, .. } = event {
                if let "quit" = id.as_str() {
                    app.exit(0)
                }
            }
        });

    global_app
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
