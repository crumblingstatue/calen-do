use crate::{
    date_util::{self, DAYS_PER_WEEK},
    user_data::UserData,
};
use chrono::prelude::*;
use layout::*;
use sfml::{graphics::*, system::Vector2, window::*};
use std::collections::HashMap;

mod color;
mod layout;
mod names;
mod render;

type NActivitiesCache = HashMap<NaiveDate, u8>;
type ActivityIdx = u8;

struct UiState {
    side_ui: SideUi,
    imode: InteractMode,
    current_activity: ActivityIdx,
    overview: bool,
    n_activities_cache: NActivitiesCache,
    edit_mode: bool,
    day_boxes: Vec<DayBox>,
}

impl UiState {
    fn new(current_date: NaiveDate) -> Self {
        Self {
            side_ui: SideUi::new(),
            imode: InteractMode::Default,
            current_activity: 0,
            overview: false,
            n_activities_cache: HashMap::new(),
            edit_mode: false,
            day_boxes: gen_day_boxes(current_date),
        }
    }
}

pub fn run(current_date: NaiveDate, user_data: &mut UserData) {
    let mut t: f32 = 0.;
    let res = render::Resources::load();
    let mut render_ctx = render::RenderContext::with_resources(&res);
    let mut bg_shader =
        Shader::from_memory(None, None, Some(include_str!("../bgshader.glsl"))).unwrap();
    bg_shader.set_uniform_vec2("res", Vector2::new(RES.0 as f32, RES.1 as f32));
    let bg_rect = RectangleShape::with_size(Vector2::new(RES.0 as f32, RES.1 as f32));
    let mut ui_state = UiState::new(current_date);

    while render_ctx.rw.is_open() {
        while let Some(ev) = render_ctx.rw.poll_event() {
            match ev {
                Event::Closed => render_ctx.rw.close(),
                Event::MouseButtonPressed {
                    button: mouse::Button::Left,
                    x,
                    y,
                } => match ui_state.imode {
                    InteractMode::Default => {
                        for day_box in &ui_state.day_boxes {
                            let box_date = day_box.date;
                            if (ui_state.edit_mode
                                || (box_date == current_date || box_date == current_date.pred()))
                                && Rect::new(
                                    day_box.x,
                                    day_box.y,
                                    DAYBOX_SIZE as u16,
                                    DAYBOX_SIZE as u16,
                                )
                                .contains2(x as u16, y as u16)
                                && !user_data.activities[ui_state.current_activity as usize]
                                    .dates
                                    .insert(box_date)
                            {
                                user_data.activities[ui_state.current_activity as usize]
                                    .dates
                                    .remove(&box_date);
                            }
                        }
                        for button in &ui_state.side_ui.buttons {
                            if !button.hidden && button.rect.contains2(x as f32, y as f32) {
                                use ButtonId::*;
                                match button.id {
                                    CurrentActivity => {
                                        ui_state.imode = InteractMode::ActivityRename
                                    }
                                    PrevActivity => {
                                        if ui_state.current_activity > 0 {
                                            ui_state.current_activity -= 1;
                                        }
                                    }
                                    AddActivity => {
                                        user_data.insert_default_activity(
                                            ui_state.current_activity as usize + 1,
                                            current_date,
                                        );
                                        ui_state.current_activity += 1;
                                    }
                                    RemActivity => {
                                        if user_data.activities.len() > 1 {
                                            user_data
                                                .activities
                                                .remove(ui_state.current_activity as usize);
                                            if ui_state.current_activity > 0 {
                                                ui_state.current_activity -= 1;
                                            }
                                        }
                                    }
                                    NextActivity => {
                                        if (ui_state.current_activity as usize)
                                            < user_data.activities.len() - 1
                                        {
                                            ui_state.current_activity += 1;
                                        }
                                    }
                                    Overview => ui_state.overview = !ui_state.overview,
                                    SetStartingDate => {
                                        ui_state.imode = InteractMode::StartingDateSelect
                                    }
                                    EditMode => ui_state.edit_mode = !ui_state.edit_mode,
                                }
                            }
                        }
                        compute_n_activities_cache(&mut ui_state.n_activities_cache, user_data);
                    }
                    InteractMode::StartingDateSelect => {
                        for day_box in &ui_state.day_boxes {
                            if Rect::new(
                                day_box.x,
                                day_box.y,
                                DAYBOX_SIZE as u16,
                                DAYBOX_SIZE as u16,
                            )
                            .contains2(x as u16, y as u16)
                            {
                                user_data.activities[ui_state.current_activity as usize]
                                    .starting_date = day_box.date;
                                ui_state.imode = InteractMode::Default;
                            }
                        }
                        if let Some(button) = ui_state.side_ui.button_at(x as f32, y as f32) {
                            if matches!(button.id, ButtonId::SetStartingDate) {
                                ui_state.imode = InteractMode::Default;
                            }
                        }
                    }
                    InteractMode::ActivityRename => {}
                },
                Event::TextEntered { unicode } => {
                    if matches!(ui_state.imode, InteractMode::ActivityRename) {
                        if unicode == 0x8 as char {
                            user_data.activities[ui_state.current_activity as usize]
                                .name
                                .pop();
                        } else if unicode == 0xD as char {
                            ui_state.imode = InteractMode::Default;
                        } else {
                            user_data.activities[ui_state.current_activity as usize]
                                .name
                                .push(unicode);
                        }
                    }
                }
                _ => {}
            }
            // Toggle visibility/highlighting of ui buttons
            for n in 0..5 {
                ui_state.side_ui.buttons[n].hidden = ui_state.overview;
            }
            for n in 6..ui_state.side_ui.buttons.len() {
                ui_state.side_ui.buttons[n].hidden = ui_state.overview;
            }
            ui_state.side_ui.buttons[6].highlighted =
                matches!(ui_state.imode, InteractMode::StartingDateSelect);
            ui_state.side_ui.buttons[7].highlighted = ui_state.edit_mode;
            ui_state.side_ui.buttons[0].highlighted =
                matches!(ui_state.imode, InteractMode::ActivityRename);
        }
        render_ctx.rw.clear(Color::WHITE);
        // Draw background
        let mut rs = RenderStates::default();
        let tval = (t / 64.).sin().abs();
        bg_shader.set_uniform_float("t", tval);
        bg_shader.set_uniform_float("cx", render_ctx.rw.mouse_position().x as f32 / RES.0 as f32);
        bg_shader.set_uniform_float(
            "cy",
            1.0 - (render_ctx.rw.mouse_position().y as f32 / RES.1 as f32),
        );
        rs.shader = Some(&bg_shader);
        render_ctx.rw.draw_with_renderstates(&bg_rect, rs);
        render::draw_calendar(&mut render_ctx, current_date, &user_data, &ui_state);
        ui_state.side_ui.draw(&mut render_ctx, user_data, &ui_state);
        render_ctx.rw.display();
        t += 1.0;
    }
}

