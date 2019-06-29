use xcb;
use xcb_util::{ewmh, icccm, keysyms};

use crate::key::Key;

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
#[derive(Clone, Debug, PartialEq)]
pub struct Window(xcb::Window);

impl Window {
    pub fn as_xcb_window(&self) -> xcb::Window {
        self.0
    }
}

pub struct WindowChanges {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    border_width: u32,
    sibling: u32,
    stack_mode: u32,
}

pub struct Connection {
    connection: ewmh::Connection,
    root_window: Window,
    atoms: InternedAtoms,
}

impl Connection {
    pub fn new() -> Connection {
        // Connect to the default display
        let (connection, root_idx) =
            xcb::Connection::connect(None).expect("Could not connect to the display");
        let connection = ewmh::Connection::connect(connection)
            .map_err(|(e, _)| e)
            .expect("Could not create ewmh connection");

        // Get the default root window
        let root_window = connection
            .get_setup()
            .roots()
            .nth(root_idx as usize)
            .expect("Could not get root window")
            .root();
        let root_window = Window(root_window);

        let atoms = InternedAtoms::new(&connection);

        Connection {
            connection,
            root_window,
            atoms,
        }
    }

    pub fn setup(&self) {
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
                .map(|w| Window(*w))
                .collect();

        xcb::ungrab_server(&self.connection);

        existing_windows
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
                _ => None,
            }
        }
    }

    fn configure_request(&self, event: &xcb::ConfigureRequestEvent) -> Option<XEvent> {
        Some(XEvent::ConfigureRequest(
            Window(event.window()),
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
            (
                xcb::CONFIG_WINDOW_BORDER_WIDTH as u16,
                window_changes.border_width,
            ),
            (xcb::CONFIG_WINDOW_SIBLING as u16, window_changes.sibling),
            (
                xcb::CONFIG_WINDOW_STACK_MODE as u16,
                window_changes.stack_mode,
            ),
        ];

        xcb::configure_window(&self.connection, window.as_xcb_window(), &value_list);
    }

    fn map_request(&self, event: &xcb::MapRequestEvent) -> Option<XEvent> {
        Some(XEvent::MapRequest(Window(event.window())))
    }

    pub fn map_window(&self, window: &Window) {
        xcb::map_window(&self.connection, window.as_xcb_window());
    }

    fn unmap_notify(&self, event: &xcb::UnmapNotifyEvent) -> Option<XEvent> {
        Some(XEvent::UnmapNotify(Window(event.window())))
    }

    pub fn unmap_window(&self, window: &Window) {
        xcb::unmap_window(&self.connection, window.as_xcb_window());
    }

    // TODO: Actually handle key events
    fn key_press(&self, event: &xcb::KeyPressEvent) -> Option<XEvent> {
        let key_symbols = keysyms::KeySymbols::new(&self.connection);
        let key = key_symbols.press_lookup_keysym(event, 0);
        let modifier = u32::from(event.state());
        let key = Key { modifier, key };
        Some(XEvent::KeyPress(key))
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

    pub fn grab_key(&self, key: Key, window: &Window) {
        let key_symbols = keysyms::KeySymbols::new(&self.connection);

        xcb::grab_key(
            &self.connection,
            false,
            window.as_xcb_window(),
            key.modifier as u16,
            key_symbols.get_keycode(key.key).next().expect("Could not resolve keysym"),
            xcb::GRAB_MODE_ASYNC as u8,
            xcb::GRAB_MODE_ASYNC as u8,
        );
    }
}

pub enum XEvent {
    ConfigureRequest(Window, WindowChanges),
    MapRequest(Window),
    UnmapNotify(Window),
    KeyPress(Key),
}
