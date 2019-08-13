use std::rc::Rc;

use crate::command::Direction;
use crate::layout::Layout;
use crate::x;
use crate::Screen;

pub struct Desktops {
    pub desktops: Vec<Desktop>,
    pub focused_desktop: usize,
}

pub struct Desktop {
    name: String,
    active: bool,
    layout: Layout,
    windows: Vec<x::Window>,
    pub focused_window: Option<usize>,
    pub focused_last: Option<usize>,
    connection: Rc<x::Connection>,
    screen: Screen,
}

impl Desktops {
    pub fn new(desktops: Vec<Desktop>, focused_desktop: usize) -> Desktops {
        Desktops {
            desktops,
            focused_desktop,
        }
    }

    pub fn contains(&self, window: &x::Window) -> bool {
        self.desktops.iter().any(|d| d.contains(window))
    }

    pub fn add_window(&mut self, window: x::Window) {
        self.desktops[self.focused_desktop].add_window(window);
    }

    // TODO: Change this
    pub fn remove_window(&mut self, window: &x::Window) {
        let window_index = self.desktops[self.focused_desktop].get_window_index(window);
        self.desktops[self.focused_desktop].remove_window(window_index);
    }

    pub fn layout(&self) -> Layout {
        self.desktops[self.focused_desktop].layout()
    }

    pub fn change_layout(&mut self, layout: &Layout) {
        self.desktops[self.focused_desktop].change_layout(layout);
    }

    pub fn apply_layout(&mut self) {
        self.desktops[self.focused_desktop].apply_layout();
    }

    // TODO: Cleanup is needed
    pub fn focus_window(&mut self, direction: &Direction) {
        if self.desktops[self.focused_desktop].get_focused_window().is_none() {
            return;
        }

        match self.layout() {
            Layout::Tile => self.focus_window_tile(direction),
            Layout::Monocle => self.focus_window_monocle(direction),
            _ => {}
        }
    }

    fn focus_window_tile(&mut self, direction: &Direction) {
        match direction {
            Direction::Up => {
                // TODO: This would need to be compared to num_master global
                // config variable if that gets supported
                if self.desktops[self.focused_desktop].focused_window == Some(0) {
                    return;
                } else if self.desktops[self.focused_desktop].focused_window.unwrap() - 1 == 0 {
                    return;
                }
                self.desktops[self.focused_desktop].focused_last =
                    self.desktops[self.focused_desktop].focused_window;
                match self.desktops[self.focused_desktop].focused_window.as_mut() {
                    Some(i) => *i -= 1,
                    None => {},
                };
                self.desktops[self.focused_desktop].update_focus();
            }
            Direction::Down => {
                if self.desktops[self.focused_desktop].focused_window == Some(0) {
                    return;
                } else if self.desktops[self.focused_desktop].focused_window.unwrap() + 1
                    == self.desktops[self.focused_desktop].windows.len()
                {
                    return;
                }
                self.desktops[self.focused_desktop].focused_last =
                    self.desktops[self.focused_desktop].focused_window;
                match self.desktops[self.focused_desktop].focused_window.as_mut() {
                    Some(i) => *i += 1,
                    None => {},
                };
                self.desktops[self.focused_desktop].update_focus();
            }
            // TODO: This would need to be compared to num_master global
            // config variable if that gets supported, not 0
            Direction::Left => {
                if self.desktops[self.focused_desktop].focused_window == Some(0) {
                    return;
                }
                self.desktops[self.focused_desktop].focused_last =
                    self.desktops[self.focused_desktop].focused_window;
                match self.desktops[self.focused_desktop].focused_window.as_mut() {
                    Some(i) => *i = 0,
                    None => {},
                };
                self.desktops[self.focused_desktop].update_focus();
            }
            // TODO: This would need to be compared to num_master global
            // config variable if that gets supported, not 0
            Direction::Right => {
                if self.desktops[self.focused_desktop].focused_window != Some(0) {
                    return;
                }
                if self.desktops[self.focused_desktop].focused_last == Some(0)
                    && self.desktops[self.focused_desktop].windows.len() > 1
                {
                    match self.desktops[self.focused_desktop].focused_last.as_mut() {
                        Some(i) => *i = 1,
                        None => {},
                    };
                }
                self.desktops[self.focused_desktop].focused_window =
                    self.desktops[self.focused_desktop].focused_last;
                match self.desktops[self.focused_desktop].focused_window.as_mut() {
                    Some(i) => *i = 0,
                    None => {},
                };
                self.update_focus();
            }
        }
    }

