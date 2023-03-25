mod map;
mod unit;

use std::process::Command;
use tcod::{
    colors,
    console::{blit, Offscreen, Root},
    input::{Key, KeyCode},
    Console, FontLayout, FontType,
};

use map::Map;
use unit::UserActions;

const WIDTH: i32 = 100;
const HEIGHT: i32 = 100;

const FPS: i32 = 60;

pub struct Game {
    pub map: Map,
}

struct App {
    root: Root,
    offscreen: Offscreen,
    game: Game,
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

fn handle_keys(app: &mut App) -> UserActions {
    use UserActions::*;

    let key = app.root.wait_for_keypress(true);

    match (key, app.game.map.get_player().is_alive()) {
        (
            Key {
                code: KeyCode::Char,
                printable: 'w',
                ..
            },
            true,
        ) => {
            if app.game.map.player_move_or_attack(0, -1) {
                TookTurn
            } else {
                DidNotTakeTurn
            }
        }

        (
            Key {
                code: KeyCode::Char,
                printable: 's',
                ..
            },
            true,
        ) => {
            if app.game.map.player_move_or_attack(0, 1) {
                TookTurn
            } else {
                DidNotTakeTurn
            }
        }

        (
            Key {
                code: KeyCode::Char,
                printable: 'a',
                ..
            },
            true,
        ) => {
            if app.game.map.player_move_or_attack(-1, 0) {
                TookTurn
            } else {
                DidNotTakeTurn
            }
        }

        (
            Key {
                code: KeyCode::Char,
                printable: 'd',
                ..
            },
            true,
        ) => {
            if app.game.map.player_move_or_attack(1, 0) {
                TookTurn
            } else {
                DidNotTakeTurn
            }
        }

        (
            Key {
                code: KeyCode::Char,
                printable: 'r',
                ..
            },
            _,
        ) => {
            restart();

            Exit
        }

        (
            Key {
                code: KeyCode::Enter,
                alt: true,
                ..
            },
            _,
        ) => {
            let fullscreen = app.root.is_fullscreen();
            app.root.set_fullscreen(!fullscreen);

            DidNotTakeTurn
        }

        (
            Key {
                code: KeyCode::Escape,
                ..
            },
            _,
        ) => {
            std::process::exit(0);
        }

        _ => DidNotTakeTurn,
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

    let mut app = App {
        root,
        offscreen: Offscreen::new(WIDTH, HEIGHT),
        game: Game {
            map: Map::new(WIDTH, HEIGHT),
        },
    };

    app.game.map.set_fov();

    loop {
        app.offscreen.set_default_background(colors::BLACK);
        app.offscreen.clear();

        app.game.map.render(&mut app.offscreen);

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

        let user_action = handle_keys(&mut app);

        if app.root.window_closed() || user_action == UserActions::Exit {
            break;
        }

        app.game.map.monsters_action(user_action);
    }
}
