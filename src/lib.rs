use std::rc::Rc;

pub mod command;
mod desktop;
pub mod key;
pub mod layout;
mod x;

use desktop::*;
use key::*;
use layout::Layout;

#[macro_export]
macro_rules! bindings {
    (  $( ($mod:expr, $key:expr, $command:expr) ),* ) => {
        {
            let mut binds = HashMap::new();
            $(
                binds.insert(Key { modifier: $mod.get_mod_mask() , key: $key }, $command);
            )*
            KeyMap { key_map: binds }
        }
    };
}

// TODO: Might needo some other screen specific things
// i.e. info about bars and stuff
/// struct that represents the usable portion of the screen
#[derive(Clone, Debug)]
pub struct Screen {
    width: u32,
    height: u32,
    x: u32,
    y: u32,
}

pub struct Worm {
    connection: Rc<x::Connection>,
    desktops: Desktops,
    binds: KeyMap,
    screen: Screen,
}

impl Worm {
    pub fn new(binds: KeyMap) -> Worm {
        let connection = x::Connection::new();
        let connection = Rc::new(connection);
        connection.setup(&binds);

        let existing_windows: Vec<x::Window> = Vec::new();

        // TODO: Change this to account for actual screen size, maybe get rid of Screen
        // struct and just use x::Window
        let root_window = connection.root_window();
        let screen = Screen {
            x: root_window.x,
            y: root_window.y,
            width: root_window.width,
            height: root_window.height,
        };

        let mut desktops: Vec<Desktop> = Vec::new();
        for i in 0..9 {
            desktops.push(Desktop::new(
                &i.to_string(),
                false,
                Layout::Tile,
                existing_windows.clone(),
                None,
                connection.clone(),
                &screen,
            ))
        }

        let desktops = Desktops::new(desktops, 0);

        let mut wm = Worm {
            connection: connection.clone(),
            desktops,
            binds,
            screen,
        };

        // Manage existing windows and add them to 1sr desktop
        let windows = connection.get_existing_windows();
        for window in windows.iter() {
            wm.manage(window);
        }

        wm
    }

    pub fn run(&mut self) {
        loop {
            self.connection.flush();

            let event = match self.connection.wait_for_event() {
                Some(e) => e,
                None => continue,
            };

            println!("EVENT: {:?}", event);

            match event {
                x::XEvent::ConfigureRequest(w, wc) => self.configure_request(w, wc),
                x::XEvent::KeyPress(k) => self.key_press_event(k),
                x::XEvent::MapRequest(w) => self.map_request(&w),
                x::XEvent::UnmapNotify(w) => self.unmap_notify(&w),
                x::XEvent::DestroyNotify(w) => self.destroy_notify(&w),
            };
        }
    }

    fn manage(&mut self, window: &x::Window) {
        if self.is_managed(&window) {
            panic!("Already managed window attempting to be managed again");
        }

        self.connection.grab_keys(window, &self.binds);
        self.connection.register_window(window);
        self.connection.track_window_events(window);
        self.desktops.add_window(window.clone());
    }

    fn unmanage(&mut self, window: &x::Window) {
        if !self.is_managed(window) {
            return;
        }

        //self.connection.stop_window_events(window);
        //self.connection.unmap_window(window);
        self.desktops.remove_window(window);
    }

    fn manage_existing(&mut self) {
        let windows = self.connection.get_existing_windows();

        for window in windows.iter() {
            self.manage(window);
        }
    }

    fn configure_request(&self, window: x::Window, window_changes: x::WindowChanges) {
        // Don't change anything
        self.connection.configure_window(&window, &window_changes);
    }

    fn key_press_event(&mut self, key: Key) {
        let cmd = self.binds.key_map[&key].clone();
        cmd.command(self);
    }

    fn map_request(&mut self, window: &x::Window) {
        self.manage(window);
    }

    fn unmap_notify(&mut self, window: &x::Window) {
        self.unmanage(window);
    }

    fn destroy_notify(&mut self, window: &x::Window) {
        self.unmanage(window);
    }

    fn is_managed(&self, window: &x::Window) -> bool {
        self.desktops.contains(window)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
