use raylib::prelude::*;
use song::load_nbs_file;

mod note;
mod piano;
mod song;

fn main() {
    let window_width = 1280.;
    let window_height = 720.;

    let (mut rl, thread) = raylib::init()
        .size(window_width as i32, window_height as i32)
        .title("Hello, World")
        .build();

    let (all_keys, key_map) = piano::generate_piano_keys();

    let piano_props = piano::initialize_piano_dimensions(window_width, window_height, &all_keys);

    let nbs_file = load_nbs_file(None);

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::DARKGRAY);
        d.draw_text("Hello, world!", 12, 12, 20, Color::BLACK);

        piano::draw_piano_keys(window_width, window_height, &all_keys, &piano_props, d);
    }
}
