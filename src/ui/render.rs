use super::{button, color, layout::*, names::*, InteractMode, SideUi, UiState};
use crate::{date_util, UserData};
use chrono::prelude::*;
use sfml::{graphics::*, system::SfBox, window::*};
use std::error::Error;

pub struct Resources {
    font: SfBox<Font>,
    sprite_sheet: SfBox<Texture>,
}

impl Resources {
    pub fn load() -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            font: Font::from_memory(include_bytes!("../../DejaVuSansMono.ttf"))
                .ok_or("Failed to create font")?,
            sprite_sheet: Texture::from_memory(
                include_bytes!("../../graphics.png"),
                &IntRect::default(),
            )
            .ok_or("Failed to create sprite sheet")?,
        })
    }
}

pub struct RenderContext<'res> {
    text: Text<'res>,
    sprite: Sprite<'res>,
    pub rw: RenderWindow,
}

impl<'res> RenderContext<'res> {
    pub fn with_resources(res: &'res Resources) -> Self {
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

fn draw_text(render_ctx: &mut RenderContext, x: i16, y: i16, string: &str) {
    render_ctx.text.set_position((x.into(), y.into()));
    render_ctx.text.set_string(string);
    render_ctx.rw.draw(&render_ctx.text);
}

pub(super) fn draw_calendar(
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

impl button::Button {
    fn draw(&self, render_ctx: &mut RenderContext, user_data: &UserData, ui_state: &UiState) {
        if self.hidden {
            return;
        }
        use button::{Id::*, Kind::*};
        match self.kind {
            RectWithText => {
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
                    SetStartingDate => {
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
            Sprite => {
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

impl SideUi {
    pub fn draw(&self, render_ctx: &mut RenderContext, user_data: &UserData, ui_state: &UiState) {
        for button in &self.buttons {
            button.draw(render_ctx, user_data, &ui_state);
        }
        //let current_streak_txt = format!("Current streak:", Fancy(ui_state.current_streak));
        //let longest_streak_txt = format!("Longest streak:", Fancy(ui_state.longest_streak));
        let mut rs = RectangleShape::new();
        rs.set_fill_color(Color::rgba(0, 0, 0, 180));
        rs.set_position((904., 300.));
        rs.set_size((180.0, 100.0));
        render_ctx.rw.draw(&rs);
        render_ctx.text.set_fill_color(Color::rgb(255, 255, 255));
        render_ctx.text.set_position((908., 300.));
        render_ctx.text.set_string("Current streak:");
        render_ctx.rw.draw(&render_ctx.text);
        render_ctx.text.set_position((908., 320.));
        render_ctx
            .text
            .set_string(&format!("{}", Fancy(ui_state.current_streak)));
        render_ctx.rw.draw(&render_ctx.text);
        render_ctx.text.set_position((908., 340.));
        render_ctx.text.set_string("Longest streak:");
        render_ctx.rw.draw(&render_ctx.text);
        render_ctx.text.set_position((908., 360.));
        render_ctx
            .text
            .set_string(&format!("{}", Fancy(ui_state.longest_streak)));
        render_ctx.rw.draw(&render_ctx.text);
    }
}

struct Fancy(u32);

impl std::fmt::Display for Fancy {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let days = self.0;
        if days == 0 {
            write!(f, "-")?;
            return Ok(());
        }
        const DAYS_PER_YEAR: u16 = 365;
        let years = days / u32::from(DAYS_PER_YEAR);
        if years > 0 {
            write!(f, "{} year{} ", years, if years == 1 { "" } else { "s" })?;
        }
        let rem_days = days % u32::from(DAYS_PER_YEAR);
        if rem_days > 0 {
            write!(
                f,
                "{} day{}",
                rem_days,
                if rem_days == 1 { "" } else { "s" }
            )?;
        }
        Ok(())
    }
}
