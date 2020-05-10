use {chrono::prelude::*, std::collections::HashSet};

mod ser;

pub struct UserData {
    pub activities: Vec<Activity>,
    pub starting_date: NaiveDate,
}

impl Default for UserData {
    fn default() -> Self {
        Self {
            activities: vec![Activity {
                name: "Unnamed Activity".to_owned(),
                dates: HashSet::default(),
            }],
            starting_date: NaiveDate::from_ymd(2020, 1, 1),
        }
    }
}

pub struct Activity {
    pub name: String,
    pub dates: HashSet<NaiveDate>,
}
