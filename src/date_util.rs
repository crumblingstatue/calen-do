use chrono::NaiveDate;

pub fn month_year_offset(month: i32, mut year: i32, offset: i32) -> (i32, i32) {
    let mut new_month = month + offset;
    if new_month < 1 {
        new_month += 12;
        year -= 1;
    } else if new_month > 12 {
        new_month = offset;
        year += 1;
    }
    (new_month, year)
}

#[test]
fn test_month_year_offset() {
    assert_eq!(month_year_offset(1, 2020, -1), (12, 2019));
    assert_eq!(month_year_offset(1, 2020, -2), (11, 2019));
    assert_eq!(month_year_offset(12, 2020, 1), (1, 2021));
    assert_eq!(month_year_offset(12, 2020, 2), (2, 2021));
    assert_eq!(month_year_offset(4, 2020, -10), (6, 2019));
    assert_eq!(month_year_offset(4, 2020, 0), (4, 2020));
}

pub fn days_in_month(year: i32, month: u8) -> u8 {
    let next_months_year = if month == 12 { year + 1 } else { year };
    let next_month = if month == 12 { 1 } else { month + 1 };
    NaiveDate::from_ymd(next_months_year, u32::from(next_month), 1)
        .signed_duration_since(NaiveDate::from_ymd(year, u32::from(month), 1))
        .num_days() as u8
}

pub const DAYS_PER_WEEK: u8 = 7;
pub const MONTHS_PER_YEAR: u8 = 12;
