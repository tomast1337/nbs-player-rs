//! wgpu + winit frontend for the NBS player, native and web.
//!
//! Native: `nbs-player-wgpu [json-config] [song-path]` (see `main.rs`).
//! Web: build the lib for `wasm32-unknown-unknown`, run wasm-bindgen over it and call the
//! exported `start(canvas_id, config_json, song_url)` (see `web/index.html`).
//! Set `NBS_SCREENSHOT=name.png` natively to save frame 15 and exit.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use nbs_player_core::app::App;
use nbs_player_core::audio::{AudioBackend, InstrumentBank, NullBackend};
use nbs_player_core::config::AppConfig;
use nbs_player_core::player::Player;
use nbs_player_core::render::InputState;
use nbs_player_core::theme::Theme;
use nbs_player_core::types::Vec2;
use nbs_player_core::{notes, song};
use web_time::Instant;
use winit::application::ApplicationHandler;
#[cfg(not(target_arch = "wasm32"))]
use winit::dpi::LogicalSize;
use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::ActiveEventLoop;
#[cfg(not(target_arch = "wasm32"))]
use winit::event_loop::{ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Fullscreen, Window, WindowId};

pub mod background;
mod gfx;
mod text;

use gfx::Gfx;

#[cfg(target_arch = "wasm32")]
mod web;

/// Everything that exists once the window and GPU are up.
struct Running {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
    gfx: Gfx,
    app: App,
    bank: InstrumentBank,
    audio: Box<dyn AudioBackend>,
    audio_ready: bool,
    input: InputState,
    mouse: Vec2,
    last_frame: Instant,
    frame: u32,
}

/// What window creation needs to build `Running`; consumed on first use.
struct Pending {
    config: AppConfig,
    player: Player,
    bank: InstrumentBank,
    #[cfg(target_arch = "wasm32")]
    canvas: web_sys::HtmlCanvasElement,
}

struct Handler {
    pending: Option<Pending>,
    /// Filled synchronously on native and when the async GPU setup finishes on the web.
    slot: Rc<RefCell<Option<Running>>>,
    screenshot: Option<String>,
}

/// Parse the song and set up the player, before any window or GPU exists.
fn prepare(config: &AppConfig, song_bytes: Option<&[u8]>) -> Result<(Player, InstrumentBank), String> {
    let song_data = song::load_nbs_file(song_bytes).map_err(|e| format!("loading song: {e}"))?;
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
    let _ = config;
    Ok((player, bank))
}

fn create_audio_backend() -> Box<dyn AudioBackend> {
    match nbs_player_cpal::CpalBackend::new() {
        Ok(b) => Box::new(b),
        Err(e) => {
            log::error!("audio unavailable ({e}); running silent");
            Box::new(NullBackend)
        }
    }
}

/// Seed for the background animation phase, so every start looks different.
fn rand_offset() -> f32 {
    let nanos = web_time::SystemTime::now()
        .duration_since(web_time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    (nanos % 1_000_000) as f32 / 1000.0
}

/// Create the surface, device, renderer and app. Async because the web has no blocking
/// adapter/device requests.
async fn build_running(
    descriptor: wgpu::InstanceDescriptor,
    window: Arc<Window>,
    pending: Pending,
) -> Result<Running, String> {
    let Pending { config, player, bank, .. } = pending;

    // On the web prefer WebGPU but fall back to WebGL2 when the browser has no adapter.
    #[cfg(target_arch = "wasm32")]
    let instance = wgpu::util::new_instance_with_webgpu_detection(descriptor).await;
    #[cfg(not(target_arch = "wasm32"))]
    let instance = wgpu::Instance::new(descriptor);

    let surface = instance
        .create_surface(window.clone())
        .map_err(|e| format!("surface: {e}"))?;
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            compatible_surface: Some(&surface),
            ..Default::default()
        })
        .await
        .map_err(|e| format!("adapter: {e}"))?;
    log::info!("wgpu adapter: {:?}", adapter.get_info());

    // WebGL2 cannot offer the full default limits.
    let limits = if cfg!(target_arch = "wasm32") {
        wgpu::Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits())
    } else {
        wgpu::Limits::default()
    };
    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
            required_limits: limits,
            ..Default::default()
        })
        .await
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
        rand_offset(),
        &gfx,
    );

    #[cfg_attr(target_arch = "wasm32", allow(unused_mut))]
    let mut running = Running {
        window,
        surface,
        surface_config,
        gfx,
        app,
        bank,
        audio: Box::new(NullBackend),
        audio_ready: false,
        input: InputState::default(),
        mouse: Vec2::ZERO,
        last_frame: Instant::now(),
        frame: 0,
    };
    // Browsers only start audio from a user gesture; the web build waits for the first one.
    #[cfg(not(target_arch = "wasm32"))]
    running.ensure_audio();
    running.window.request_redraw();
    Ok(running)
}

