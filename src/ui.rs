use {
    crate::{
        date_util::{self, DAYS_PER_WEEK},
        user_data::UserData,
    },
    chrono::prelude::*,
    layout::*,
    names::*,
    sfml::{
        graphics::*,
        system::{SfBox, Vector2},
        window::*,
    },
    std::collections::HashMap,
};

mod color;
mod layout;
mod names;

fn draw_text(render_ctx: &mut RenderContext, x: i16, y: i16, string: &str) {
    render_ctx.text.set_position((x.into(), y.into()));
    render_ctx.text.set_string(string);
    render_ctx.rw.draw(&render_ctx.text);
}

type NActivitiesCache = HashMap<NaiveDate, u8>;

fn draw_calendar(
    render_ctx: &mut RenderContext,
    date: NaiveDate,
    user_data: &UserData,
    ui_state: &UiState,
) {
    render_ctx.text.set_fill_color(Color::BLACK);
    let curr_month = date.month();
    let mut rect = RectangleShape::default();
    rect.set_fill_color(Color::TRANSPARENT);
    for m in 0..12 {
        rect.set_size((MONTH_BOX_SIZE.0 as f32, MONTH_BOX_SIZE.1 as f32));
        let month_offset = m as i32 - CURRENT_MONTH_OFFSET as i32;
        let (actual_month, actual_year) =
            date_util::month_year_offset(curr_month as i32, date.year(), month_offset);
        let (x, y) = month_box_pixel_position(m);
        if m == CURRENT_MONTH_OFFSET {
            rect.set_position((x, y));
            rect.set_outline_color(color::GOLD);
            rect.set_outline_thickness(2.0);
            render_ctx.rw.draw(&rect);
        }
        draw_text(
            render_ctx,
            x as i16 + 64,
            y as i16 + MONTH_BOX_PADDING as i16,
            &format!(
                "{} {}",
                MONTH_NAMES[(actual_month - 1) as usize],
                actual_year,
            ),
        );
        for wd in 0..7 {
            draw_text(
                render_ctx,
                (x as i16 + wd * (DAYBOX_SIZE + DAYBOX_PADDING) as i16) + MONTH_BOX_PADDING as i16,
                y as i16 + MONTH_BOX_PADDING as i16 + 16 + MONTH_BOX_PADDING as i16,
                WEEKDAY_NAMES_2[wd as usize],
            );
        }
    }
    for day_box in &ui_state.day_boxes {
        let starting_date = if ui_state.overview {
            user_data
                .activities
                .iter()
                .min_by_key(|act| act.starting_date)
                .unwrap()
                .starting_date
        } else {
            user_data.activities[ui_state.current_activity as usize].starting_date
        };
        if day_box.date >= starting_date && day_box.date <= date {
            render_ctx
                .sprite
                .set_position((day_box.x as f32, day_box.y as f32));
            if ui_state.overview {
                let n_activities = *ui_state.n_activities_cache.get(&day_box.date).unwrap_or(&0);
                let (sprite_idx, text_color) = match n_activities {
                    0 => (1, Color::WHITE),
                    1 => (0, color::GOLD_BRIGHTER),
                    2 => (6, color::GOLD_BRIGHTER),
                    _ => (7, color::GOLD_BRIGHTER),
                };
                render_ctx.text.set_fill_color(text_color);
                render_ctx
                    .sprite
                    .set_texture_rect(&IntRect::new(sprite_idx * 24, 0, 24, 24));
            } else if user_data.activities[ui_state.current_activity as usize]
                .dates
                .contains(&day_box.date)
            {
                if day_box.date == date {
                    render_ctx.text.set_fill_color(color::GOLD_BRIGHTER);
                } else {
                    render_ctx.text.set_fill_color(Color::BLACK);
                }
                render_ctx.sprite.set_texture_rect(&IntRect::new(
                    0,
                    0,
                    DAYBOX_SIZE as i32,
                    DAYBOX_SIZE as i32,
                ));
            } else {
                if day_box.date == date {
                    render_ctx.text.set_fill_color(color::GOLD_BRIGHTER);
                } else {
                    render_ctx.text.set_fill_color(Color::WHITE);
                }
                render_ctx.sprite.set_texture_rect(&IntRect::new(
                    DAYBOX_SIZE as i32,
                    0,
                    DAYBOX_SIZE as i32,
                    DAYBOX_SIZE as i32,
                ));
            }
            render_ctx.rw.draw(&render_ctx.sprite);
        } else {
            render_ctx.text.set_fill_color(Color::BLACK);
        }
        if day_box.date == date {
            rect.set_outline_color(color::GOLD);
            rect.set_outline_thickness(2.0);
            rect.set_size((DAYBOX_SIZE as f32, DAYBOX_SIZE as f32));
            rect.set_position((day_box.x as f32, day_box.y as f32));
            render_ctx.rw.draw(&rect);
        }
        draw_text(
            render_ctx,
            day_box.x as i16 + 2,
            day_box.y as i16 + 2,
            &format!("{:>2}", day_box.date.day()),
        )
    }
}

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

