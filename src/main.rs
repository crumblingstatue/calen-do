use {
    byteorder::{ReadBytesExt, WriteBytesExt, LE},
    chrono::prelude::*,
    directories::ProjectDirs,
    sfml::{graphics::*, system::Vector2, window::*},
    std::{
        collections::HashSet,
        fs::File,
        path::{Path, PathBuf},
    },
};

const WEEKDAY_NAMES_2: [&str; DAYS_PER_WEEK as usize] = ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"];

const MONTH_NAMES: [&str; N_MONTHS as usize] = [
    "January",
    "Febuary",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

// Always the current and the next month are the last 2 months displayed.
const CURRENT_MONTH_OFFSET: u8 = 10;

fn draw_text(rw: &mut RenderWindow, text: &mut Text, x: i16, y: i16, string: &str) {
    text.set_position((x.into(), y.into()));
    text.set_string(string);
    rw.draw(text);
}

fn draw_calendar(
    rw: &mut RenderWindow,
    text: &mut Text,
    date: NaiveDate,
    day_boxes: &[DayBox],
    good_dates: &HashSet<NaiveDate>,
) {
    let curr_month = date.month();
    let mut rect = RectangleShape::default();
    rect.set_fill_color(Color::TRANSPARENT);
    let padding_offset = (MONTH_BOX_PADDING / 2) as f32;
    for m in 0..12 {
        rect.set_size((
            MONTH_BOX_SIZE.0 as f32 - MONTH_BOX_PADDING as f32,
            MONTH_BOX_SIZE.1 as f32 - MONTH_BOX_PADDING as f32,
        ));
        let month_offset = m as i32 - CURRENT_MONTH_OFFSET as i32;
        let (actual_month, actual_year) =
            month_year_offset(curr_month as i32, date.year(), month_offset);
        let x = ((m % 4) as f32 * MONTH_BOX_SIZE.0 as f32) + padding_offset;
        let y = ((m / 4) as f32 * MONTH_BOX_SIZE.1 as f32) + padding_offset;
        rect.set_position((x, y));
        if m == CURRENT_MONTH_OFFSET {
            rect.set_outline_color(Color::rgb(255, 128, 0));
            rect.set_outline_thickness(-2.0);
            rw.draw(&rect);
        }
        draw_text(
            rw,
            text,
            x as i16 + 64,
            y as i16 + 4,
            &format!(
                "{} {}",
                MONTH_NAMES[(actual_month - 1) as usize],
                actual_year,
            ),
        );
        for wd in 0..7 {
            draw_text(
                rw,
                text,
                (x as i16 + wd * 36) + 8,
                y as i16 + 32,
                WEEKDAY_NAMES_2[wd as usize],
            );
        }
    }
    for day_box in day_boxes {
        if day_box.date <= date {
            rect.set_position((day_box.x as f32, day_box.y as f32));
            rect.set_size((DAYBOX_SIZE as f32, DAYBOX_SIZE as f32));
            rect.set_outline_color(Color::TRANSPARENT);
            if good_dates.contains(&day_box.date) {
                rect.set_fill_color(Color::GREEN);
            } else {
                rect.set_fill_color(Color::RED);
            }
            rw.draw(&rect);
        }
        if day_box.date == date {
            let mut cs = CircleShape::default();
            cs.set_radius(16.0);
            cs.set_outline_color(Color::MAGENTA);
            cs.set_fill_color(Color::TRANSPARENT);
            cs.set_outline_thickness(4.0);
            cs.set_position((day_box.x as f32 - 5.0, day_box.y as f32 - 5.0));
            rw.draw(&cs);
        }
        draw_text(
            rw,
            text,
            day_box.x as i16,
            day_box.y as i16,
            &format!("{:>2}", day_box.date.day()),
        )
    }
}

fn days_in_month(year: i32, month: u8) -> u8 {
    let next_months_year = if month == 12 { year + 1 } else { year };
    let next_month = if month == 12 { 1 } else { month + 1 };
    NaiveDate::from_ymd(next_months_year, next_month as u32, 1)
        .signed_duration_since(NaiveDate::from_ymd(year, month as u32, 1))
        .num_days() as u8
}

const RES: (u16, u16) = (1024, 720);
const MONTHS_PER_ROW: u8 = 4;
const N_MONTHS: u8 = 12;
const MONTHS_PER_COLUMN: u8 = N_MONTHS / MONTHS_PER_ROW;
const MONTH_BOX_SIZE: (u16, u16) = (
    RES.0 / MONTHS_PER_ROW as u16,
    RES.1 / MONTHS_PER_COLUMN as u16,
);
const MONTH_BOX_PADDING: u16 = 4;
const DAYS_PER_WEEK: u8 = 7;

fn month_year_offset(month: i32, mut year: i32, offset: i32) -> (i32, i32) {
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

fn main() {
    let mut t: f32 = 0.;
    let current_date = {
        let date = Local::now().date();
        NaiveDate::from_ymd(date.year(), date.month(), date.day())
    };

    let mut rw = RenderWindow::new(
        (RES.0.into(), RES.1.into()),
        "Calen-Do!",
        Style::CLOSE,
        &ContextSettings::default(),
    );
    let font = Font::from_memory(include_bytes!("../DejaVuSansMono.ttf")).unwrap();
    let mut text = Text::new("", &font, 16);
    text.set_fill_color(Color::BLACK);
    rw.set_vertical_sync_enabled(true);
    let day_boxes = gen_day_boxes(current_date);
    let dirs = ProjectDirs::from("", "crumblingstatue", "calen-do").unwrap();
    let data_path = dirs.data_dir();
    if !data_path.exists() {
        std::fs::create_dir_all(data_path).unwrap();
    }
    let mut good_dates: HashSet<NaiveDate> = load_or_new(save_path(data_path));
    let mut bg_shader =
        Shader::from_memory(None, None, Some(include_str!("../bgshader.glsl"))).unwrap();
    bg_shader.set_uniform_vec2("res", Vector2::new(RES.0 as f32, RES.1 as f32));
    let bg_rect = RectangleShape::with_size(Vector2::new(RES.0 as f32, RES.1 as f32));
    while rw.is_open() {
        while let Some(ev) = rw.poll_event() {
            match ev {
                Event::Closed => rw.close(),
                Event::MouseButtonPressed {
                    button: mouse::Button::Left,
                    x,
                    y,
                } => {
                    for day_box in &day_boxes {
                        let box_date = day_box.date;
                        if (box_date == current_date || box_date == current_date.pred())
                            && Rect::new(
                                day_box.x,
                                day_box.y,
                                DAYBOX_SIZE as u16,
                                DAYBOX_SIZE as u16,
                            )
                            .contains2(x as u16, y as u16)
                            && !good_dates.insert(box_date)
                        {
                            good_dates.remove(&box_date);
                        }
                    }
                }
                _ => {}
            }
        }
        rw.clear(Color::WHITE);
        // Draw background
        let mut rs = RenderStates::default();
        let tval = (t / 64.).sin().abs();
        bg_shader.set_uniform_float("t", tval);
        bg_shader.set_uniform_float("cx", rw.mouse_position().x as f32 / RES.0 as f32);
        bg_shader.set_uniform_float("cy", 1.0 - (rw.mouse_position().y as f32 / RES.1 as f32));
        rs.shader = Some(&bg_shader);
        rw.draw_with_renderstates(&bg_rect, rs);
        draw_calendar(&mut rw, &mut text, current_date, &day_boxes, &good_dates);
        rw.display();
        t += 1.0;
    }
    save(&good_dates, save_path(data_path));
}

fn save_path(data_dir: &Path) -> PathBuf {
    data_dir.join("calen-do.dat")
}

struct DayBox {
    x: u16,
    y: u16,
    date: NaiveDate,
}

fn gen_day_boxes(date: NaiveDate) -> Vec<DayBox> {
    let mut boxes = Vec::new();
    let curr_month = date.month();
    let padding_offset = (MONTH_BOX_PADDING / 2) as f32;
    for m in 0..12 {
        let month_offset = m as i32 - CURRENT_MONTH_OFFSET as i32;
        let (actual_month, actual_year) =
            month_year_offset(curr_month as i32, date.year(), month_offset);
        let x = ((m % 4) as f32 * MONTH_BOX_SIZE.0 as f32) + padding_offset;
        let y = ((m / 4) as f32 * MONTH_BOX_SIZE.1 as f32) + padding_offset;
        let n_days = days_in_month(actual_year, actual_month as u8);
        let weekday_offset = NaiveDate::from_ymd(actual_year, actual_month as u32, 1)
            .weekday()
            .num_days_from_monday() as u8;
        for index in weekday_offset..n_days + weekday_offset {
            let dx = (index % DAYS_PER_WEEK) * 36;
            let dy = (index / DAYS_PER_WEEK) * 29;
            boxes.push(DayBox {
                x: (x as u16 + dx as u16) + 8,
                y: (y as u16 + dy as u16) + 64,
                date: NaiveDate::from_ymd(
                    actual_year,
                    actual_month as u32,
                    ((index - weekday_offset) + 1) as u32,
                ),
            });
        }
    }
    boxes
}

const DAYBOX_SIZE: u8 = 24;

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
