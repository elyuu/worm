use std::rc::Rc;

use crate::layout::Layout;
use crate::x;

use crate::Screen;

pub struct Desktop {
    name: String,
    active: bool,
    layout: Layout,
    windows: Vec<x::Window>,
    focused_window: u32,
    connection: Rc<x::Connection>,
    screen: Screen,
}

impl Desktop {
    pub fn new(
        name: &String,
        active: bool,
        layout: Layout,
        windows: Vec<x::Window>,
        focused_window: u32,
        connection: Rc<x::Connection>,
        screen: &Screen,
    ) -> Desktop {
        Desktop {
            name: name.clone(),
            active,
            layout,
            windows: windows,
            focused_window,
            connection,
            screen: screen.clone(),
        }
    }

    pub fn contains(&self, window: &x::Window) -> bool {
        self.windows.iter().any(|w| w == window)
    }

    pub fn add_window(&mut self, window: x::Window) {
        self.connection.map_window(&window);
        self.windows.push(window);
    }

    pub fn remove_window(&mut self, window: &x::Window) {}

    pub fn layout(&self) -> Layout {
        self.layout
    }

    pub fn change_layout(&mut self, layout: &Layout) {
        if self.layout == *layout {
            return;
        }
        self.layout = layout.clone();
        self.layout.apply(&self.connection, &self.windows, &self.screen);
    }

    pub fn focus_window(&mut self) {}
}
