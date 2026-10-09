//! wgpu + winit frontend. Usage: `nbs-player-wgpu [json-config] [song-path]`.
//! With no arguments it plays `song.nbsx` if present, else the bundled demo song, using a
//! default theme. Set `NBS_SCREENSHOT=name.png` to save frame 15 and exit.

use std::sync::Arc;
use std::time::Instant;

use nbs_player_core::app::App;
use nbs_player_core::audio::{AudioBackend, InstrumentBank, NullBackend};
use nbs_player_core::config::AppConfig;
use nbs_player_core::player::Player;
use nbs_player_core::render::InputState;
use nbs_player_core::theme::Theme;
use nbs_player_core::types::Vec2;
use nbs_player_core::{notes, song};
use winit::application::ApplicationHandler;
use winit::dpi::{LogicalSize, PhysicalPosition};
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Fullscreen, Window, WindowId};

mod background;
mod gfx;
mod text;

use gfx::Gfx;

/// Everything that exists once the window and GPU are up.
struct Running {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
    gfx: Gfx,
    app: App,
    audio: Box<dyn AudioBackend>,
    input: InputState,
    mouse: Vec2,
    last_frame: Instant,
    frame: u32,
}

/// What `resumed` needs to build `Running`; consumed on first use.
struct Pending {
    config: AppConfig,
    player: Player,
    bank: InstrumentBank,
}

struct Handler {
    pending: Option<Pending>,
    running: Option<Running>,
    screenshot: Option<String>,
}

fn create_audio() -> Box<dyn AudioBackend> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        match nbs_player_cpal::CpalBackend::new() {
            Ok(b) => return Box::new(b),
            Err(e) => log::error!("audio unavailable ({e}); running silent"),
        }
    }
    Box::new(NullBackend)
}

impl Handler {
    fn start(&mut self, event_loop: &ActiveEventLoop) -> Result<(), String> {
        let Pending {
            config,
            player,
            bank,
        } = self.pending.take().ok_or("already started")?;

        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("NBS Player")
                        .with_inner_size(LogicalSize::new(config.window_width, config.window_height)),
                )
                .map_err(|e| format!("window: {e}"))?,
        );

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_with_display_handle(Box::new(
            event_loop.owned_display_handle(),
        )));
        let surface = instance
            .create_surface(window.clone())
            .map_err(|e| format!("surface: {e}"))?;
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            compatible_surface: Some(&surface),
            ..Default::default()
        }))
        .map_err(|e| format!("adapter: {e}"))?;
        log::info!("wgpu adapter: {:?}", adapter.get_info());
        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
                .map_err(|e| format!("device: {e}"))?;

        // Colors are authored as sRGB bytes and blended in gamma space by the GL frontends;
        // a non-sRGB surface format keeps that look.
        let caps = surface.get_capabilities(&adapter);
        let format = [wgpu::TextureFormat::Bgra8Unorm, wgpu::TextureFormat::Rgba8Unorm]
            .into_iter()
            .find(|f| caps.formats.contains(f))
            .unwrap_or(caps.formats[0]);
        log::info!("surface format {format:?} (available: {:?})", caps.formats);
        let size = window.inner_size();
        let mut surface_config = surface
            .get_default_config(&adapter, size.width.max(1), size.height.max(1))
            .ok_or("surface not supported by adapter")?;
        surface_config.format = format;
        surface.configure(&device, &surface_config);

        let gfx = Gfx::new(device, queue, format, config.font_id.ttf_bytes(), &config.background)?;

        let scale = window.scale_factor() as f32;
        let logical = Vec2::new(size.width as f32 / scale, size.height as f32 / scale);
        let app = App::new(
            Theme::from_theme_config(&config.theme),
            player,
            logical,
            config.initial_volume.unwrap_or(0.5),
            (Instant::now().elapsed().subsec_nanos() % 1000) as f32 + rand_offset(),
            &gfx,
        );

        let mut audio = create_audio();
        bank.install(audio.as_mut());
        audio.set_master_volume(app.volume);

        window.request_redraw();
        self.running = Some(Running {
            window,
            surface,
            surface_config,
            gfx,
            app,
            audio,
            input: InputState::default(),
            mouse: Vec2::ZERO,
            last_frame: Instant::now(),
            frame: 0,
        });
        Ok(())
    }
}

/// Seed for the background animation phase, so every start looks different.
fn rand_offset() -> f32 {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    (nanos % 1_000_000) as f32 / 1000.0
}

