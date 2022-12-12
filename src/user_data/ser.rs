use super::{Activity, UserData};
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use chrono::prelude::*;
use rfd::MessageLevel;
use std::{
    collections::HashSet,
    error::Error,
    fs::File,
    io::prelude::*,
    path::{Path, PathBuf},
};

const TEST_MODE_PATH: &str = "calen-do-test.dat";

impl UserData {
    pub fn load_or_new(data_dir: &Path, current_date: NaiveDate, test_mode: bool) -> Self {
        let path = if test_mode {
            PathBuf::from(TEST_MODE_PATH)
        } else {
            save_path(data_dir)
        };
        match Self::try_load(&path) {
            Ok(data) => data,
            Err(e) => {
                let msg = format!(
                    "Error loading user data from {}: {}\n\
                    Creating new user data.\n\
                    If this is your first time running the program, this is natural.",
                    path.display(),
                    e
                );
                rfd::MessageDialog::new()
                    .set_title("Warning")
                    .set_description(&msg)
                    .set_level(MessageLevel::Info)
                    .show();
                Self::new_default(current_date)
            }
        }
    }
    fn try_load(path: &Path) -> Result<Self, Box<dyn Error>> {
        let mut f = File::open(path)?;
        verify(&mut f)?;
        let n_activities = f.read_u32::<LE>()?;
        let mut activities = Vec::with_capacity(n_activities as usize);
        for _ in 0..n_activities {
            let name_len = f.read_u8()?;
            let mut name_buf = vec![0; name_len as usize];
            f.read_exact(&mut name_buf)?;
            let name = String::from_utf8(name_buf)?;
            let starting_year = f.read_u16::<LE>()?;
            let starting_month = f.read_u8()?;
            let starting_day = f.read_u8()?;
            let len = f.read_u32::<LE>()?;
            let mut set = HashSet::with_capacity(len as usize);
            for _ in 0..len {
                let year = f.read_u16::<LE>()?;
                let month = f.read_u8()?;
                let day = f.read_u8()?;
                set.insert(NaiveDate::from_ymd_opt(year.into(), month.into(), day.into()).unwrap());
            }
            activities.push(Activity {
                name,
                starting_date: NaiveDate::from_ymd_opt(
                    i32::from(starting_year),
                    u32::from(starting_month),
                    u32::from(starting_day),
                )
                .unwrap(),
                dates: set,
            });
        }
        Ok(UserData { activities })
    }
    pub fn save(&self, data_dir: &Path, test_mode: bool) -> Result<(), Box<dyn Error>> {
        let path = if test_mode {
            PathBuf::from(TEST_MODE_PATH)
        } else {
            save_path(data_dir)
        };
        let mut f = File::create(path)?;
        f.write_all(MAGIC)?;
        f.write_u16::<LE>(VERSION)?;
        f.write_u32::<LE>(self.activities.len() as u32)?;
        for ac in &self.activities {
            f.write_u8(ac.name.len() as u8)?;
            f.write_all(ac.name.as_bytes())?;
            f.write_u16::<LE>(ac.starting_date.year() as u16)?;
            f.write_u8(ac.starting_date.month() as u8)?;
            f.write_u8(ac.starting_date.day() as u8)?;
            f.write_u32::<LE>(ac.dates.len() as u32)?;
            for date in &ac.dates {
                f.write_u16::<LE>(date.year() as u16)?;
                f.write_u8(date.month() as u8)?;
                f.write_u8(date.day() as u8)?;
            }
        }
        Ok(())
    }
}

fn save_path(data_dir: &Path) -> PathBuf {
    data_dir.join("calen-do.dat")
}

const MAGIC: &[u8] = b"CALDOSAVE";
const VERSION: u16 = 1;

fn verify<R: Read>(reader: &mut R) -> Result<(), Box<dyn Error>> {
    let mut buf = [0; MAGIC.len()];
    reader.read_exact(&mut buf)?;
    if buf != MAGIC {
        return Err("Not a valid cal-do save file".into());
    }
    let ver = reader.read_u16::<LE>()?;
    if ver == VERSION {
        Ok(())
    } else {
        Err(format!("Version mismatch: program ver: {VERSION} vs save ver: {ver}",).into())
    }
}
