use tcod::{colors, BackgroundFlag, Color, Console};

#[derive(PartialEq)]
pub enum UserActions {
    TookTurn,
    DidNotTakeTurn,
    Exit
}

pub enum UnitActions {
    Attack,
    Move,
    AFK
}

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
    alive: bool,
    spawn_room: u32
}

impl Unit {
    pub fn new(
        x: i32,
        y: i32,
        char: char,
        color: Color,
        name: &str,
        blocks_point: bool,
        alive: bool,
        spawn_room: u32
    ) -> Self {
        Unit {
            position: Coordinates { x, y },
            char,
            color,
            alive,
            blocks_point,
            name: String::from(name),
            spawn_room
        }
    }

    /// move by the given amount
    pub fn r#move(&mut self, x: i32, y: i32) {
            self.position.x += x;
            self.position.y += y;
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

    pub fn is_blocks_point(&self) -> bool {
        self.blocks_point
    }

    pub fn is_alive(&self) -> bool {
        self.alive
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn spawn_room(&self) -> u32 {
        self.spawn_room
    }
}

impl Unit {
    pub fn player(x: i32, y: i32) -> Self {
        Self {
            position: Coordinates { x, y },
            char: '@',
            color: colors::WHITE,
            name: "Player".into(),
            blocks_point: true,
            alive: true,
            spawn_room: 0
        }
    }

    pub fn orc(x: i32, y: i32, spawn_room: u32) -> Self {
        Self {
            position: Coordinates { x, y },
            char: 'o',
            color: colors::DESATURATED_GREEN,
            alive: true,
            blocks_point: true,
            name: "Orc".into(),
            spawn_room
        }
    }

    pub fn troll(x: i32, y: i32, spawn_room: u32) -> Self {
        Self {
            position: Coordinates { x, y },
            char: 'T',
            color: colors::DARK_GREEN,
            alive: true,
            blocks_point: true,
            name: "Troll".into(),
            spawn_room
        }
    }
}
