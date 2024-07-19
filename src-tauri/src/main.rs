// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

static UPDATE_GLUCOSE_EVENT_ID: &str = "update_glucose";
static TRAY_MENU_ITEM_GLUCOSE_ENTRY: &str = "tray_menu_item_glucose_entry";
static TRAY_MENU_ITEM_OPEN_SETTINGS: &str = "tray_menu_item_settings";
static TRAY_MENU_ITEM_OPEN_SETTINGS_DISPLAY: &str = "Settings";
static TRAY_MENU_ITEM_QUIT_ENTRY: &str = "tray_menu_item_quit_entry";
static TRAY_MENU_ITEM_QUIT_DISPLAY: &str = "Quit Glucmon Completely";

use config_data::{get_glucmon_config, set_glucmon_config, GlucmonConfigStore};
use nightscout::{get_glucose_data, Direction};
use std::path::PathBuf;
use std::{sync::Mutex, thread};
use tauri::{
    AppHandle, CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu,
    SystemTrayMenuItem,
};

#[derive(Debug)]
struct Storage {
    config: Mutex<GlucmonConfigStore>,
}

mod config_data;
mod nightscout;

use tauri::Icon;

fn create_icon_from_path(path: &PathBuf) -> Result<Icon, Box<dyn std::error::Error>> {
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

fn get_icon_path_from_direction(
    app: &AppHandle,
    direction: &Direction,
    glucose_value: &str,
) -> PathBuf {
    let base_path = "icons/tray/";

    // Parse the glucose_value string to f64
    let parts: Vec<&str> = glucose_value.split_whitespace().collect();
    let glucose_str = parts[0];
    let glucose_f64 = match glucose_str.parse::<f64>() {
        Ok(value) => value,
        Err(_) => {
            return app
                .path_resolver()
                .resolve_resource(format!("{}glucmon_icon_NOT-CONFIGURED.png", base_path))
                .expect("failed to resolve resource")
        }
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
        Direction::RateOutOfRange => {
            return app
                .path_resolver()
                .resolve_resource(format!("{}glucmon_icon_NOT-WORKING.png", base_path))
                .expect("failed to resolve resource")
        }
        Direction::NotComputable => {
            return app
                .path_resolver()
                .resolve_resource(format!("{}glucmon_icon_NOT-WORKING.png", base_path))
                .expect("failed to resolve resource")
        }
        Direction::None => {
            return app
                .path_resolver()
                .resolve_resource(format!("{}glucmon_icon.png", base_path))
                .expect("failed to resolve resource")
        }
    };
    dbg!("{}tray_{}_{}.png", base_path, severity, direction_str);

    app.path_resolver()
        .resolve_resource(format!(
            "{}tray_{}_{}.png",
            base_path, severity, direction_str
        ))
        .expect("failed to resolve resource")
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

            // let handle = app.handle();

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
                    tauri::WindowBuilder::new(
                        app,
                        "local",
                        tauri::WindowUrl::App("index.html".into()),
                    )
                    .inner_size(400.0, 400.0)
                    .title("Glucmon | Settings")
                    .build()
                    .expect("Could not create settings window.");
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
