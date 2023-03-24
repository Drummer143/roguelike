use tcod::{console::Offscreen, BackgroundFlag, Color, Console};

pub const COLOR_DARK_WALL: Color = Color { r: 0, g: 0, b: 100 };
pub const COLOR_DARK_GROUND: Color = Color {
    r: 50,
    g: 50,
    b: 150,
};

#[derive(Debug, Clone, Copy)]
pub struct Tile {
    blocked: bool,
    block_sight: bool,
}

impl Tile {
    pub fn empty() -> Self {
        Tile {
            blocked: false,
            block_sight: false,
        }
    }

    pub fn wall() -> Self {
        Tile {
            blocked: true,
            block_sight: true,
        }
    }

    pub fn is_blocked(self) -> bool {
        self.blocked
    }

    pub fn is_block_sight(self) -> bool {
        self.block_sight
    }
}

#[derive(Debug, Clone)]
pub struct Map {
    tiles: Vec<Vec<Tile>>,
    width: i32,
    height: i32,
}

impl Map {
    pub fn new(width: i32, height: i32) -> Self {
        let tiles = vec![vec![Tile::empty(); width as usize]; height as usize];

        Map {
            tiles,
            width,
            height,
        }
    }

    pub fn set_tile(&mut self, x: i32, y: i32, tile: Tile) -> bool {
        if x > self.width || x < 0 || y > self.height || y < 0 {
            return false;
        }

        self.tiles[x as usize][y as usize] = tile;

        true
    }

    pub fn get_tile(&self, x: i32, y: i32) -> Result<Tile, String> {
        if x > self.width || x < 0 || y > self.height || y < 0 {
            Err("Invalid coordinates".into())
        } else {
            Ok(self.tiles[x as usize][y as usize])
        }
    }

    pub fn render(&self, offscreen: &mut Offscreen) {
        for y in 0..self.height {
            for x in 0..self.width {
                let wall = self.tiles[x as usize][y as usize].block_sight;
                if wall {
                    offscreen.set_char_background(x, y, COLOR_DARK_WALL, BackgroundFlag::Set);
                } else {
                    offscreen.set_char_background(x, y, COLOR_DARK_GROUND, BackgroundFlag::Set);
                }
            }
        }
    }

    pub fn get_width(&self) -> i32 {
        self.width
    }

    pub fn get_height(&self) -> i32 {
        self.height
    }
}
