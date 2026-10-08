use nbs_player_core::theme::Theme;
use raylib::prelude::GuiControl::*;
use raylib::prelude::GuiControlProperty::*;
use raylib::prelude::*;

use crate::render::color;

/// Applies the theme to raygui's button and slider styles.
pub fn set_gui_style(theme: &Theme, rl: &mut RaylibHandle) {
    let base_w = Color::WHITE.alpha(0.).color_to_int();
    let accent_color = color(theme.accent_color).color_to_int();
    let background_color = color(theme.background_color).color_to_int();
    let black_key_color = color(theme.black_key_color).color_to_int();
    let white_key = theme.white_key_color;
    // ------------------------------BUTTON STYLE------------------------------
    rl.gui_set_style(BUTTON, BASE_COLOR_NORMAL, base_w);
    rl.gui_set_style(BUTTON, BASE_COLOR_FOCUSED, base_w);
    rl.gui_set_style(BUTTON, BASE_COLOR_PRESSED, base_w);
    rl.gui_set_style(BUTTON, TEXT_COLOR_NORMAL, base_w);
    rl.gui_set_style(BUTTON, BORDER_COLOR_NORMAL, base_w);
    rl.gui_set_style(BUTTON, BORDER_COLOR_PRESSED, accent_color);
    rl.gui_set_style(BUTTON, BORDER_COLOR_FOCUSED, accent_color);
    // ------------------------------SLIDER STYLE------------------------------
    rl.gui_set_style(SLIDER, BASE_COLOR_NORMAL, background_color);
    rl.gui_set_style(SLIDER, BASE_COLOR_FOCUSED, black_key_color);
    rl.gui_set_style(
        SLIDER,
        BASE_COLOR_PRESSED,
        color(white_key.brightness(0.9)).color_to_int(),
    );
    rl.gui_set_style(SLIDER, BORDER_COLOR_NORMAL, base_w);
    rl.gui_set_style(SLIDER, BORDER_COLOR_PRESSED, accent_color);
    rl.gui_set_style(SLIDER, BORDER_COLOR_FOCUSED, accent_color);
    rl.gui_set_style(
        SLIDER,
        TEXT_COLOR_NORMAL,
        color(white_key.alpha(1.)).color_to_int(),
    );
    rl.gui_set_style(SLIDER, TEXT_COLOR_FOCUSED, accent_color);
    rl.gui_set_style(SLIDER, TEXT_COLOR_PRESSED, accent_color);
}
