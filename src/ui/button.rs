use sfml::graphics::Rect;

pub enum Id {
    CurrentActivity,
    PrevActivity,
    AddActivity,
    RemActivity,
    NextActivity,
    Overview,
    SetStartingDate,
    EditMode,
}

pub struct Button {
    pub rect: Rect<f32>,
    pub id: Id,
    pub kind: Kind,
    pub hidden: bool,
    pub highlighted: bool,
}

pub enum Kind {
    RectWithText,
    Sprite,
}

macro_rules! buttons {
    ($($x:expr, $y:expr, $w:expr, $h:expr, $id:ident, $kind:ident),+) => {
        vec![
            $(Button {
                rect: Rect::new($x as f32, $y as f32, $w as f32, $h as f32),
                id: Id::$id,
                kind: Kind::$kind,
                hidden: false,
                highlighted: false,
            }),+
        ]
    }
}

pub fn buttons() -> Vec<Button> {
    buttons! {
    //  x     y              w    h   id               kind
        904,  4,            178, 42, CurrentActivity, RectWithText,
        934, 52,             24, 24, PrevActivity,    Sprite,
        964, 52,             24, 24, AddActivity,     Sprite,
        994, 52,             24, 24, RemActivity,     Sprite,
       1024, 52,             24, 24, NextActivity,    Sprite,
        904, 82,            178, 32, Overview,        RectWithText,
        904, 82 + 42,       178, 32, SetStartingDate, RectWithText,
        904, 82 + (2 * 42), 178, 32, EditMode,        RectWithText
    }
}
