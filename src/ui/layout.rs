use crate::date_util::{DAYS_PER_WEEK, MONTHS_PER_YEAR};

pub const DAYBOX_SIZE: u8 = 24;
pub const DAYBOX_PADDING: u8 = 6;
pub const RES: (u16, u16) = (1088, 720);
pub const CALENDAR_SIZE: (u16, u16) = (
    RES.0 - MONTH_BOX_MARGIN as u16 / 2,
    RES.1 - MONTH_BOX_MARGIN as u16 / 2,
);
pub const MONTHS_PER_ROW: u8 = 4;
pub const MONTHS_PER_COLUMN: u8 = MONTHS_PER_YEAR / MONTHS_PER_ROW;
pub const MONTH_BOX_SIZE: (u16, u16) = (
    MONTH_BOX_PADDING as u16 + (DAYBOX_SIZE as u16 + DAYBOX_PADDING as u16) * DAYS_PER_WEEK as u16,
    (CALENDAR_SIZE.1 / MONTHS_PER_COLUMN as u16)
        - MONTH_BOX_MARGIN as u16
        - MONTH_BOX_PADDING as u16,
);
/// Internal padding between box and content
pub const MONTH_BOX_PADDING: u8 = DAYBOX_PADDING;
/// External margin between boxes
pub const MONTH_BOX_MARGIN: u8 = DAYBOX_PADDING / 2;
// Always the current and the next month are the last 2 months displayed.
pub const CURRENT_MONTH_OFFSET: u8 = 10;

pub fn month_box_pixel_position(month: u8) -> (f32, f32) {
    // The "grid positioning" of the boxes, rougher than the pixel.
    let (gx, gy) = (month % MONTHS_PER_ROW, month / MONTHS_PER_ROW);
    // The pixel positioning of where the boxes will be drawn
    let x = f32::from(MONTH_BOX_MARGIN)
        + (f32::from(gx)
            * (f32::from(MONTH_BOX_SIZE.0)
                + f32::from(MONTH_BOX_PADDING)
                + f32::from(MONTH_BOX_MARGIN)));
    let y = f32::from(MONTH_BOX_MARGIN)
        + (f32::from(gy)
            * (f32::from(MONTH_BOX_SIZE.1)
                + f32::from(MONTH_BOX_PADDING)
                + f32::from(MONTH_BOX_MARGIN)));
    (x, y)
}