impl Running {
    /// Open the audio device (once) and hand it the instruments.
    fn ensure_audio(&mut self) {
        if self.audio_ready {
            return;
        }
        self.audio_ready = true;
        let mut audio = create_audio_backend();
        self.bank.install(audio.as_mut());
        audio.set_master_volume(self.app.volume);
        self.audio = audio;
    }

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
            self.window.request_redraw();
            return false;
        }
        // keep the surface in step with the window even if no Resized event arrived
        if size.width != self.surface_config.width || size.height != self.surface_config.height {
            self.resize(size.width, size.height);
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
            #[cfg(not(target_arch = "wasm32"))]
            {
                let pixels = self.gfx.capture();
                match image::save_buffer(path, &pixels, size.width, size.height, image::ColorType::Rgba8) {
                    Ok(()) => log::info!("saved {path} (mouse at {:?})", self.mouse),
                    Err(e) => log::error!("screenshot failed: {e}"),
                }
            }
            #[cfg(target_arch = "wasm32")]
            let _ = path;
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

impl Handler {
    fn start(&mut self, event_loop: &ActiveEventLoop) -> Result<(), String> {
        let pending = self.pending.take().ok_or("already started")?;

        let attributes = Window::default_attributes().with_title("NBS Player");
        // Native: the configured size. Web: the canvas keeps the size its CSS gives it.
        #[cfg(not(target_arch = "wasm32"))]
        let attributes = attributes.with_inner_size(LogicalSize::new(
            pending.config.window_width,
            pending.config.window_height,
        ));
        #[cfg(target_arch = "wasm32")]
        let attributes = {
            use winit::platform::web::WindowAttributesExtWebSys;
            attributes.with_canvas(Some(pending.canvas.clone()))
        };
        let window = Arc::new(
            event_loop
                .create_window(attributes)
                .map_err(|e| format!("window: {e}"))?,
        );

        let descriptor = wgpu::InstanceDescriptor::new_with_display_handle(Box::new(
            event_loop.owned_display_handle(),
        ));

        #[cfg(not(target_arch = "wasm32"))]
        {
            let running = pollster::block_on(build_running(descriptor, window, pending))?;
            *self.slot.borrow_mut() = Some(running);
        }
        #[cfg(target_arch = "wasm32")]
        {
            let slot = self.slot.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match build_running(descriptor, window, pending).await {
                    Ok(running) => *slot.borrow_mut() = Some(running),
                    Err(e) => log::error!("startup failed: {e}"),
                }
            });
        }
        Ok(())
    }
}

impl ApplicationHandler for Handler {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.pending.is_some() {
            if let Err(e) = self.start(event_loop) {
                log::error!("startup failed: {e}");
                event_loop.exit();
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        let mut slot = self.slot.borrow_mut();
        let Some(r) = slot.as_mut() else {
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
                    r.ensure_audio();
                    r.input.mouse_down = true;
                    r.input.mouse_pressed = true;
                }
                ElementState::Released => {
                    r.input.mouse_down = false;
                    r.input.mouse_released = true;
                }
            },
            WindowEvent::KeyboardInput { event, .. } if event.state == ElementState::Pressed && !event.repeat => {
                r.ensure_audio();
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

fn handler(pending: Pending) -> Handler {
    Handler {
        pending: Some(pending),
        slot: Rc::new(RefCell::new(None)),
        screenshot: screenshot_path(),
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn screenshot_path() -> Option<String> {
    std::env::var("NBS_SCREENSHOT").ok()
}

#[cfg(target_arch = "wasm32")]
fn screenshot_path() -> Option<String> {
    None
}

/// Run the player in a native window. Blocks until it closes.
#[cfg(not(target_arch = "wasm32"))]
pub fn run_native(config: AppConfig, song_bytes: Option<Vec<u8>>) -> Result<(), String> {
    let (player, bank) = prepare(&config, song_bytes.as_deref())?;
    let event_loop = EventLoop::new().map_err(|e| format!("event loop: {e}"))?;
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut handler = handler(Pending { config, player, bank });
    event_loop.run_app(&mut handler).map_err(|e| format!("event loop: {e}"))
}
