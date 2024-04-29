// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::thread;

use nightscout::get_glucose_data;
use tauri::{CustomMenuItem, Manager, SystemTray, SystemTrayMenu};

mod nightscout;

#[derive(Clone, serde::Serialize)]
struct Payload {
    message: String,
}

fn main() {
    let tray_menu_glucose_data_item =
        CustomMenuItem::new("entry".to_string(), get_glucose_data().unwrap());
    let tray_menu = SystemTrayMenu::new().add_item(tray_menu_glucose_data_item);
    let tray = SystemTray::new().with_menu(tray_menu);

    let global_app = tauri::Builder::default().system_tray(tray).setup(|app| {
        let handle = app.handle();

        app.listen_global("update_glucose", move |_| {
            let item_handle = handle.tray_handle().get_item("entry");
            item_handle.set_title(get_glucose_data().unwrap()).unwrap();
        });

        let handle = app.handle();

        thread::spawn(move || loop {
            std::thread::sleep(std::time::Duration::from_millis(10000));
            handle.trigger_global("update_glucose", Some("halo?".to_string()));
        });
        Ok(())
    });
    global_app
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
