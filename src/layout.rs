use crate::x;
use crate::Screen;

type Gap = u32;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Layout {
    Float,
    Monocle,
    Tile,
}

impl Layout {
    pub fn apply(&self, conn: &x::Connection, windows: &Vec<x::Window>, screen: &Screen) {
        match self {
            Layout::Float => Layout::float(conn, windows),
            Layout::Monocle => Layout::monocle(conn, windows, screen),
            Layout::Tile => Layout::tile(conn, windows, screen),
        };
    }

    fn float(connection: &x::Connection, windows: &Vec<x::Window>) {
        if windows.is_empty() {
            return;
        }

        for window in windows {
            connection.stop_window_events(&window);
            connection.map_window(&window);
            connection.track_window_events(&window);
        }
    }

    fn monocle(connection: &x::Connection, windows: &Vec<x::Window>, screen: &Screen) {
        if windows.is_empty() {
            return;
        }

        let window_changes = x::WindowChanges {
            x: screen.x,
            y: screen.y,
            width: screen.width,
            height: screen.height,
            border_width: 0,
            sibling: 0,
            stack_mode: 0,
        };

        for window in windows {
            connection.stop_window_events(&window);
            connection.map_window(&window);
            connection.configure_window(&window, &window_changes);
            connection.track_window_events(&window);
        }
    }

    // TODO: Cleanup
    fn tile(connection: &x::Connection, windows: &Vec<x::Window>, screen: &Screen) {
        if windows.is_empty() {
            return;
        }

        // TODO: TEMP
        let gap = 10;
        let num_master = 1;
        let master_fact = 0.5;

        let num_windows = windows.len();
        let mut g = 0;
        let mut master_width = 0;

        if num_windows > num_master {
            master_width = if num_master > 0 {
                g = gap;
                ((screen.width - g) as f32 * master_fact) as u32
            } else {
                0
            };
        } else {
            master_width = 0;
        }

        let mut my = 0;
        let mut ty = 0;
        let mut r = 0;
        let mut h = 0;

        for (i, window) in windows.iter().enumerate() {
            let mut window_changes = x::WindowChanges::default();
            if i < num_master {
                r = (usize::min(num_windows, num_master) - i) as u32;
                h = (screen.height - my - gap * (r - 1)) / r;
                window_changes = x::WindowChanges {
                    x: screen.x,
                    y: screen.y + my,
                    width: master_width,
                    height: h,
                    border_width: 0,
                    sibling: 0,
                    stack_mode: 0,
                };
                my += h;
            } else {
                r = (num_windows - i) as u32;
                h = (screen.height - ty - gap * (r - 1)) / r;
                window_changes = x::WindowChanges {
                    x: screen.x + master_width + g,
                    y: screen.y + ty,
                    width: screen.width - master_width - g,
                    height: h,
                    border_width: 0,
                    sibling: 0,
                    stack_mode: 0,
                };
                ty += h + gap;
            }
            connection.stop_window_events(&window);
            connection.map_window(&window);
            connection.configure_window(&window, &window_changes);
            connection.track_window_events(&window);
        }
    }
}
