use {chrono::prelude::*, std::collections::HashSet};

mod ser;

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
