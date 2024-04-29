use std::fs::{create_dir_all, OpenOptions};
use std::io::{Read, Write};

pub fn initialize_config_data(app: &tauri::App) -> anyhow::Result<()> {
    let data_dir = app.path_resolver().app_data_dir().unwrap();
    create_dir_all(&data_dir)?;

    let data_file = data_dir.join("data.txt");

    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(data_file)?;

    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    dbg!(contents);
    // file.write_all(b"hello world")
    //     .expect("Could not write to file.");

    Ok(())
}
