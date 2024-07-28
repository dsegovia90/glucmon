use crate::error::Result;
use std::path::PathBuf;
use tauri::Icon;

pub fn create_icon_from_path(path: &PathBuf) -> Result<Icon> {
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
