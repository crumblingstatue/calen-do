use {
    byteorder::{ReadBytesExt, WriteBytesExt, LE},
    chrono::prelude::*,
    std::{
        collections::HashSet,
        error::Error,
        fs::File,
        path::{Path, PathBuf},
    },
};

pub struct UserData {
    pub good_dates: HashSet<NaiveDate>,
    pub starting_date: NaiveDate,
}

impl Default for UserData {
    fn default() -> Self {
        Self {
            good_dates: Default::default(),
            starting_date: NaiveDate::from_ymd(2020, 1, 1),
        }
    }
}

impl UserData {
    pub fn load_or_new(data_dir: &Path) -> Self {
        let path = save_path(data_dir);
        match Self::try_load(&path) {
            Ok(data) => data,
            Err(e) => {
                eprintln!("Error loading user data from {}: {}", path.display(), e);
                Self::default()
            }
        }
    }
    fn try_load(path: &Path) -> Result<Self, Box<dyn Error>> {
        let mut set = HashSet::new();
        let mut f = File::open(path)?;
        let starting_year = f.read_u16::<LE>()?;
        let starting_month = f.read_u8()?;
        let starting_day = f.read_u8()?;
        let len = f.read_u32::<LE>()?;
        for _ in 0..len {
            let year = f.read_u16::<LE>()?;
            let month = f.read_u8()?;
            let day = f.read_u8()?;
            set.insert(NaiveDate::from_ymd(year.into(), month.into(), day.into()));
        }
        Ok(UserData {
            good_dates: set,
            starting_date: NaiveDate::from_ymd(
                starting_year as i32,
                starting_month as u32,
                starting_day as u32,
            ),
        })
    }
    pub fn save(&self, data_dir: &Path) {
        let mut f = File::create(save_path(data_dir)).unwrap();
        f.write_u16::<LE>(self.starting_date.year() as u16).unwrap();
        f.write_u8(self.starting_date.month() as u8).unwrap();
        f.write_u8(self.starting_date.day() as u8).unwrap();
        f.write_u32::<LE>(self.good_dates.len() as u32).unwrap();
        for date in &self.good_dates {
            f.write_u16::<LE>(date.year() as u16).unwrap();
            f.write_u8(date.month() as u8).unwrap();
            f.write_u8(date.day() as u8).unwrap();
        }
    }
}

fn save_path(data_dir: &Path) -> PathBuf {
    data_dir.join("calen-do.dat")
}
