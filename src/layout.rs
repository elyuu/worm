use crate::x;

pub enum Layout {
    Float,
    Monocle,
    Tile,
}

impl Layout {
    pub fn apply(&self, conn: &x::Connection, windows: &Vec<x::Window>) {
        match self {
            Layout::Float => Layout::float(conn, windows),
            Layout::Monocle => Layout::monocle(conn, windows),
            Layout::Tile => Layout::tile(conn, windows),
        };
    }

    fn float(conn: &x::Connection, windows: &Vec<x::Window>) {
        for window in windows.iter() {
            conn.map_window(window);
        }
    }

    fn monocle(conn: &x::Connection, windows: &Vec<x::Window>) {
        println!("Monocle");
    }

    fn tile(conn: &x::Connection, windows: &Vec<x::Window>) {
        println!("Tile");
    }
}

