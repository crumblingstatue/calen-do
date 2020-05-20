#![windows_subsystem = "windows"]
#![warn(clippy::cast_lossless)]

use crate::user_data::UserData;
use chrono::prelude::*;
use directories::ProjectDirs;
use std::error::Error;

mod date_util;
mod ui;
mod user_data;

fn run() -> Result<(), Box<dyn Error>> {
    let dirs =
        ProjectDirs::from("", "crumblingstatue", "calen-do").ok_or("Can't create ProjectDirs")?;
    let data_dir = dirs.data_dir();
    if !data_dir.exists() {
        std::fs::create_dir_all(data_dir)?;
    }
    let current_date = {
        let date = Local::now().date();
        NaiveDate::from_ymd(date.year(), date.month(), date.day())
    };
    let test_mode = matches!(std::env::args().nth(1).as_deref(), Some("--test"));
    let mut user_data = UserData::load_or_new(data_dir, current_date, test_mode);
    ui::run(current_date, &mut user_data)?;
    user_data.save(data_dir, test_mode)?;
    Ok(())
}

fn main() {
    let result = run();
    if let Err(e) = result {
        msgbox::create(
            "Fatal error",
            &format!("Fatal error: {}", e),
            msgbox::IconType::Error,
        );
    }
}