impl Running {
    fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        self.surface_config.width = width;
        self.surface_config.height = height;
        self.surface.configure(&self.gfx.device, &self.surface_config);
    }

    /// Returns true when the app should exit.
    fn redraw(&mut self, screenshot: Option<&str>) -> bool {
        let now = Instant::now();
        let dt = (now - self.last_frame).as_secs_f32();
        self.last_frame = now;

        let size = self.window.inner_size();
        if size.width == 0 || size.height == 0 {
            return false;
        }
        let scale = self.window.scale_factor() as f32;
        let logical = Vec2::new(size.width as f32 / scale, size.height as f32 / scale);

        self.gfx.begin_frame((size.width, size.height), scale);
        self.input.mouse_pos = self.mouse;
        let events = self
            .app
            .frame(&self.input, dt, logical, &mut self.gfx, self.audio.as_mut());
        // edge-triggered inputs last one frame
        self.input.mouse_pressed = false;
        self.input.mouse_released = false;
        self.input.space_pressed = false;
        self.input.toggle_profiler = false;
        self.input.dump_profile = false;

        if events.toggle_fullscreen {
            let next = if self.window.fullscreen().is_some() {
                None
            } else {
                Some(Fullscreen::Borderless(None))
            };
            self.window.set_fullscreen(next);
        }

        if let (Some(path), 15) = (screenshot, self.frame) {
            let (w, h) = (size.width, size.height);
            let pixels = self.gfx.capture();
            match image::save_buffer(path, &pixels, w, h, image::ColorType::Rgba8) {
                Ok(()) => log::info!("saved {path} (mouse at {:?})", self.mouse),
                Err(e) => log::error!("screenshot failed: {e}"),
            }
            return true;
        }
        self.frame += 1;

        use wgpu::CurrentSurfaceTexture as T;
        match self.surface.get_current_texture() {
            T::Success(frame) | T::Suboptimal(frame) => {
                let view = frame.texture.create_view(&Default::default());
                self.gfx.render(&view);
                self.window.pre_present_notify();
                self.gfx.queue.present(frame);
            }
            T::Outdated | T::Lost => {
                self.surface.configure(&self.gfx.device, &self.surface_config);
            }
            T::Timeout | T::Occluded => {}
            other => log::warn!("surface texture unavailable: {other:?}"),
        }
        self.window.request_redraw();
        false
    }
}

impl ApplicationHandler for Handler {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.running.is_none() {
            if let Err(e) = self.start(event_loop) {
                log::error!("startup failed: {e}");
                event_loop.exit();
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        let Some(r) = self.running.as_mut() else {
            return;
        };
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => r.resize(size.width, size.height),
            WindowEvent::CursorMoved { position, .. } => {
                let PhysicalPosition { x, y } = position;
                let s = r.window.scale_factor();
                r.mouse = Vec2::new((x / s) as f32, (y / s) as f32);
            }
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } => match state {
                ElementState::Pressed => {
                    r.input.mouse_down = true;
                    r.input.mouse_pressed = true;
                }
                ElementState::Released => {
                    r.input.mouse_down = false;
                    r.input.mouse_released = true;
                }
            },
            WindowEvent::KeyboardInput { event, .. } if event.state == ElementState::Pressed && !event.repeat => {
                match event.physical_key {
                    PhysicalKey::Code(KeyCode::Space) => r.input.space_pressed = true,
                    PhysicalKey::Code(KeyCode::F3) => r.input.toggle_profiler = true,
                    PhysicalKey::Code(KeyCode::F4) => r.input.dump_profile = true,
                    _ => {}
                }
            }
            WindowEvent::RedrawRequested => {
                if r.redraw(self.screenshot.as_deref()) {
                    event_loop.exit();
                }
            }
            _ => {}
        }
    }
}

fn load_config() -> AppConfig {
    match std::env::args().nth(1) {
        Some(text) => serde_json::from_str(&text).unwrap_or_else(|e| {
            eprintln!("Error parsing JSON config: {e}");
            std::process::exit(1);
        }),
        None => AppConfig::demo(),
    }
}

fn main() {
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .with_module_level("wgpu_core", log::LevelFilter::Warn)
        .with_module_level("wgpu_hal", log::LevelFilter::Warn)
        .with_module_level("naga", log::LevelFilter::Warn)
        .init()
        .ok();

    let config = load_config();
    let path = std::env::args().nth(2).unwrap_or_else(|| "song.nbsx".into());
    let bytes = std::fs::read(&path).ok();
    let song_data = song::load_nbs_file(bytes.as_deref()).unwrap_or_else(|e| {
        eprintln!("Error loading song: {e}");
        std::process::exit(1);
    });
    let header = &song_data.song.header;

    let bank = InstrumentBank::new(&song_data.extra_sounds);
    let note_blocks = notes::get_note_blocks(&song_data.song, &bank.base_keys());
    let name = String::from_utf8(header.song_name.clone()).unwrap_or_else(|_| "Unknown".into());
    let author = String::from_utf8(header.song_author.clone()).unwrap_or_else(|_| "Unknown".into());
    let player = Player::new(
        format!("{name} - {author}"),
        header.tempo,
        header.song_length as usize,
        note_blocks,
        notes::generate_instrument_palette(),
    );

    let event_loop = EventLoop::new().expect("event loop");
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut handler = Handler {
        pending: Some(Pending {
            config,
            player,
            bank,
        }),
        running: None,
        screenshot: std::env::var("NBS_SCREENSHOT").ok(),
    };
    event_loop.run_app(&mut handler).expect("event loop");
}
