use std::cmp::min;

use rand::Rng;
use tcod::{
    console::Offscreen,
    map::{FovAlgorithm, Map as FovMap},
    BackgroundFlag, Color, Console,
};

use crate::unit::{Coordinates, Unit};

const FOV_ALGO: FovAlgorithm = FovAlgorithm::Basic; // default FOV algorithm
const FOV_LIGHT_WALLS: bool = true; // light walls or not
const TORCH_RADIUS: i32 = 10;

const ROOM_MAX_SIZE: i32 = 10;
const ROOM_MIN_SIZE: i32 = 6;
const MAX_ROOMS: i32 = 30;

const MAX_ROOM_MONSTERS: i32 = 3;

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

#[derive(Debug, Clone)]
pub struct Room {
    pub left_x: i32,
    pub top_y: i32,
    pub right_x: i32,
    pub bottom_y: i32,
    pub monsters: Vec<Unit>,
}

impl Room {
    pub fn new(left_x: i32, right_x: i32, bottom_y: i32, top_y: i32) -> Self {
        Self {
            left_x,
            right_x,
            top_y,
            bottom_y,
            monsters: vec![],
        }
    }

    pub fn fill(&self, map: &mut Map) {
        for x in (self.left_x)..(self.right_x) {
            for y in (self.bottom_y)..(self.top_y) {
                map.set_tile(x, y, Tile::empty());
            }
        }
    }

    pub fn get_center(&self) -> Coordinates {
        let x = (self.left_x + self.right_x) / 2;
        let y = (self.top_y + self.bottom_y) / 2;

        Coordinates { x, y }
    }

    pub fn intersects_with_as_tunnels(&self, room: &Room) -> bool {
        (self.left_x < room.right_x)
            && (self.right_x > room.left_x)
            && (self.top_y > room.bottom_y)
            && (self.bottom_y < room.top_y)
    }

    pub fn intersects_with_as_rooms(&self, room: &Room) -> bool {
        (self.left_x <= room.right_x)
            && (self.right_x >= room.left_x)
            && (self.top_y >= room.bottom_y)
            && (self.bottom_y <= room.top_y)
    }
}

fn h_v_tunnel(new_center: &Coordinates, prev_center: &Coordinates, map: &mut Map) {
    let (bottom_y, top_y) = (prev_center.y, prev_center.y + 1);
    let (left_x, right_x) = if new_center.x < prev_center.x {
        (new_center.x, prev_center.x)
    } else {
        (prev_center.x, new_center.x)
    };

    let h_tunnel = Room::new(left_x, right_x, bottom_y, top_y);
    h_tunnel.fill(map);

    let (left_x, right_x) = (new_center.x, new_center.x + 1);
    let (bottom_y, top_y) = if new_center.y < prev_center.y {
        (new_center.y, prev_center.y)
    } else {
        (prev_center.y, new_center.y)
    };

    let v_tunnel = Room::new(left_x, right_x, bottom_y, top_y);
    v_tunnel.fill(map);

    if !h_tunnel.intersects_with_as_tunnels(&v_tunnel) {
        let v_tunnel = Room::new(left_x, right_x, bottom_y, top_y + 1);
        v_tunnel.fill(map);
    }
}

fn v_h_tunnel(new_center: &Coordinates, prev_center: &Coordinates, map: &mut Map) {
    let (left_x, right_x) = (prev_center.x, prev_center.x + 1);
    let (bottom_y, top_y) = if new_center.y < prev_center.y {
        (new_center.y, prev_center.y)
    } else {
        (prev_center.y, new_center.y)
    };

    let v_tunnel = Room::new(left_x, right_x, bottom_y, top_y);
    v_tunnel.fill(map);

    let (bottom_y, top_y) = (new_center.y, new_center.y + 1);
    let (left_x, right_x) = if new_center.x < prev_center.x {
        (new_center.x, prev_center.x)
    } else {
        (prev_center.x, new_center.x)
    };

    let h_tunnel = Room::new(left_x, right_x, bottom_y, top_y);
    h_tunnel.fill(map);

    if !h_tunnel.intersects_with_as_tunnels(&v_tunnel) {
        let h_tunnel = Room::new(left_x, right_x + 1, bottom_y, top_y);
        h_tunnel.fill(map);
    }
}

fn distance_diff(room1: &Room, room2: &Room) -> (i32, i32) {
    let rlx = room1.right_x - room2.left_x;
    let lrx = room1.left_x - room2.right_x;
    let tby = room1.top_y - room2.bottom_y;
    let bty = room1.bottom_y - room2.top_y;

    let w = min(rlx, lrx);
    let h = min(tby, bty);

    (h.abs(), w.abs())
}

fn find_nearest_room<'a>(rooms: &'a Vec<Room>, target_room: &'a Room) -> &'a Room {
    let mut nearest = &rooms[0];
    let (h, w) = distance_diff(target_room, &nearest);

    for i in 1..rooms.len() {
        let (nh, nw) = distance_diff(target_room, &rooms[i]);

        if nh < h || nw < w {
            nearest = &rooms[i];
        }
    }

    nearest
}

// fn connect_rooms(rooms: &Vec<Room>, map: &mut Map) {
//     for i in 1..rooms.len() {
//         let slice = &rooms[i..].to_vec();

