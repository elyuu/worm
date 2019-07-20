use crate::layout::Layout;
use crate::x::Window;
use crate::Worm;

#[derive(Copy, Clone)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Copy, Clone)]
pub enum Command {
    ChangeLayout(Layout),
    FocusDirection(Direction),
    FocusDesktop(usize),
    Kill(Window),
}

impl Command {
    pub fn command(&self, wm: &mut Worm) {
        match self {
            Command::ChangeLayout(l) => Command::change_layout(wm, l),
            _ => return,
        };
    }

    fn change_layout(wm: &mut Worm, layout: &Layout) {
        wm.desktops[wm.active_desktop].change_layout(layout);
    }
}
