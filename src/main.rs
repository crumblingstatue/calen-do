use {crate::user_data::UserData, chrono::prelude::*, directories::ProjectDirs};

mod date_util;
mod ui;
mod user_data;

fn main() {
    let dirs = ProjectDirs::from("", "crumblingstatue", "calen-do").unwrap();
    let data_dir = dirs.data_dir();
    if !data_dir.exists() {
        std::fs::create_dir_all(data_dir).unwrap();
    }
    let current_date = {
        let date = Local::now().date();
        NaiveDate::from_ymd(date.year(), date.month(), date.day())
    };
    let mut user_data = UserData::load_or_new(data_dir, current_date);
    ui::run(current_date, &mut user_data);
    user_data.save(data_dir);
}
