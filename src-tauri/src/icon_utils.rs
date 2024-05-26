use crate::nightscout::Direction;
use tauri::Icon;

pub fn create_icon_from_path(path: &str) -> Result<Icon, Box<dyn std::error::Error>> {
    let image = image::open(path)?.to_rgb8();
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

pub fn get_icon_path_from_direction(direction: Direction) -> String {
    match direction {
        Direction::Flat => "icons/tray/glucmon_icon_FLAT.png".to_string(),
        Direction::FortyFiveUp => "icons/tray/glucmon_icon_FORTYFIVE-UP.png".to_string(),
        Direction::FortyFiveDown => "icons/tray/glucmon_icon_FORTYFIVE-DOWN.png".to_string(),
        Direction::SingleUp => "icons/tray/glucmon_icon_SINGLE-UP.png".to_string(),
        Direction::SingleDown => "icons/tray/glucmon_icon_SINGLE-DOWN.png".to_string(),
        Direction::DoubleUp => "icons/tray/glucmon_icon_DOUBLE-UP.png".to_string(),
        Direction::DoubleDown => "icons/tray/glucmon_icon_DOUBLE-DOWN.png".to_string(),
        Direction::TripleUp => "icons/tray/glucmon_icon_TRIPLE-UP.png".to_string(),
        Direction::TripleDown => "icons/tray/glucmon_icon_TRIPLE-DOWN.png".to_string(),
        Direction::RateOutOfRange => "icons/tray/glucmon_icon_NOT-WORKING.png".to_string(),
        Direction::NotComputable => "icons/tray/glucmon_icon_NOT-WORKING.png".to_string(),
        Direction::None => "icons/tray/glucmon_icon.png".to_string(),
    }
}
