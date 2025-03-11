use raylib::prelude::*;
use song::load_nbs_file;

mod note;
mod piano;
mod song;

#[derive(Clone, Debug)]
pub struct PianoProps {
    pub key_spacing: f32,
    pub white_key_width: f32,
    pub white_key_height: f32,
    pub black_key_width: f32,
    pub black_key_height: f32,
}

fn main() {
    let window_width = 1280.0;
    let window_height = 720.0;

    let (mut rl, thread) = raylib::init()
        .size(window_width as i32, window_height as i32)
        .title("Hello, World")
        .build();

    let (all_keys, key_map) = piano::generate_piano_keys();

    let piano_props = initialize_piano_dimensions(window_width, window_height, &all_keys);

    let nbs_file = load_nbs_file(None);

    let note_blocks: Vec<Vec<note::NoteBlock>> = note::get_note_blocks(&nbs_file);

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::DARKGRAY);
        d.draw_text("Hello, world!", 12, 12, 20, Color::BLACK);

        draw_piano_keys(window_width, window_height, &all_keys, &piano_props, d);
    }
}

fn draw_piano_keys(
    window_width: f32,
    window_height: f32,
    all_keys: &Vec<piano::PianoKey>,
    piano_props: &PianoProps,
    mut d: RaylibDrawHandle<'_>,
) {
    let key_spacing = piano_props.key_spacing;
    let white_key_width = piano_props.white_key_width;
    let white_key_height = piano_props.white_key_height;
    let black_key_width = piano_props.black_key_width;
    let black_key_height = piano_props.black_key_height;
    // Single drawing loop
    for (i, key) in all_keys.iter().enumerate() {
        let (x_pos, y_pos, width, height, color, text_color) = if key.is_white {
            // White key positioning
            let x = i as f32 * (white_key_width + key_spacing) - window_width / 2.0
                + white_key_width / 2.0;
            let y = (white_key_height / 2.0) + window_height / 2.0 - white_key_height;
            (
                x,
                y,
                white_key_width,
                white_key_height,
                if key.is_pressed {
                    Color::GRAY
                } else {
                    Color::WHITE
                },
                Color::BLACK,
            )
        } else if let Some(white_idx) = key.white_key_index {
            // Black key positioning
            let x = (white_idx as f32 + 0.5) * (white_key_width + key_spacing) - window_width / 2.0
                + black_key_width / 2.0;
            let y = (black_key_height / 2.0) + window_height / 2.0 - black_key_height;
            (
                x,
                y,
                black_key_width,
                black_key_height,
                if key.is_pressed {
                    Color::DARKGRAY
                } else {
                    Color::BLACK
                },
                Color::WHITE,
            )
        } else {
            continue;
        };

        // Draw key
        d.draw_rectangle(
            (x_pos + window_width / 2.0 - width / 2.0) as i32,
            (y_pos + window_height / 2.0 - height / 2.0) as i32,
            width as i32,
            height as i32,
            color,
        );

        // Draw label
        d.draw_text(
            &key.label,
            (x_pos + window_width / 2.0 - width / 2.0 + 5.0) as i32,
            (y_pos + window_height / 2.0 - height / 2.0) as i32,
            10,
            text_color,
        );
    }
}

fn initialize_piano_dimensions(
    window_width: f32,
    window_height: f32,
    all_keys: &Vec<piano::PianoKey>,
) -> PianoProps {
    let num_white_keys = all_keys.iter().filter(|k| k.is_white).count() as f32;

    let key_size_relative_to_screen = 0.1;
    let black_key_width_ratio = 1.;
    let black_key_height_ratio = 0.6;

    // Spacing between keys
    let key_spacing = 1.;

    let white_key_width = (window_width / num_white_keys) - key_spacing;
    let white_key_height = window_height * key_size_relative_to_screen;
    let black_key_width = (white_key_width * black_key_width_ratio) - key_spacing;
    let black_key_height = white_key_height * black_key_height_ratio;
    PianoProps {
        key_spacing,
        white_key_width,
        white_key_height,
        black_key_width,
        black_key_height,
    }
}
