use worm::Worm;

fn main() {
    println!("Worm - X Window Manager");

    let mut wm = Worm::new();

    wm.run();
}
