use bevy::prelude::*;

use crate::AppState;

#[derive(Component)]
pub struct PianoKey {
    pub key: u8,
    pub is_pressed: bool,
}

// Setup the piano keyboard
pub fn setup_piano(mut commands: Commands, app_state: Res<AppState>) {
    commands.spawn(Camera2d);

    let white_key_width = app_state.white_key_width;
    let white_key_height = app_state.white_key_height;
    let black_key_width = app_state.black_key_width;
    let black_key_height = app_state.black_key_height;
    let key_spacing = app_state.key_spacing;
    let black_keys = &app_state.black_keys;
    let white_keys = &app_state.white_keys;
    let window_width = app_state.window_width;
    let window_height = app_state.window_height;

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
                key: *midi_note as u8,
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
        let x_pos = *midi_note as f32 * (white_key_width + key_spacing)
            - window_width / 2.0
            - white_key_width / 2.0
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
                key: *midi_note as u8,
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
