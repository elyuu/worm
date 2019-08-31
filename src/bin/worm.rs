use std::collections::HashMap;

use worm::command::*;
use worm::key::*;
use worm::layout::*;
use worm::*;

use x11::keysym::*;

fn main() {
    println!("Worm - X Window Manager");

    let binds = bindings!(
        (&[Modifier::Mod1], XK_t, Command::ChangeLayout(Layout::Tile)),
        (&[Modifier::Mod1], XK_s, Command::ChangeLayout(Layout::Float)),
        (&[Modifier::Mod1], XK_m, Command::ChangeLayout(Layout::Monocle)),
        (&[Modifier::Mod1], XK_h, Command::FocusDirection(Direction::Left)),
        (&[Modifier::Mod1], XK_j, Command::FocusDirection(Direction::Down)),
        (&[Modifier::Mod1], XK_k, Command::FocusDirection(Direction::Up)),
        (&[Modifier::Mod1], XK_l, Command::FocusDirection(Direction::Right)),
        (&[Modifier::Mod1, Modifier::Shift], XK_k, Command::MoveDirection(Direction::Up)),
        (&[Modifier::Mod1, Modifier::Shift], XK_j, Command::MoveDirection(Direction::Down)),
        (&[Modifier::Mod1, Modifier::Shift], XK_h, Command::MoveDirection(Direction::Left)),
        (&[Modifier::Mod1, Modifier::Shift], XK_l, Command::MoveDirection(Direction::Right)),
        (&[Modifier::Mod1], XK_w, Command::KillFocused)
    );

    let mut wm = Worm::new(binds);

    wm.run();
}
