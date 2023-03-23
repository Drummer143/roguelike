mod map;

use map::{Map, Tile};

use tcod::{
    colors,
    console::{blit, Offscreen, Root},
    input::{Key, KeyCode},
    BackgroundFlag, Color, Console, FontLayout, FontType,
};

const WIDTH: i32 = 100;
const HEIGHT: i32 = 100;

const MAP_WIDTH: i32 = 100;
const MAP_HEIGHT: i32 = 100;

const FPS: i32 = 60;

struct App {
    root: Root,
    offscreen: Offscreen,
}

#[derive(Clone, Copy)]
struct Coordinates {
    x: i32,
    y: i32,
}

#[derive(Clone, Copy)]
struct Unit {
    position: Coordinates,
    char: char,
    color: Color,
}

impl Unit {
    pub fn new(x: i32, y: i32, char: char, color: Color) -> Self {
        Unit {
            position: Coordinates { x, y },
            char,
            color,
        }
    }

    /// move by the given amount
    pub fn r#move(&mut self, x: i32, y: i32, game: &Game) {
        let next_x = self.position.x + x;
        let next_y = self.position.y + y;

        let is_map_end =
            next_x < 0 || next_x > MAP_WIDTH - 1 || next_y < 0 || next_y > MAP_HEIGHT - 1;

        if is_map_end {
            return;
        }

        let tile = game.map.get_tile(next_x, next_y);

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
}

pub struct Game {
    pub map: Map,
}

fn handle_keys(app: &mut App, player: &mut Unit, game: &Game) -> bool {
    let key = app.root.wait_for_keypress(true);

    match key {
        Key {
            code: KeyCode::Char,
            printable: 'w',
            ..
        } => player.r#move(0, -1, game),

        Key {
            code: KeyCode::Char,
            printable: 's',
            ..
        } => player.r#move(0, 1, game),

        Key {
            code: KeyCode::Char,
            printable: 'a',
            ..
        } => player.r#move(-1, 0, game),

        Key {
            code: KeyCode::Char,
            printable: 'd',
            ..
        } => player.r#move(1, 0, game),

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
            return true;
        }

        _ => println!("{:?}", key),
    }

    false
}

fn render_all(app: &mut App, units: &Vec<&mut Unit>) {
    // draw all objects in the list
    for unit in units.into_iter().rev() {
        unit.draw(&mut app.offscreen);
    }
}

fn main() {
    tcod::system::set_fps(FPS);

    let root = Root::initializer()
        .font("arial10x10.png", FontLayout::Tcod)
        .font_type(FontType::Greyscale)
        .size(WIDTH, HEIGHT)
        .title("Roguelike game")
        .init();

    let offscreen = Offscreen::new(MAP_WIDTH, MAP_HEIGHT);

    let mut app = App { root, offscreen };

    let mut player = Unit::new(WIDTH / 2, HEIGHT / 2, '@', colors::WHITE);

    let mut units: Vec<&mut Unit> = Vec::new();

    units.push(&mut player);

    let mut npc = Unit::new(WIDTH / 2 - 10, HEIGHT / 2 - 10, '$', colors::BLACK);

    units.push(&mut npc);

    let map = Map::new(MAP_WIDTH, MAP_HEIGHT);

    let mut game = Game { map };

    game.map.set_tile(30, 22, Tile::wall());
    game.map.set_tile(50, 22, Tile::wall());

    loop {
        app.offscreen.set_default_background(colors::BLUE);
        app.offscreen.clear();

        render_all(&mut app, &units);

        game.map.render(&mut app.offscreen);

        blit(
            &app.offscreen,
            (0, 0),
            (MAP_WIDTH, MAP_HEIGHT),
            &mut app.root,
            (0, 0),
            1.0,
            1.0,
        );

        app.root.flush();

        let exit = handle_keys(&mut app, units[0], &game);

        if app.root.window_closed() || exit {
            break;
        }
    }
}
