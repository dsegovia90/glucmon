use crate::nightscout::Direction;
use std::path::PathBuf;
use tauri::AppHandle;

pub fn get_icon_path_from_direction(
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

    app.path_resolver()
        .resolve_resource(format!(
            "{}tray_{}_{}.png",
            base_path, severity, direction_str
        ))
        .expect("failed to resolve resource")
}
