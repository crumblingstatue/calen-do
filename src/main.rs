use {
    byteorder::{ReadBytesExt, WriteBytesExt, LE},
    chrono::prelude::*,
    directories::ProjectDirs,
    std::{
        collections::HashSet,
        fs::File,
        path::{Path, PathBuf},
    },
};

mod date_util;
mod ui;

fn main() {
    let dirs = ProjectDirs::from("", "crumblingstatue", "calen-do").unwrap();
    let data_path = dirs.data_dir();
    if !data_path.exists() {
        std::fs::create_dir_all(data_path).unwrap();
    }
    let mut good_dates: HashSet<NaiveDate> = load_or_new(save_path(data_path));
    ui::run(&mut good_dates);
    save(&good_dates, save_path(data_path));
}

fn save_path(data_dir: &Path) -> PathBuf {
    data_dir.join("calen-do.dat")
}

fn load_or_new<P: AsRef<Path>>(path: P) -> HashSet<NaiveDate> {
    let mut set = HashSet::new();
    let mut f = match File::open(path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Error loading date file: {}", e);
            return set;
        }
    };
    let len = f.read_u32::<LE>().unwrap();
    for _ in 0..len {
        let year = f.read_u16::<LE>().unwrap();
        let month = f.read_u8().unwrap();
        let day = f.read_u8().unwrap();
        set.insert(NaiveDate::from_ymd(year.into(), month.into(), day.into()));
    }
    set
}

fn save<P: AsRef<Path>>(dates: &HashSet<NaiveDate>, path: P) {
    let mut f = File::create(path).unwrap();
    f.write_u32::<LE>(dates.len() as u32).unwrap();
    for date in dates {
        f.write_u16::<LE>(date.year() as u16).unwrap();
        f.write_u8(date.month() as u8).unwrap();
        f.write_u8(date.day() as u8).unwrap();
    }
}
