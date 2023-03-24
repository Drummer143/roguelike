mod map;
mod unit;

use map::{Map, Tile};
use unit::Unit;

use tcod::{
    colors,
    console::{blit, Offscreen, Root},
    input::{Key, KeyCode},
    Console, FontLayout, FontType,
};

const WIDTH: i32 = 100;
const HEIGHT: i32 = 100;

const MAP_WIDTH: i32 = 100;
const MAP_HEIGHT: i32 = 100;

const FPS: i32 = 60;

pub struct Game {
    pub map: Map,
}

struct App {
    root: Root,
    offscreen: Offscreen,
    game: Game
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

    let mut player = Unit::new(WIDTH / 2, HEIGHT / 2, '@', colors::WHITE);

    let mut units: Vec<&mut Unit> = Vec::new();

    units.push(&mut player);

    let mut npc = Unit::new(WIDTH / 2 - 10, HEIGHT / 2 - 10, '$', colors::BLACK);

    units.push(&mut npc);

    let mut app = App { root, offscreen, game: Game { map: Map::new(MAP_WIDTH, MAP_HEIGHT) } };

    app.game.map.set_tile(30, 22, Tile::wall());
    app.game.map.set_tile(50, 22, Tile::wall());

    loop {
        app.offscreen.set_default_background(colors::BLUE);
        app.offscreen.clear();

        render_all(&mut app, &units);

        app.game.map.render(&mut app.offscreen);

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

        let exit = handle_keys(&mut app, units[0]);

        if app.root.window_closed() || exit {
            break;
        }
    }
}
