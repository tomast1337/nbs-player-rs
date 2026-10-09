use raylib::prelude::*;
use nbs_player_core::config::BackgroundType;
use nbs_player_core::theme::Theme;
fn load_background_shader(
    rl: &mut raylib::core::RaylibHandle,
    thread: &raylib::core::RaylibThread,
    background: &BackgroundType,
) -> raylib::shaders::Shader {
    let shader_header = {
        if cfg!(target_arch = "wasm32") {
            "#version 100\n\nprecision mediump float;\n"
        } else {
            "#version 330 core\n"
        }
    };
    let fs_code = background.fragment_source();
    let fs_code = format!("{}{}", shader_header, fs_code);
    let background_shader = rl.load_shader_from_memory(thread, None, Some(&fs_code));
    return background_shader;
}

#[derive(Debug)]
pub struct Background {
    pub shader: Shader,

    speed: f32,

    pub i_time_loc: i32,
    pub i_resolution_loc: i32,
    pub speed_loc: i32,
    pub background_color_loc: i32,
    pub accent_color_loc: i32,
    pub text_color_loc: i32,
    pub white_key_color_loc: i32,
    pub black_key_color_loc: i32,
    pub white_text_key_color_loc: i32,
    pub black_text_key_color_loc: i32,
    pub shader_time: f32,
}
impl Background {
    pub fn new(
        rl: &mut raylib::core::RaylibHandle,
        thread: &raylib::core::RaylibThread,
        background: BackgroundType,
    ) -> Self {
        let background_shader = load_background_shader(rl, &thread, &background);
        let i_time_loc = background_shader.get_shader_location("iTime");
        let i_resolution_loc = background_shader.get_shader_location("iResolution");
        let speed_loc = background_shader.get_shader_location("speed");
        let background_color_loc = background_shader.get_shader_location("background_color");
        let accent_color_loc = background_shader.get_shader_location("accent_color");
        let text_color_loc = background_shader.get_shader_location("text_color");
        let white_key_color_loc = background_shader.get_shader_location("white_key_color");
        let black_key_color_loc = background_shader.get_shader_location("black_key_color");
        let white_text_key_color_loc =
            background_shader.get_shader_location("white_text_key_color");
        let black_text_key_color_loc =
            background_shader.get_shader_location("black_text_key_color");

        let speed = background.speed();

        Self {
            shader_time: rand::random::<f32>() * 1000.0,
            speed,
            shader: background_shader,
            i_time_loc,
            i_resolution_loc,
            speed_loc,
            background_color_loc,
            accent_color_loc,
            text_color_loc,
            white_key_color_loc,
            black_key_color_loc,
            white_text_key_color_loc,
            black_text_key_color_loc,
        }
    }

    pub fn draw(
        self: &mut Self,
        d: &mut RaylibDrawHandle<'_>,
        theme: &Theme,
        resolution: [f32; 2],
    ) {
        let mut shader = &mut self.shader;

        shader.set_shader_value(self.i_time_loc, self.shader_time);
        shader.set_shader_value(self.i_resolution_loc, resolution);
        shader.set_shader_value(self.speed_loc, self.speed);
        shader.set_shader_value(self.background_color_loc, theme.background_color.to_f32_rgb());
        shader.set_shader_value(self.accent_color_loc, theme.accent_color.to_f32_rgb());
        shader.set_shader_value(self.text_color_loc, theme.text_color.to_f32_rgb());
        shader.set_shader_value(self.white_key_color_loc, theme.white_key_color.to_f32_rgb());
        shader.set_shader_value(self.black_key_color_loc, theme.black_key_color.to_f32_rgb());
        shader.set_shader_value(self.white_text_key_color_loc, theme.white_text_key_color.to_f32_rgb());
        shader.set_shader_value(self.black_text_key_color_loc, theme.black_text_key_color.to_f32_rgb());

        let mut shader_mode_handle = d.begin_shader_mode(&mut shader);
        shader_mode_handle.draw_rectangle(
            0,
            0,
            shader_mode_handle.get_screen_width(),
            shader_mode_handle.get_screen_height(),
            Color::WHITE,
        );
    }
}
