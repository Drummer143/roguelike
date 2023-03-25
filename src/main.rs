mod map;
mod unit;

use std::{cmp::min, process::Command};

use rand::Rng;
use tcod::{
    colors,
    console::{blit, Offscreen, Root},
    input::{Key, KeyCode},
    Console, FontLayout, FontType,
};

use map::{Map, Tile};
use unit::{Coordinates, Unit};

const WIDTH: i32 = 100;
const HEIGHT: i32 = 100;

const FPS: i32 = 60;

const ROOM_MAX_SIZE: i32 = 10;
const ROOM_MIN_SIZE: i32 = 6;
const MAX_ROOMS: i32 = 30;

const MAX_ROOM_MONSTERS: i32 = 3;

const PLAYER: usize = 0;

pub struct Game {
    pub map: Map,
}

struct App {
    root: Root,
    offscreen: Offscreen,
    game: Game,
}

#[derive(Debug, Clone)]
struct Room {
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

fn restart() {
    let path_to_app = std::env::current_exe();

    if let Ok(path_to_app) = path_to_app {
        Command::new(path_to_app)
            .spawn()
            .expect("failed to restart process");
        std::process::exit(0);
    } else {
        panic!("failed to restart process");
    }
}

fn handle_keys(app: &mut App, player: &mut Unit) -> bool {
    let key = app.root.wait_for_keypress(true);

    match key {
        Key {
            code: KeyCode::Char,
            printable: 'w',
            ..
        } => player.r#move(0, -1, &app.game.map),

        Key {
            code: KeyCode::Char,
            printable: 's',
            ..
        } => player.r#move(0, 1, &app.game.map),

        Key {
            code: KeyCode::Char,
            printable: 'a',
            ..
        } => player.r#move(-1, 0, &app.game.map),

        Key {
            code: KeyCode::Char,
            printable: 'd',
            ..
        } => player.r#move(1, 0, &app.game.map),

        Key {
            code: KeyCode::Enter,
            alt: true,
            ..
        } => {
            let fullscreen = app.root.is_fullscreen();
            app.root.set_fullscreen(!fullscreen);
        }

        Key {
            code: KeyCode::Escape,
            ..
        } => {
            std::process::exit(0);
        }

        Key {
            code: KeyCode::Char,
            printable: 'r',
            ..
        } => {
            restart();
            // main();
            // std::process::exit(1);
        }

        _ => {}
    }

    false
}

fn render_all(app: &mut App, units: &Vec<Unit>) {
    // draw all objects in the list
    for unit in units.into_iter().rev() {
        unit.draw(&mut app.offscreen);
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

// fn coonect_rooms(rooms: &Vec<Room>, map: &mut Map) {
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

fn main() {
    tcod::system::set_fps(FPS);

    let root = Root::initializer()
        .font("arial10x10.png", FontLayout::Tcod)
        .font_type(FontType::Greyscale)
        .size(WIDTH, HEIGHT)
        .title("Roguelike game")
        .init();

    let mut app = App {
        root,
        offscreen: Offscreen::new(WIDTH, HEIGHT),
        game: Game {
            map: Map::new(WIDTH, HEIGHT),
        },
    };

    let (rooms, monsters) = generate_rooms(&mut app.game.map);

    let spawn_position = rooms[0].get_center();
    let player = Unit::new(spawn_position.x, spawn_position.y, '@', colors::WHITE, "Player", true, true);

    let mut units = vec![player];

    units.extend(monsters);

    app.game.map.set_fov();

    loop {
        app.offscreen.set_default_background(colors::BLACK);
        app.offscreen.clear();

        render_all(&mut app, &units);

        app.game
            .map
            .render(&mut app.offscreen, &units[PLAYER].get_position());

        blit(
            &app.offscreen,
            (0, 0),
            (WIDTH, HEIGHT),
            &mut app.root,
            (0, 0),
            1.0,
            1.0,
        );

        app.root.flush();

        let exit = handle_keys(&mut app, &mut units[PLAYER]);

        if app.root.window_closed() || exit {
            break;
        }
    }
}
