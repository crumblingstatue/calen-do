use {
    super::{Activity, UserData},
    byteorder::{ReadBytesExt, WriteBytesExt, LE},
    chrono::prelude::*,
    std::{
        collections::HashSet,
        error::Error,
        fs::File,
        io::prelude::*,
        path::{Path, PathBuf},
    },
};

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
        let mut f = File::open(path)?;
        let starting_year = f.read_u16::<LE>()?;
        let starting_month = f.read_u8()?;
        let starting_day = f.read_u8()?;
        let n_activities = f.read_u32::<LE>()?;
        let mut activities = Vec::new();
        for _ in 0..n_activities {
            let mut set = HashSet::new();
            let name_len = f.read_u8()?;
            let mut name_buf = vec![0; name_len as usize];
            f.read_exact(&mut name_buf)?;
            let name = String::from_utf8(name_buf).unwrap();
            let len = f.read_u32::<LE>()?;
            for _ in 0..len {
                let year = f.read_u16::<LE>()?;
                let month = f.read_u8()?;
                let day = f.read_u8()?;
                set.insert(NaiveDate::from_ymd(year.into(), month.into(), day.into()));
            }
            activities.push(Activity { name, dates: set });
        }
        Ok(UserData {
            activities,
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
        f.write_u32::<LE>(self.activities.len() as u32).unwrap();
        for ac in &self.activities {
            f.write_u8(ac.name.len() as u8).unwrap();
            f.write_all(ac.name.as_bytes()).unwrap();
            f.write_u32::<LE>(ac.dates.len() as u32).unwrap();
            for date in &ac.dates {
                f.write_u16::<LE>(date.year() as u16).unwrap();
                f.write_u8(date.month() as u8).unwrap();
                f.write_u8(date.day() as u8).unwrap();
            }
        }
    }
}

fn save_path(data_dir: &Path) -> PathBuf {
    data_dir.join("calen-do.dat")
}
