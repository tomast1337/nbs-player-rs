use bevy::prelude::*;

// Component for piano keys
#[derive(Component)]
pub struct PianoKey {
    pub key: u8, // MIDI note number
    pub is_pressed: bool,
}

// Setup the piano keyboard
pub fn setup_keyboard(mut commands: Commands, windows: Query<&mut Window>) {
    let window = windows.single();
    let window_width = window.width();
    let window_height = window.height();

    commands.spawn(Camera2d);

    let white_keys = [
        ("A0", 21),
        ("B0", 23),
        ("C1", 24),
        ("D1", 26),
        ("E1", 28),
        ("F1", 29),
        ("G1", 31),
        ("A1", 33),
        ("B1", 35),
        ("C2", 36),
        ("D2", 38),
        ("E2", 40),
        ("F2", 41),
        ("G2", 43),
        ("A2", 45),
        ("B2", 47),
        ("C3", 48),
        ("D3", 50),
        ("E3", 52),
        ("F3", 53),
        ("G3", 55),
        ("A3", 57),
        ("B3", 59),
        ("C4", 60),
        ("D4", 62),
        ("E4", 64),
        ("F4", 65),
        ("G4", 67),
        ("A4", 69),
        ("B4", 71),
        ("C5", 72),
        ("D5", 74),
        ("E5", 76),
        ("F5", 77),
        ("G5", 79),
        ("A5", 81),
        ("B5", 83),
        ("C6", 84),
        ("D6", 86),
        ("E6", 88),
        ("F6", 89),
        ("G6", 91),
        ("A6", 93),
        ("B6", 95),
        ("C7", 96),
        ("D7", 98),
        ("E7", 100),
        ("F7", 101),
        ("G7", 103),
        ("A7", 105),
        ("B7", 107),
        ("C8", 108),
    ];

    let black_keys = [
        ("A#0", 1),
        ("C#1", 3),
        ("D#1", 4),
        ("F#1", 6),
        ("G#1", 7),
        ("A#1", 8),
        ("C#2", 10),
        ("D#2", 11),
        ("F#2", 13),
        ("G#2", 14),
        ("A#2", 15),
        ("C#3", 17),
        ("D#3", 18),
        ("F#3", 20),
        ("G#3", 21),
        ("A#3", 22),
        ("C#4", 24),
        ("D#4", 25),
        ("F#4", 27),
        ("G#4", 28),
        ("A#4", 29),
        ("C#5", 31),
        ("D#5", 32),
        ("F#5", 34),
        ("G#5", 35),
        ("A#5", 36),
        ("C#6", 38),
        ("D#6", 39),
        ("F#6", 41),
        ("G#6", 42),
        ("A#6", 43),
        ("C#7", 45),
        ("D#7", 46),
        ("F#7", 48),
        ("G#7", 49),
        ("A#7", 50),
    ];

    let key_size_relative_to_screen = 0.1;
    let black_key_width_ratio = 0.6;
    let black_key_height_ratio = 0.6;

    let num_white_keys = white_keys.len() as f32;
    let white_key_width = window_width / num_white_keys;
    let white_key_height = window_height * key_size_relative_to_screen;
    let black_key_width = white_key_width * black_key_width_ratio;
    let black_key_height = white_key_height * black_key_height_ratio;

    let key_spacing = 0.1; // Spacing between keys

    // Draw white keys
    for (i, (key, midi_note)) in white_keys.iter().enumerate() {
        let x_pos =
            i as f32 * (white_key_width + key_spacing) - window_width / 2.0 + white_key_width / 2.0; // Centered on screen

        let y_pos = -window_height / 2.0 + white_key_height / 2.0;

        commands.spawn((
            Sprite {
                color: Color::WHITE,
                custom_size: Some(Vec2::new(white_key_width, white_key_height)),
                ..default()
            },
            Transform::from_xyz(x_pos, y_pos, 0.0),
            GlobalTransform::default(),
            Visibility::default(),
            PianoKey {
                key: *midi_note,
                is_pressed: false,
            },
        ));

        // Add label
        commands.spawn((
            Text2d::new(key.to_string()),
            TextColor(Color::BLACK),
            TextFont {
                font_size: 30.0,
                ..default()
            },
            Transform::from_xyz(x_pos, -200.0, 1.1),
        ));
    }

    // Draw black keys (similar logic as above)
    for (key, midi_note) in black_keys.iter() {
        let x_pos = *midi_note as f32 * (white_key_width + key_spacing) - window_width / 2.0
            + white_key_width / 2.0
            - (black_key_width / 2.0);

        let y_pos = -window_height / 2.0 + black_key_height / 2.0;

        commands.spawn((
            Sprite {
                color: Color::BLACK,
                custom_size: Some(Vec2::new(black_key_width, black_key_height)),
                ..default()
            },
            Transform::from_xyz(x_pos, y_pos, 1.0), // Higher z-index
            GlobalTransform::default(),
            Visibility::default(),
            PianoKey {
                key: *midi_note,
                is_pressed: false,
            },
        ));

        // Add label
        commands.spawn((
            Text2d::new(key.to_string()),
            TextColor(Color::WHITE),
            TextFont {
                font_size: 30.0,
                ..default()
            },
            Transform::from_xyz(x_pos, -200.0, 1.1),
            GlobalTransform::default(),
            Visibility::default(),
        ));
    }
}

// Handle key press effects (dark tint or move down)
pub fn handle_key_press_effects(
    mut query: Query<(&mut Sprite, &mut Transform, &mut PianoKey)>,
    time: Res<Time>,
) {
    for (mut sprite, mut transform, piano_key) in &mut query {
        if piano_key.is_pressed {
            // Apply visual effect when the key is pressed
            sprite.color = Color::hsl(0.0, 0.0, 0.5); // Dark tint
            transform.translation.y -= 5.0 * time.delta_secs(); // Move down
        } else {
            // Reset the key state
            sprite.color = if piano_key.key % 12 == 1
                || piano_key.key % 12 == 3
                || piano_key.key % 12 == 6
                || piano_key.key % 12 == 8
                || piano_key.key % 12 == 10
            {
                Color::BLACK // Black keys
            } else {
                Color::WHITE // White keys
            };
            transform.translation.y = if piano_key.key % 12 == 1
                || piano_key.key % 12 == 3
                || piano_key.key % 12 == 6
                || piano_key.key % 12 == 8
                || piano_key.key % 12 == 10
            {
                -200.0 // Black keys
            } else {
                -250.0 // White keys
            };
        }
    }
}
