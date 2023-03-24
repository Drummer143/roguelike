use tcod::{
    console::Offscreen,
    map::{FovAlgorithm, Map as FovMap},
    BackgroundFlag, Color, Console,
};

use crate::unit::Coordinates;

const FOV_ALGO: FovAlgorithm = FovAlgorithm::Basic; // default FOV algorithm
const FOV_LIGHT_WALLS: bool = true; // light walls or not
const TORCH_RADIUS: i32 = 10;

const COLOR_DARK_WALL: Color = Color { r: 0, g: 0, b: 100 };
const COLOR_LIGHT_WALL: Color = Color {
    r: 130,
    g: 110,
    b: 50,
};
const COLOR_DARK_GROUND: Color = Color {
    r: 50,
    g: 50,
    b: 150,
};
const COLOR_LIGHT_GROUND: Color = Color {
    r: 200,
    g: 180,
    b: 50,
};

#[derive(Debug, Clone, Copy)]
pub struct Tile {
    blocked: bool,
    explored: bool,
    block_sight: bool,
}

impl Tile {
    pub fn empty() -> Self {
        Tile {
            blocked: false,
            explored: false,
            block_sight: false,
        }
    }

    pub fn wall() -> Self {
        Tile {
            blocked: true,
            explored: false,
            block_sight: true,
        }
    }

    pub fn set_explored(&mut self, value: bool) {
        self.explored = value;
    }

    pub fn is_blocked(self) -> bool {
        self.blocked
    }

    pub fn is_block_sight(self) -> bool {
        self.block_sight
    }

    pub fn is_explored(self) -> bool {
        self.explored
    }
}

#[derive(Clone)]
pub struct Map {
    tiles: Vec<Vec<Tile>>,
    width: i32,
    height: i32,
    fov: FovMap,
    prev_player_pos: Coordinates,
}

impl Map {
    pub fn new(width: i32, height: i32) -> Self {
        let tiles = vec![vec![Tile::wall(); width as usize]; height as usize];

        Map {
            tiles,
            width,
            height,
            fov: FovMap::new(width, height),
            prev_player_pos: Coordinates { x: 0, y: 0 },
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

    pub fn set_fov(&mut self) {
        for y in 0..self.height {
            for x in 0..self.height {
                let tile = self.tiles[x as usize][y as usize];

                self.fov
                    .set(x, y, !tile.is_block_sight(), !tile.is_blocked());
            }
        }
    }

    pub fn render(&mut self, offscreen: &mut Offscreen, player_position: &Coordinates) {
        if !player_position.is_equal(&self.prev_player_pos) {
            self.prev_player_pos.x = player_position.x;
            self.prev_player_pos.y = player_position.y;

            self.fov.compute_fov(
                player_position.x,
                player_position.y,
                TORCH_RADIUS,
                FOV_LIGHT_WALLS,
                FOV_ALGO,
            );
        }

        for y in 0..self.height {
            for x in 0..self.width {
                let visible = self.fov.is_in_fov(x, y);
                let tile = &mut self.tiles[x as usize][y as usize];

                if visible {
                    tile.set_explored(true);
                }

                if tile.is_explored() {
                    let color = match (visible, tile.blocked) {
                        // outside of field of view:
                        (false, true) => COLOR_DARK_WALL,
                        (false, false) => COLOR_DARK_GROUND,
                        // inside fov:
                        (true, true) => COLOR_LIGHT_WALL,
                        (true, false) => COLOR_LIGHT_GROUND,
                    };

                    offscreen.set_char_background(x, y, color, BackgroundFlag::Set);
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
