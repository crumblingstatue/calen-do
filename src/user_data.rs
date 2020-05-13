use chrono::prelude::*;
use std::collections::HashSet;

mod ser;

pub struct UserData {
    pub activities: Vec<Activity>,
}

impl UserData {
    fn new_default(current_date: NaiveDate) -> Self {
        Self {
            activities: vec![Activity::new_default(current_date)],
        }
    }
    pub fn insert_default_activity(&mut self, index: usize, current_date: NaiveDate) {
        self.activities
            .insert(index, Activity::new_default(current_date));
    }
}

pub struct Activity {
    pub name: String,
    pub starting_date: NaiveDate,
    pub dates: HashSet<NaiveDate>,
}

impl Activity {
    fn new_default(current_date: NaiveDate) -> Self {
        Self {
            name: "New Unnamed Activity".to_owned(),
            dates: HashSet::default(),
            starting_date: current_date,
        }
    }
}
