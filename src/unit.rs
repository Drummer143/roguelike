use tcod::{colors, BackgroundFlag, Color, Console};

#[derive(PartialEq)]
pub enum UserActions {
    TookTurn,
    DidNotTakeTurn,
    Exit,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UnitActions {
    Attack,
    Move,
    AFK,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AI {
    Basic,
    Player,
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UnitStats {
    max_hp: i32,
    current_hp: i32,
    defense: i32,
    damage: i32,
}

#[derive(Clone, Debug)]
pub struct Unit {
    position: Coordinates,
    char: char,
    color: Color,
    name: String,
    blocks_point: bool,
    alive: bool,
    spawn_room: u32,
    stats: UnitStats,
    ai: AI,
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
        spawn_room: u32,
        ai: AI,
        max_hp: i32,
        current_hp: i32,
        defense: i32,
        damage: i32,
    ) -> Self {
        Unit {
            position: Coordinates { x, y },
            char,
            color,
            alive,
            blocks_point,
            name: String::from(name),
            spawn_room,
            stats: UnitStats {
                max_hp,
                current_hp,
                defense,
                damage,
            },
            ai,
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

    pub fn monster_step(&self, target: &Coordinates) -> (f32, i32, i32) {
        // vector from this object to the target, and distance
        let dx = target.x - self.position.x;
        let dy = target.y - self.position.y;
        let distance = ((dx.pow(2) + dy.pow(2)) as f32).sqrt();

        // normalize it to length 1 (preserving direction), then round it and
        // convert to integer so the movement is restricted to the map grid
        let dx = (dx as f32 / distance).round() as i32;
        let dy = (dy as f32 / distance).round() as i32;

        (distance, dx, dy)
    }

    pub fn take_damage(&mut self, damage: i32) {
        self.stats.current_hp -= damage;

        if self.stats.current_hp <= 0 {
            self.alive = false;
            self.blocks_point = false;

            println!("{} is dead", self.name);

            self.color = colors::GREY;
        }
    }

    pub fn attack(&self, target: &mut Unit) {
        let damage = self.stats.damage - target.defense();

        if damage > 0 {
            target.take_damage(damage);

            println!("{} got {} damage from {}", target.name(), damage, self.name);
        } else if self.stats.damage <= 0 {
            println!(
                "{} tries to attack {} but it is too weak to deal damage",
                self.name,
                target.name()
            );
        } else {
            println!(
                "{} attacks {} but all damage was absorbed by armor",
                self.name,
                target.name()
            );
        }
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

    pub fn defense(&self) -> i32 {
        self.stats.defense
    }

    pub fn max_hp(&self) -> i32 {
        self.stats.max_hp
    }

    pub fn current_hp(&self) -> i32 {
        self.stats.current_hp
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
            spawn_room: 0,
            ai: AI::Player,
            stats: UnitStats {
                max_hp: 30,
                current_hp: 30,
                defense: 2,
                damage: 5,
            },
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
            spawn_room,
            ai: AI::Basic,
            stats: UnitStats {
                max_hp: 10,
                current_hp: 10,
                defense: 0,
                damage: 3,
            },
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
            spawn_room,
            ai: AI::Basic,
            stats: UnitStats {
                max_hp: 16,
                current_hp: 16,
                defense: 1,
                damage: 4,
            },
        }
    }
}