//         let nearest = find_nearest_room(slice, &rooms[i - 1]);

//         if rand::random() {
//             v_h_tunnel(&rooms[i - 1].get_center(), &nearest.get_center(), map);
//         } else {
//             h_v_tunnel(&rooms[i].get_center(), &nearest.get_center(), map);
//         }
//     }
// }

fn spawn_monsters(room: &mut Room) -> Vec<Unit> {
    let mut monsters: Vec<Unit> = vec![];

    let count_of_monsters_in_room = rand::thread_rng().gen_range(0..MAX_ROOM_MONSTERS + 1);
    let mut index = 0;

    while index < count_of_monsters_in_room {
        let x = rand::thread_rng().gen_range(room.left_x..room.right_x);
        let y = rand::thread_rng().gen_range(room.bottom_y..room.top_y);

        let mut is_place_taken = false;

        for monster in room.monsters.iter() {
            if monster.get_position().is_equal(&Coordinates { x, y }) {
                is_place_taken = true;
                break;
            }
        }

        if !is_place_taken {
            index += 1;

            let new_monster = if rand::random::<f32>() < 0.8 {
                Unit::orc(x, y)
            } else {
                Unit::troll(x, y)
            };

            // room.monsters.push(new_monster);
            monsters.push(new_monster);
        }
    }

    monsters
}

fn generate_rooms(map: &mut Map) -> (Vec<Room>, Vec<Unit>) {
    let mut index = 0;
    let mut rooms: Vec<Room> = vec![];
    let mut monsters: Vec<Unit> = vec![];

    while index < MAX_ROOMS {
        let w = rand::thread_rng().gen_range(ROOM_MIN_SIZE..ROOM_MAX_SIZE + 1);
        let h = rand::thread_rng().gen_range(ROOM_MIN_SIZE..ROOM_MAX_SIZE + 1);
        let x = rand::thread_rng().gen_range(1..map.get_height() - w - 1);
        let y = rand::thread_rng().gen_range(1..map.get_width() - h - 1);

        let mut new_room = Room::new(x, x + w, y, y + h);

        let intersects = rooms
            .iter()
            .any(|room| room.intersects_with_as_rooms(&new_room));

        if intersects {
            continue;
        }

        if !rooms.is_empty() {
            let nearest = find_nearest_room(&rooms, &new_room);

            if rand::random() {
                v_h_tunnel(&new_room.get_center(), &nearest.get_center(), map);
            } else {
                h_v_tunnel(&new_room.get_center(), &nearest.get_center(), map);
            }

            let monsters_in_room = spawn_monsters(&mut new_room);

            monsters.extend(monsters_in_room);
        }

        new_room.fill(map);

        rooms.push(new_room);

        index += 1;
    }

    (rooms, monsters)
}

#[derive(Clone)]
pub struct Map {
    tiles: Vec<Vec<Tile>>,
    width: i32,
    height: i32,
    fov: FovMap,
    prev_player_pos: Coordinates,
    rooms: Vec<Room>,
    monsters: Vec<Unit>,
    player: Unit,
}

impl Map {
    pub fn new(width: i32, height: i32) -> Self {
        let tiles = vec![vec![Tile::wall(); width as usize]; height as usize];

        let mut map = Self {
            tiles,
            width,
            height,
            fov: FovMap::new(width, height),
            prev_player_pos: Coordinates { x: 0, y: 0 },
            rooms: vec![],
            monsters: vec![],
            player: Unit::player(0, 0),
        };

        let (rooms, monsters) = generate_rooms(&mut map);

        map.rooms.extend(rooms);
        map.monsters.extend(monsters);

        let spawn_point = map.get_spawn_point();

        let player = Unit::player(spawn_point.x, spawn_point.y);

        map.player = player;

        map
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

    pub fn render(&mut self, offscreen: &mut Offscreen) {
        let player_position = self.player.get_position();

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

        for monster in &self.monsters {
            monster.draw(offscreen);
        }

        self.player.draw(offscreen);
    }

    pub fn get_player(&mut self) -> &mut Unit {
        &mut self.player
    }

    pub fn get_spawn_point(&self) -> Coordinates {
        self.rooms[0].get_center()
    }

    pub fn get_width(&self) -> i32 {
        self.width
    }

    pub fn get_height(&self) -> i32 {
        self.height
    }

    pub fn can_move_at(&self, x: i32, y: i32) -> bool {
        let is_map_end = x < 0 || x > self.width - 1 || y < 0 || y > self.height - 1;

        if is_map_end {
            return false;
        }

        let tile = self.get_tile(x, y);

        let is_tile_blocked = if let Ok(tile) = tile {
            tile.blocked
        } else {
            false
        };

        if is_tile_blocked {
            return false;
        }

        let is_monster_here = self.monsters.iter().any(|monster| {
            let pos = monster.get_position();

            monster.is_blocks_point() && x == pos.x && y == pos.y
        });

        !is_monster_here
    }

    pub fn move_player(&mut self, x: i32, y: i32) {
        let next_x = self.player.get_position().x + x;
        let next_y = self.player.get_position().y + y;

        if self.can_move_at(next_x, next_y) {
            self.player.r#move(x, y);
        }
    }
}
