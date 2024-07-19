// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

static UPDATE_GLUCOSE_EVENT_ID: &str = "update_glucose";
static TRAY_MENU_ITEM_GLUCOSE_ENTRY: &str = "tray_menu_item_glucose_entry";
static TRAY_MENU_ITEM_QUIT_ENTRY: &str = "tray_menu_item_quit_entry";
static TRAY_MENU_ITEM_QUIT_DISPLAY: &str = "Quit Glucmon Completely";

use crate::nightscout::Direction;
use config_data::initialize_config_data;
use nightscout::get_glucose_data;
use std::thread;
use tauri::{
    CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem,
};

mod config_data;
mod nightscout;

use tauri::Icon;

fn create_icon_from_path(path: &str) -> Result<Icon, Box<dyn std::error::Error>> {
    let image = image::open(path)?.to_rgba8();
    let (width, height) = image.dimensions();
    let resized_image = image::imageops::resize(
        &image,
        width / 2,
        height / 2,
        image::imageops::FilterType::Nearest,
    );
    Ok(Icon::Rgba {
        rgba: resized_image.into_vec(),
        width: width / 2,
        height: height / 2,
    })
}

fn get_icon_path_from_direction(direction: &Direction, glucose_value: &str) -> String {
    let base_path = "icons/tray/";

    // Parse the glucose_value string to f64
    let parts: Vec<&str> = glucose_value.split_whitespace().collect();
    let glucose_str = parts[0];
    let glucose_f64 = match glucose_str.parse::<f64>() {
        Ok(value) => value,
        Err(_) => return format!("{}glucmon_icon_NOT-CONFIGURED.png", base_path),
    };

    let severity = match glucose_f64 {
        v if v < 3.0 => "urgent",
        v if v < 3.885 => "concern",
        v if v < 10.0 => "normal",
        v if v < 13.875 => "concern",
        _ => "urgent",
    };

    let direction_str = match direction {
        Direction::Flat => "flat",
        Direction::FortyFiveUp => "1_up",     // "fortyfive_up",
        Direction::FortyFiveDown => "1_down", // "fortyfive_down",
        Direction::SingleUp => "1_up",
        Direction::SingleDown => "1_down",
        Direction::DoubleUp => "2_up",
        Direction::DoubleDown => "2_down",
        Direction::TripleUp => "3_up",
        Direction::TripleDown => "3_down",
        Direction::RateOutOfRange => return format!("{}glucmon_icon_NOT-WORKING.png", base_path),
        Direction::NotComputable => return format!("{}glucmon_icon_NOT-WORKING.png", base_path),
        Direction::None => return format!("{}glucmon_icon.png", base_path),
    };
    println!("{}tray_{}_{}.png", base_path, severity, direction_str);

    format!("{}tray_{}_{}.png", base_path, severity, direction_str).to_string()
}
fn main() {
    let (glucose_value_str, direction) = get_glucose_data().unwrap();
    let tray_menu_glucose_data_item =
        CustomMenuItem::new(TRAY_MENU_ITEM_GLUCOSE_ENTRY, &glucose_value_str);
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

            let icon_path = get_icon_path_from_direction(&direction, &glucose_value_str);
            let icon = create_icon_from_path(&icon_path).unwrap();
            handle.tray_handle().set_icon(icon).unwrap();

            app.listen_global(UPDATE_GLUCOSE_EVENT_ID, move |_| {
                let item_handle = handle.tray_handle().get_item(TRAY_MENU_ITEM_GLUCOSE_ENTRY);
                let (glucose_value_str, direction) = get_glucose_data().unwrap();

                let icon_path = get_icon_path_from_direction(&direction, &glucose_value_str);
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
