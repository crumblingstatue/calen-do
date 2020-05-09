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
