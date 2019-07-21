use std::rc::Rc;

use crate::layout::Layout;
use crate::x;

use crate::Screen;

pub struct Desktops {
    desktops: Vec<Desktop>,
    active_desktop: usize,
}

pub struct Desktop {
    name: String,
    active: bool,
    layout: Layout,
    windows: Vec<x::Window>,
    focused_window: u32,
    connection: Rc<x::Connection>,
    screen: Screen,
}

impl Desktops {
    pub fn new(desktops: Vec<Desktop>, active_desktop: usize) -> Desktops {
        Desktops { desktops, active_desktop }
    }

    pub fn contains(&self, window: &x::Window) -> bool {
        self.desktops.iter().any(|d| d.contains(window))
    }

    pub fn add_window(&mut self, window: x::Window) {
        self.desktops[self.active_desktop].add_window(window);
    }

    pub fn remove_window(&mut self, window: &x::Window) {}

    pub fn layout(&self) -> Layout {
        self.desktops[self.active_desktop].layout()
    }

    pub fn change_layout(&mut self, layout: &Layout) {
        self.desktops[self.active_desktop].change_layout(layout);
    }

    pub fn apply_layout(&mut self) {
        self.desktops[self.active_desktop].apply_layout();
    }

    pub fn focus_window(&mut self) {}
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
        self.windows.push(window);
        self.apply_layout();
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
        self.apply_layout();
    }

    pub fn apply_layout(&mut self) {
        /* TODO: Set and check for active desktops everywhere
        if !self.active {
            return
        }
        */
        self.layout.apply(&self.connection, &self.windows, &self.screen);

        // TODO: Set focus on a window maybe
    }

    pub fn focus_window(&mut self) {}
}
