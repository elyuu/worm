use std::collections::HashMap;

use worm::command::*;
use worm::key::*;
use worm::layout::*;
use worm::*;

use x11::keysym::*;

fn main() {
    println!("Worm - X Window Manager");

    let binds = bindings!(
        (Modifier::Mod1, XK_t, Command::ChangeLayout(Layout::Tile)),
        (Modifier::Mod1, XK_s, Command::ChangeLayout(Layout::Float))
    );

    let mut wm = Worm::new(binds);

    wm.run();
}
