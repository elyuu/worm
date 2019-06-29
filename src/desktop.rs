use std::rc::Rc;

use crate::layout::Layout;
use crate::x;

pub struct Desktop {
    name: String,
    active: bool,
    layout: Layout,
    windows: Vec<x::Window>,
    focused_window: u32,
    connection: Rc<x::Connection>,
}

impl Desktop {
    pub fn new(
        name: &String,
        active: bool,
        layout: Layout,
        windows: Vec<x::Window>,
        focused_window: u32,
        connection: Rc<x::Connection>,
    ) -> Desktop {
        Desktop {
            name: name.clone(),
            active,
            layout,
            windows: windows,
            focused_window,
            connection,
        }
    }

    pub fn contains(&self, window: &x::Window) -> bool {
        self.windows.iter().any(|w| w == window)
    }

    pub fn add_window(&mut self, window: x::Window) {
        println!("Mapping a window");
        self.connection.map_window(&window);
        self.windows.push(window);
    }

    pub fn remove_window(&mut self, window: &x::Window) {}

    pub fn layout(&self) -> Layout {
        self.layout
    }
}