fn compute_n_activities_cache(cache: &mut NActivitiesCache, user_data: &UserData) {
    cache.clear();
    for ac in &user_data.activities {
        for date in &ac.dates {
            *cache.entry(*date).or_insert(0) += 1;
        }
    }
}

struct DayBox {
    x: u16,
    y: u16,
    date: NaiveDate,
}

fn gen_day_boxes(date: NaiveDate) -> Vec<DayBox> {
    let mut boxes = Vec::new();
    let curr_month = date.month();
    for m in 0..12 {
        let month_offset = m as i32 - CURRENT_MONTH_OFFSET as i32;
        let (actual_month, actual_year) =
            date_util::month_year_offset(curr_month as i32, date.year(), month_offset);
        let (x, y) = month_box_pixel_position(m);
        let n_days = date_util::days_in_month(actual_year, actual_month as u8);
        let weekday_offset = NaiveDate::from_ymd(actual_year, actual_month as u32, 1)
            .weekday()
            .num_days_from_monday() as u8;
        for index in weekday_offset..n_days + weekday_offset {
            let dx = (index % DAYS_PER_WEEK) * (DAYBOX_SIZE + DAYBOX_PADDING);
            let dy = (index / DAYS_PER_WEEK) * (DAYBOX_SIZE + DAYBOX_PADDING);
            let magic_y_offset = 44;
            boxes.push(DayBox {
                x: (x as u16 + dx as u16) + MONTH_BOX_PADDING as u16,
                y: (y as u16 + dy as u16) + MONTH_BOX_PADDING as u16 + magic_y_offset,
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

struct SideUi {
    buttons: Vec<Button>,
}

impl SideUi {
    fn new() -> Self {
        use ButtonId::*;
        use ButtonKind::*;
        Self {
            buttons: vec![
                Button {
                    rect: Rect::new(904.0, 4.0, 178.0, 42.0),
                    id: CurrentActivity,
                    kind: RectWithText,
                    hidden: false,
                    highlighted: false,
                },
                Button {
                    rect: Rect::new(934.0, 52.0, 24.0, 24.0),
                    id: PrevActivity,
                    kind: Sprite,
                    hidden: false,
                    highlighted: false,
                },
                Button {
                    rect: Rect::new(964.0, 52.0, 24.0, 24.0),
                    id: AddActivity,
                    kind: Sprite,
                    hidden: false,
                    highlighted: false,
                },
                Button {
                    rect: Rect::new(994.0, 52.0, 24.0, 24.0),
                    id: RemActivity,
                    kind: Sprite,
                    hidden: false,
                    highlighted: false,
                },
                Button {
                    rect: Rect::new(1024.0, 52.0, 24.0, 24.0),
                    id: NextActivity,
                    kind: Sprite,
                    hidden: false,
                    highlighted: false,
                },
                Button {
                    rect: Rect::new(904.0, 82.0, 178.0, 32.0),
                    id: Overview,
                    kind: RectWithText,
                    hidden: false,
                    highlighted: false,
                },
                Button {
                    rect: Rect::new(904.0, 82.0 + 42.0, 178.0, 32.0),
                    id: SetStartingDate,
                    kind: RectWithText,
                    hidden: false,
                    highlighted: false,
                },
                Button {
                    rect: Rect::new(904.0, 82.0 + (2.0 * 42.0), 178.0, 32.0),
                    id: EditMode,
                    kind: RectWithText,
                    hidden: false,
                    highlighted: false,
                },
            ],
        }
    }
    fn button_at(&self, x: f32, y: f32) -> Option<&Button> {
        for b in &self.buttons {
            if b.rect.contains2(x, y) {
                return Some(b);
            }
        }
        None
    }
}

enum ButtonId {
    CurrentActivity,
    PrevActivity,
    AddActivity,
    RemActivity,
    NextActivity,
    Overview,
    SetStartingDate,
    EditMode,
}

struct Button {
    rect: Rect<f32>,
    id: ButtonId,
    kind: ButtonKind,
    hidden: bool,
    highlighted: bool,
}

enum ButtonKind {
    RectWithText,
    Sprite,
}

// How you interact with the calendar and the whole UI
#[derive(Copy, Clone)]
enum InteractMode {
    Default,
    StartingDateSelect,
    ActivityRename,
}
