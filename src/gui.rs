use tcod::{console::Offscreen, Console};

pub struct GUI {
    offscreen: Offscreen,
    width: i32,
    height: i32,
}

impl GUI {
    pub fn new(width: i32, height: i32) -> Self {
        Self {
            offscreen: Offscreen::new(width, height),
            width,
            height,
        }
    }
}
