use xcb;
use xcb_util::{ewmh, icccm, keysyms};

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

// Wrapping xcb::Window to not leak dependency
#[derive(Clone, Debug, PartialEq)]
pub struct Window(xcb::Window);

impl Window {
    pub fn as_xcb_window(&self) -> xcb::Window {
        self.0
    }
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
                xcb::MAP_REQUEST => self.map_request(xcb::cast_event(&e)),
                _ => None,
            }
        }
    }

    fn map_request(&self, event: &xcb::MapRequestEvent) -> Option<XEvent> {
        Some(XEvent::MapRequest(Window(event.window())))
    }

    pub fn map_window(&self, window: &Window) {
        xcb::map_window(&self.connection, window.as_xcb_window());
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
}

pub enum XEvent {
    MapRequest(Window),
    UnmapNotify(Window),
    KeyPress(Window),
}
