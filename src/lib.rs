use std::rc::Rc;

mod desktop;
mod layout;
mod x;

use desktop::Desktop;
use layout::Layout;

pub struct Worm {
    connection: Rc<x::Connection>,
    desktops: Vec<Desktop>,
    active_desktop: usize,
}

impl Worm {
    pub fn new() -> Worm {
        let connection = x::Connection::new();
        let connection = Rc::new(connection);
        connection.setup();

        let existing_windows: Vec<x::Window> = Vec::new();

        let mut desktops: Vec<Desktop> = Vec::new();
        for i in 0..9 {
            desktops.push(Desktop::new(
                &i.to_string(),
                false,
                Layout::Tile,
                existing_windows.clone(),
                0,
                connection.clone(),
            ))
        }


        let mut wm = Worm {
            connection: connection.clone(),
            desktops,
            active_desktop: 0,
        };

        let windows = connection.get_existing_windows();

        for window in windows.iter() {
            wm.manage(window.clone());
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

            match event {
                x::XEvent::MapRequest(w) => self.map_request(w),
                _ => continue,
            };
        }
    }

    fn manage(&mut self, window: x::Window) {
        if self.is_managed(&window) {
            panic!("Already managed window attempting to be managed again");
        }

        self.connection.register_window(&window);
        self.desktops[self.active_desktop].add_window(window);
    }

    fn manage_existing(&mut self) {
        let windows = self.connection.get_existing_windows();

        for window in windows.iter() {
            self.manage(window.clone());
        }
    }

    fn map_request(&mut self, window: x::Window) {
        self.manage(window);
    }

    fn is_managed(&self, window: &x::Window) -> bool {
        self.desktops.iter().any(|d| d.contains(window))
    }

}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
