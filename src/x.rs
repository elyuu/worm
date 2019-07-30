use xcb;
use xcb_util::{ewmh, icccm, keysyms};

use crate::key::*;

#[derive(Debug)]
#[allow(non_snake_case)]
struct InternedAtoms {
    WM_PROTOCOLS: xcb::Atom,
    WM_DELETE_WINDOW: xcb::Atom,
}

#[allow(non_snake_case)]
impl InternedAtoms {
    pub fn new(connection: &xcb::Connection) -> InternedAtoms {
        let WM_PROTOCOLS = xcb::intern_atom(&connection, false, "WM_PROTOCOLS")
            .get_reply()
            .expect("Error creating InternedAtoms")
            .atom();
        let WM_DELETE_WINDOW = xcb::intern_atom(&connection, false, "WM_DELETE_WINDOW")
            .get_reply()
            .expect("Error creating InternedAtoms")
            .atom();
        InternedAtoms {
            WM_PROTOCOLS,
            WM_DELETE_WINDOW,
        }
    }
}

/// Wrapping xcb::Window to not leak dependency
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Window {
    pub window: xcb::Window,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl Window {
    pub fn new(connection: &Connection, window: xcb::Window) -> Window {
        let geo = connection.get_window_geometry(window);
        Window {
            window: window,
            x: geo.0,
            y: geo.1,
            width: geo.2,
            height: geo.3,
        }
    }

    pub fn as_xcb_window(&self) -> xcb::Window {
        self.window
    }
}

// TODO: Check if last 3 will ever be needed
#[derive(Debug, Default)]
pub struct WindowChanges {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub border_width: u32,
    pub sibling: u32,
    pub stack_mode: u32,
}

#[derive(Debug)]
pub enum XEvent {
    ConfigureRequest(Window, WindowChanges),
    MapRequest(Window),
    UnmapNotify(Window),
    DestroyNotify(Window),
    KeyPress(Key),
}

pub struct Connection {
    connection: ewmh::Connection,
    root_window: Window,
    root_index: i32,
    atoms: InternedAtoms,
}

impl Connection {
    pub fn new() -> Connection {
        // Connect to the default display
        let (connection, root_index) =
            xcb::Connection::connect(None).expect("Could not connect to the display");
        let connection = ewmh::Connection::connect(connection)
            .map_err(|(e, _)| e)
            .expect("Could not create ewmh connection");

        // Get the default root window
        let root_window = connection
            .get_setup()
            .roots()
            .nth(root_index as usize)
            .expect("Could not get root window")
            .root();

        let root_geo = xcb::get_geometry(&connection, root_window)
            .get_reply()
            .expect("Could not get window geometry");

        let root_window = Window {
            window: root_window,
            x: root_geo.x() as u32,
            y: root_geo.y() as u32,
            width: root_geo.width() as u32,
            height: root_geo.height() as u32,
        };

        let atoms = InternedAtoms::new(&connection);

        Connection {
            connection,
            root_window,
            root_index,
            atoms,
        }
    }

    pub fn setup(&self, keys: &KeyMap) {
        // register for substructure redirect/notify
        let values = [(
            xcb::CW_EVENT_MASK,
            xcb::EVENT_MASK_SUBSTRUCTURE_NOTIFY | xcb::EVENT_MASK_SUBSTRUCTURE_REDIRECT,
        )];

        xcb::change_window_attributes_checked(
            &self.connection,
            self.root_window.as_xcb_window(),
            &values,
        )
        .request_check()
        .expect("Could not register for substructure redirect/notify");

        self.grab_keys(&self.root_window, keys);
    }

    pub fn get_existing_windows(&self) -> Vec<Window> {
        // frame existing windows
        xcb::grab_server(&self.connection);

        let existing_windows: Vec<_> =
            xcb::query_tree(&self.connection, self.root_window.as_xcb_window())
                .get_reply()
                .expect("Could not query existing windows")
                .children()
                .iter()
                .map(|w| Window::new(&self, *w))
                .collect();

        xcb::ungrab_server(&self.connection);

        existing_windows
    }

    pub fn root_window(&self) -> Window {
        Window::new(&self, self.root_window.as_xcb_window())
    }

    pub fn flush(&self) {
        self.connection.flush();
    }

    pub fn wait_for_event(&self) -> Option<XEvent> {
        let e = self
            .connection
            .wait_for_event()
            .expect("Error receiving event");
        unsafe {
            match e.response_type() {
                xcb::CONFIGURE_REQUEST => self.configure_request(xcb::cast_event(&e)),
                xcb::MAP_REQUEST => self.map_request(xcb::cast_event(&e)),
                xcb::UNMAP_NOTIFY => self.unmap_notify(xcb::cast_event(&e)),
                xcb::KEY_PRESS => self.key_press(xcb::cast_event(&e)),
                xcb::DESTROY_NOTIFY => self.destroy_notify(xcb::cast_event(&e)),
                _ => None,
            }
        }
    }

    fn configure_request(&self, event: &xcb::ConfigureRequestEvent) -> Option<XEvent> {
        Some(XEvent::ConfigureRequest(
            Window::new(&self, event.window()),
            WindowChanges {
                x: event.x() as u32,
                y: event.y() as u32,
                width: event.width() as u32,
                height: event.height() as u32,
                border_width: event.border_width() as u32,
                sibling: event.sibling() as u32,
                stack_mode: event.stack_mode() as u32,
            },
        ))
    }

    pub fn configure_window(&self, window: &Window, window_changes: &WindowChanges) {
        let value_list = vec![
            (xcb::CONFIG_WINDOW_X as u16, window_changes.x),
            (xcb::CONFIG_WINDOW_Y as u16, window_changes.y),
            (xcb::CONFIG_WINDOW_WIDTH as u16, window_changes.width),
            (xcb::CONFIG_WINDOW_HEIGHT as u16, window_changes.height),
        ];

        xcb::configure_window(&self.connection, window.as_xcb_window(), &value_list);
    }

    fn map_request(&self, event: &xcb::MapRequestEvent) -> Option<XEvent> {
        Some(XEvent::MapRequest(Window::new(&self, event.window())))
    }

    pub fn map_window(&self, window: &Window) {
        xcb::map_window(&self.connection, window.as_xcb_window());
    }

    fn unmap_notify(&self, event: &xcb::UnmapNotifyEvent) -> Option<XEvent> {
        let mut ret;
        if event.event() == self.root_window.as_xcb_window() {
            ret = None;
        } else {
            // Avoiding a looking for the window geo through Window cons
            let mut window = Window::default();
            window.window = event.window();
            ret = Some(XEvent::UnmapNotify(window));
        }
        ret
    }

    pub fn unmap_window(&self, window: &Window) {
        xcb::unmap_window(&self.connection, window.as_xcb_window());
    }

    fn key_press(&self, event: &xcb::KeyPressEvent) -> Option<XEvent> {
        let key_symbols = keysyms::KeySymbols::new(&self.connection);
        let key = key_symbols.press_lookup_keysym(event, 0);
        let modifier = u32::from(event.state());
        let key = Key { modifier, key };
        Some(XEvent::KeyPress(key))
    }

    fn destroy_notify(&self, event: &xcb::DestroyNotifyEvent) -> Option<XEvent> {
        let mut ret;
        if event.event() == self.root_window.as_xcb_window() {
            ret = None;
        } else {
            // Avoiding a looking for the window geo through Window cons
            let mut window = Window::default();
            window.window = event.window();
            ret = Some(XEvent::UnmapNotify(window));
        }
        ret
    }

    pub fn grab_keys(&self, window: &Window, keys: &KeyMap) {
        let key_symbols = keysyms::KeySymbols::new(&self.connection);
        for key in keys.key_map.keys() {
            xcb::grab_key(
                &self.connection,
                false,
                window.as_xcb_window(),
                key.modifier as u16,
                key_symbols
                    .get_keycode(key.key)
                    .next()
                    .expect("Could not resolve keysym"),
                xcb::GRAB_MODE_ASYNC as u8,
                xcb::GRAB_MODE_ASYNC as u8,
            );
        }
    }

    pub fn register_window(&self, window: &Window) {
        let values = [(
            xcb::CW_EVENT_MASK,
            xcb::EVENT_MASK_SUBSTRUCTURE_NOTIFY | xcb::EVENT_MASK_SUBSTRUCTURE_REDIRECT,
        )];

        xcb::change_window_attributes_checked(&self.connection, window.as_xcb_window(), &values)
            .request_check()
            .expect("Could not register for substructure redirect/notify");
    }

    pub fn track_window_events(&self, window: &Window) {
        let values = [(
            xcb::CW_EVENT_MASK,
            xcb::EVENT_MASK_ENTER_WINDOW | xcb::EVENT_MASK_STRUCTURE_NOTIFY,
        )];

        xcb::change_window_attributes_checked(&self.connection, window.as_xcb_window(), &values)
            .request_check()
            .expect("Could not track window events");
    }

    pub fn stop_window_events(&self, window: &Window) {
        let values = [(xcb::CW_EVENT_MASK, xcb::EVENT_MASK_NO_EVENT)];
        xcb::change_window_attributes_checked(&self.connection, window.as_xcb_window(), &values)
            .request_check()
            .expect("Could not stop window events");
    }

    pub fn focus_window(&self, window: Window) {
        xcb::set_input_focus_checked(
            &self.connection,
            xcb::INPUT_FOCUS_POINTER_ROOT as u8,
            window.as_xcb_window(),
            xcb::CURRENT_TIME,
        )
        .request_check()
        .expect("Could not set input focus to focus window");

        ewmh::set_active_window_checked(&self.connection, self.root_index, window.as_xcb_window())
            .request_check()
            .expect("Could not set ewmh focus");
    }

    pub fn delete_window(&self, window: &Window) {
        xcb::grab_server(&self.connection);
        if self
            .get_wm_protocols(window)
            .contains(&self.atoms.WM_DELETE_WINDOW)
        {
            let data = xcb::ClientMessageData::from_data32([
                self.atoms.WM_DELETE_WINDOW,
                xcb::CURRENT_TIME,
                0,
                0,
                0,
            ]);
            let event = xcb::ClientMessageEvent::new(
                32,
                window.as_xcb_window(),
                self.atoms.WM_PROTOCOLS,
                data,
            );
            //self.unmap_window(window);
            xcb::send_event(
                &self.connection,
                false,
                window.as_xcb_window(),
                xcb::EVENT_MASK_NO_EVENT,
                &event,
            );
        } else {
            xcb::destroy_window(&self.connection, window.as_xcb_window());
        }
        xcb::ungrab_server(&self.connection);
    }

    fn get_wm_protocols(&self, window: &Window) -> Vec<xcb::Atom> {
        let protocols = icccm::get_wm_protocols(
            &self.connection,
            window.as_xcb_window(),
            self.atoms.WM_PROTOCOLS,
        )
        .get_reply()
        .expect("Could not get wm protocols");
        Vec::from(protocols.atoms())
    }

    /// function to find xcb::idow geometry as (x, y, width, height)
    pub fn get_window_geometry(&self, window: xcb::Window) -> (u32, u32, u32, u32) {
        println!("GETTING GEO FOR WINDOW: {}", window);
        let geo = xcb::get_geometry(&self.connection, window)
            .get_reply()
            .expect(&format!("Could not get window geometry for window: {}", window));
        (
            geo.x() as u32,
            geo.y() as u32,
            geo.width() as u32,
            geo.height() as u32,
        )
    }
}
