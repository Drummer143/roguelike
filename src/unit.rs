use crate::map::Map;
use tcod::{colors, BackgroundFlag, Color, Console};

#[derive(Clone, Copy, Debug)]
pub struct Coordinates {
    pub x: i32,
    pub y: i32,
}

impl Coordinates {
    pub fn is_equal(&self, point: &Coordinates) -> bool {
        self.x == point.x && self.y == point.y
    }
}

#[derive(Clone, Debug)]
pub struct Unit {
    position: Coordinates,
    char: char,
    color: Color,
    name: String,
    blocks_point: bool,
    alive: bool
}

impl Unit {
    pub fn new(x: i32, y: i32, char: char, color: Color, name: &str, blocks_point: bool, alive: bool) -> Self {
        Unit {
            position: Coordinates { x, y },
            char,
            color,
            alive,
            blocks_point,
            name: String::from(name)
        }
    }

    /// move by the given amount
    pub fn r#move(&mut self, x: i32, y: i32, map: &Map) {
        let next_x = self.position.x + x;
        let next_y = self.position.y + y;

        let is_map_end = next_x < 0
            || next_x > map.get_width() - 1
            || next_y < 0
            || next_y > map.get_height() - 1;

        if is_map_end {
            return;
        }

        let tile = map.get_tile(next_x, next_y);

        if tile.is_ok() && !tile.unwrap().is_blocked() {
            self.position.x += x;
            self.position.y += y;
        }
    }

    /// set the color and then draw the character that represents this object at its position
    pub fn draw(&self, con: &mut dyn Console) {
        con.set_default_foreground(self.color);
        con.put_char(
            self.position.x,
            self.position.y,
            self.char,
            BackgroundFlag::None,
        );
    }

    pub fn get_position(&self) -> &Coordinates {
        &self.position
    }
}

impl Unit {
    pub fn orc(x: i32, y: i32) -> Self {
        Self {
            position: Coordinates { x, y },
            char: 'o',
            color: colors::DESATURATED_GREEN,
            alive: true,
            blocks_point: true,
            name: "Orc".into()
        }
    }

    pub fn troll(x: i32, y: i32) -> Self {
        Self {
            position: Coordinates { x, y },
            char: 'T',
            color: colors::DARK_GREEN,
            alive: true,
            blocks_point: true,
            name: "Troll".into()
        }
    }
}