    fn focus_window_monocle(&mut self, direction: &Direction) {
        // Treat both up/down as cylcle backward/forward
        match direction {
            Direction::Down | Direction::Right => {
                self.desktops[self.focused_desktop].cycle_window_forward();
                self.update_focus();
            }
            Direction::Up | Direction::Left => {
                self.desktops[self.focused_desktop].cycle_window_backward();
                self.update_focus();
            }
        }
    }

    fn update_focus(&self) {
        self.desktops[self.focused_desktop].update_focus();
    }

    // TODO: Probably propogate the Option
    fn get_focused_window(&self) -> Option<x::Window> {
        self.desktops[self.focused_desktop].get_focused_window()
    }

    pub fn delete_focused_window(&mut self) {
        self.desktops[self.focused_desktop].delete_focused_window();
        self.update_focus();
    }
}

impl Desktop {
    pub fn new(
        name: &String,
        active: bool,
        layout: Layout,
        windows: Vec<x::Window>,
        connection: Rc<x::Connection>,
        screen: &Screen,
    ) -> Desktop {
        Desktop {
            name: name.clone(),
            active,
            layout,
            windows: windows,
            focused_window: None,
            focused_last: None,
            connection,
            screen: screen.clone(),
        }
    }

    fn contains(&self, window: &x::Window) -> bool {
        self.windows.iter().any(|w| w == window)
    }

    fn add_window(&mut self, window: x::Window) {
        if self.focused_window.is_none() {
            self.focused_window = Some(0);
            self.focused_last = None;
        }
        self.windows.push(window);
        self.apply_layout();
    }

    fn remove_window(&mut self, index: Option<usize>) -> Option<x::Window>{
        match index {
            Some(i) => Some(self.windows.remove(i)),
            None => None,
        }
    }

    fn layout(&self) -> Layout {
        self.layout
    }

    fn change_layout(&mut self, layout: &Layout) {
        if self.layout == *layout {
            return;
        }
        self.layout = layout.clone();
        self.apply_layout();
    }

    fn apply_layout(&mut self) {
        /* TODO: Set and check for active desktops everywhere
        if !self.active {
            return
        }
        */

        self.layout
            .apply(&self.connection, &self.windows, &self.screen);

        // TODO: Set focus on a window maybe
    }

    // TODO: Maybe propogate option
    fn update_focus(&self) {
        if let Some(focused) = self.get_focused_window() {
            self.connection.focus_window(focused);
        } else {
            return;
        }
    }

    fn cycle_window_forward(&mut self) {
        if self.layout() != Layout::Monocle {
            panic!("Trying to cylce on non monocle layout");
        }

        if let Some(focused) = self.get_focused_window() {
            if self.focused_window == Some(self.windows.len() - 1) {
                self.connection.unmap_window(&focused);
                self.focused_last = self.focused_window;
                self.focused_window = Some(0);
                self.connection.map_window(&focused);
            } else {
                self.connection.unmap_window(&focused);
                self.focused_last = self.focused_window;
                self.focused_window.map(|mut i| i += 1);
                self.connection.map_window(&focused);
            }
        } else {
            return;
        }
    }

    fn cycle_window_backward(&mut self) {
        if self.layout() != Layout::Monocle {
            panic!("Trying to cylce on non monocle layout");
        }

        if let Some(focused) = self.get_focused_window() {
            if self.focused_window == Some(0) {
                self.connection.unmap_window(&focused);
                self.focused_last = Some(0);
                match self.focused_window.as_mut() {
                    Some(i) => *i = self.windows.len() - 1,
                    None => {},
                };
                self.connection.map_window(&focused);
            } else {
                self.connection.unmap_window(&focused);
                self.focused_last = self.focused_window;
                match self.focused_window.as_mut() {
                    Some(i) => *i -= 1,
                    None => {},
                };
                self.connection.map_window(&focused);
            }
        } else {
            return;
        }
    }


    /// Gets the focused x::Window or panics if focused is None
    fn get_focused_window(&self) -> Option<x::Window> {
        match self.focused_window {
            Some(i) => Some(self.windows[i]),
            None => None,
        }
    }

    fn get_window_index(&self, window: &x::Window) -> Option<usize> {
        for (i, win) in self.windows.iter().enumerate() {
            if win.as_xcb_window() == window.as_xcb_window() {
                return Some(i);
            }
        }
        None
    }

    fn delete_focused_window(&mut self) {
        if self.focused_window.is_none() {
            return;
        }

        if let Some(focused) = self.get_focused_window() {
            self.connection.delete_window(&focused);
            self.connection.flush();
            self.remove_window(self.focused_window);

            // FIXME: Make sure this works
            self.focused_window = self.focused_last;
            self.focused_last = None;
            self.apply_layout();
            self.update_focus();
        } else {
            return;
        }


    }
}
