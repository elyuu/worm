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
    MoveDirection(Direction),
    KillFocused,
}

impl Command {
    pub fn command(&self, wm: &mut Worm) {
        match self {
            Command::ChangeLayout(l) => Command::change_layout(wm, l),
            Command::FocusDirection(d) => Command::focus_direction(wm, d),
            Command::MoveDirection(d) => Command::move_direction(wm, d),
            Command::KillFocused => Command::kill_focused(wm),
            _ => return,
        };
    }

    fn change_layout(wm: &mut Worm, layout: &Layout) {
        wm.desktops.change_layout(layout);
    }

    fn focus_direction(wm: &mut Worm, direction: &Direction) {
        wm.desktops.focus_window(direction);
        println!(
            "FOCUSED: {:?}, FOCUSED_LAST: {:?}",
            wm.desktops.desktops[wm.desktops.focused_desktop].focused_window,
            wm.desktops.desktops[wm.desktops.focused_desktop].focused_last
        );
    }

    fn move_direction(wm: &mut Worm, direction: &Direction) {
        wm.desktops.move_window_tile(direction);
    }

    fn kill_focused(wm: &mut Worm) {
        wm.desktops.delete_focused_window();
    }
}