struct Resources {
    font: SfBox<Font>,
    sprite_sheet: SfBox<Texture>,
}

impl Resources {
    fn load() -> Self {
        Self {
            font: Font::from_memory(include_bytes!("../DejaVuSansMono.ttf")).unwrap(),
            sprite_sheet: Texture::from_memory(
                include_bytes!("../graphics.png"),
                &IntRect::default(),
            )
            .unwrap(),
        }
    }
}

struct RenderContext<'res> {
    text: Text<'res>,
    sprite: Sprite<'res>,
    rw: RenderWindow,
}

impl<'res> RenderContext<'res> {
    fn with_resources(res: &'res Resources) -> Self {
        let mut rw = RenderWindow::new(
            (RES.0.into(), RES.1.into()),
            "Calen-Do!",
            Style::CLOSE,
            &ContextSettings::default(),
        );
        rw.set_vertical_sync_enabled(true);
        Self {
            text: Text::new("", &res.font, 16),
            sprite: Sprite::with_texture(&res.sprite_sheet),
            rw,
        }
    }
}

pub fn run(current_date: NaiveDate, user_data: &mut UserData) {
    let mut t: f32 = 0.;
    let res = Resources::load();
    let mut render_ctx = RenderContext::with_resources(&res);
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
        draw_calendar(&mut render_ctx, current_date, &user_data, &ui_state);
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

fn draw_rect_with_text(
    render_ctx: &mut RenderContext,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    string: &str,
    highlighted: bool,
) {
    let mut rs = RectangleShape::new();
    render_ctx.text.set_fill_color(if highlighted {
        Color::WHITE
    } else {
        Color::BLACK
    });
    rs.set_outline_color(if highlighted {
        Color::WHITE
    } else {
        Color::BLACK
    });
    rs.set_outline_thickness(1.0);
    rs.set_fill_color(if highlighted {
        Color::BLACK
    } else {
        Color::WHITE
    });
    rs.set_position((x, y));
    rs.set_size((w, h));
    render_ctx.rw.draw(&rs);
    draw_text_wrapped(render_ctx, string, x, y, w, h);
}

fn draw_text_wrapped(render_ctx: &mut RenderContext, string: &str, x: f32, y: f32, w: f32, h: f32) {
    render_ctx.text.set_string(string);
    let mut bounds = render_ctx.text.global_bounds();
    let mut vert_div = 3.0;
    if bounds.width > w {
        let half = string.len() / 2;
        if let Some(pos) = string[half..].find(' ') {
            let next_word = pos + half + 1;
            let halved = &string[next_word..];
            draw_text_wrapped(render_ctx, halved, x, y + 12.0, w, h);
            render_ctx.text.set_string(&string[..next_word]);
            bounds = render_ctx.text.global_bounds();
            vert_div = 8.0
        }
    }
    let remaining_space = w - bounds.width;
    let horiz_offset = remaining_space / 2.0;
    let remaining_y = h - bounds.height;
    let vert_offset = remaining_y / vert_div;
    render_ctx
        .text
        .set_position((x + horiz_offset, y + vert_offset));
    render_ctx.rw.draw(&render_ctx.text);
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
    fn draw(&self, render_ctx: &mut RenderContext, user_data: &UserData, ui_state: &UiState) {
        for button in &self.buttons {
            button.draw(render_ctx, user_data, &ui_state);
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

impl Button {
    fn draw(&self, render_ctx: &mut RenderContext, user_data: &UserData, ui_state: &UiState) {
        if self.hidden {
            return;
        }
        use ButtonId::*;
        match self.kind {
            ButtonKind::RectWithText => {
                let string = match self.id {
                    CurrentActivity => {
                        &user_data.activities[ui_state.current_activity as usize].name
                    }
                    Overview => {
                        if ui_state.overview {
                            "Back"
                        } else {
                            "Overview"
                        }
                    }
                    ButtonId::SetStartingDate => {
                        if matches!(ui_state.imode, InteractMode::StartingDateSelect) {
                            "Cancel"
                        } else {
                            "Set starting date"
                        }
                    }
                    EditMode => "Edit mode",
                    _ => panic!("Unknown text button"),
                };
                draw_rect_with_text(
                    render_ctx,
                    self.rect.left,
                    self.rect.top,
                    self.rect.width,
                    self.rect.height,
                    string,
                    self.highlighted,
                );
            }
            ButtonKind::Sprite => {
                render_ctx
                    .sprite
                    .set_position((self.rect.left, self.rect.top));
                let sprite_offset = match self.id {
                    PrevActivity => 4 * 24,
                    AddActivity => 2 * 24,
                    RemActivity => 3 * 24,
                    NextActivity => 5 * 24,
                    _ => panic!("Unknown sprite button"),
                };
                render_ctx
                    .sprite
                    .set_texture_rect(&IntRect::new(sprite_offset, 0, 24, 24));
                render_ctx.rw.draw(&render_ctx.sprite);
            }
        }
    }
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
