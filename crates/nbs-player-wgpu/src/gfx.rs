//! wgpu implementation of the core's `Renderer`: batched textured quads (rects, sprites and
//! glyphs share one pipeline, rects use a 1x1 white texture) plus a fullscreen background
//! pass built from the shared GLSL shaders.

use std::borrow::Cow;

use nbs_player_core::config::BackgroundType;
use nbs_player_core::render::{Renderer, Sprite};
use nbs_player_core::theme::Theme;
use nbs_player_core::types::{Rect, Rgba, TextMeasure, Vec2};
use wgpu::util::DeviceExt;

use crate::background::{self, BackgroundUniforms};
use crate::text::{GlyphAtlas, Upload, ATLAS_SIZE};

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    pos: [f32; 2],
    uv: [f32; 2],
    color: [f32; 4],
}

const WHITE_TEX: usize = 0;
const SPRITE_TEX: usize = 1; // + index in Sprite::ALL
const ATLAS_TEX: usize = SPRITE_TEX + Sprite::ALL.len();

struct Batch {
    tex: usize,
    start: u32,
    end: u32,
}

pub struct Gfx {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub format: wgpu::TextureFormat,

    quad_pipeline: wgpu::RenderPipeline,
    bg_pipeline: wgpu::RenderPipeline,
    screen_buf: wgpu::Buffer,
    screen_bind: wgpu::BindGroup,
    bg_buf: wgpu::Buffer,
    bg_bind: wgpu::BindGroup,
    tex_binds: Vec<wgpu::BindGroup>,
    atlas_tex: wgpu::Texture,
    atlas: GlyphAtlas,

    vertex_buf: wgpu::Buffer,
    vertex_cap: u64,
    vertices: Vec<Vertex>,
    batches: Vec<Batch>,
    uploads: Vec<Upload>,
    background: Option<BackgroundUniforms>,
    bg_speed: f32,

    /// Physical pixels per logical pixel; the core works in logical coordinates.
    scale: f32,
    size_px: (u32, u32),
}

fn texture_bind(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    view: &wgpu::TextureView,
    sampler: &wgpu::Sampler,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("texture"),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
        ],
    })
}

fn rgba_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    w: u32,
    h: u32,
    data: &[u8],
) -> wgpu::Texture {
    device.create_texture_with_data(
        queue,
        &wgpu::TextureDescriptor {
            label: Some("image"),
            size: wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        },
        wgpu::util::TextureDataOrder::LayerMajor,
        data,
    )
}

impl Gfx {
    pub fn new(
        device: wgpu::Device,
        queue: wgpu::Queue,
        format: wgpu::TextureFormat,
        font_ttf: &[u8],
        background_kind: &BackgroundType,
    ) -> Result<Self, String> {
        let atlas = GlyphAtlas::new(font_ttf)?;

        // --- bind group layouts
        let uniform_layout = |label, visibility| {
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some(label),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            })
        };
        let screen_layout = uniform_layout("screen", wgpu::ShaderStages::VERTEX);
        let bg_layout = uniform_layout("background", wgpu::ShaderStages::FRAGMENT);
        let tex_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("texture"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        // --- buffers
        let screen_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("screen"),
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let screen_bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("screen"),
            layout: &screen_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: screen_buf.as_entire_binding(),
            }],
        });
        let bg_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("background"),
            size: std::mem::size_of::<BackgroundUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bg_bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("background"),
            layout: &bg_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: bg_buf.as_entire_binding(),
            }],
        });
        let vertex_cap = 64 * 1024;
        let vertex_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("vertices"),
            size: vertex_cap,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // --- textures: white, sprites, glyph atlas
        let nearest = device.create_sampler(&wgpu::SamplerDescriptor {
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        let linear = device.create_sampler(&wgpu::SamplerDescriptor {
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let mut tex_binds = Vec::new();
        let white = rgba_texture(&device, &queue, 1, 1, &[255, 255, 255, 255]);
        tex_binds.push(texture_bind(
            &device,
            &tex_layout,
            &white.create_view(&Default::default()),
            &nearest,
        ));
        for sprite in Sprite::ALL {
            let img = image::load_from_memory_with_format(sprite.png_bytes(), image::ImageFormat::Png)
                .map_err(|e| format!("sprite {sprite:?}: {e}"))?
                .to_rgba8();
            let tex = rgba_texture(&device, &queue, img.width(), img.height(), img.as_raw());
            tex_binds.push(texture_bind(
                &device,
                &tex_layout,
                &tex.create_view(&Default::default()),
                &nearest,
            ));
        }
        let atlas_tex = rgba_texture(
            &device,
            &queue,
            ATLAS_SIZE,
            ATLAS_SIZE,
            &vec![0u8; (ATLAS_SIZE * ATLAS_SIZE * 4) as usize],
        );
        tex_binds.push(texture_bind(
            &device,
            &tex_layout,
            &atlas_tex.create_view(&Default::default()),
            &linear,
        ));

        // --- pipelines
        let blend = Some(wgpu::BlendState::ALPHA_BLENDING);
        let target = [Some(wgpu::ColorTargetState {
            format,
            blend,
            write_mask: wgpu::ColorWrites::ALL,
        })];

        let wgsl = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("quads"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shaders.wgsl"))),
        });
        let quad_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("quads"),
            bind_group_layouts: &[Some(&screen_layout), Some(&tex_layout)],
            immediate_size: 0,
        });
        let quad_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("quads"),
            layout: Some(&quad_layout),
            vertex: wgpu::VertexState {
                module: &wgsl,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2, 2 => Float32x4],
                })],
            },
            fragment: Some(wgpu::FragmentState {
                module: &wgsl,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &target,
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        let glsl = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("background"),
            source: wgpu::ShaderSource::Glsl {
                shader: Cow::Owned(background::fragment_glsl(background_kind)),
                stage: wgpu::naga::ShaderStage::Fragment,
                defines: &[],
            },
        });
        let bg_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("background"),
            bind_group_layouts: &[Some(&bg_layout)],
            immediate_size: 0,
        });
        let bg_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("background"),
            layout: Some(&bg_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &wgsl,
                entry_point: Some("vs_fullscreen"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &glsl,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                targets: &target,
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        Ok(Self {
            device,
            queue,
            format,
            quad_pipeline,
            bg_pipeline,
            screen_buf,
            screen_bind,
            bg_buf,
            bg_bind,
            tex_binds,
            atlas_tex,
            atlas,
            vertex_buf,
            vertex_cap,
            vertices: Vec::new(),
            batches: Vec::new(),
            uploads: Vec::new(),
            background: None,
            bg_speed: background_kind.speed(),
            scale: 1.0,
            size_px: (1, 1),
        })
    }

    /// Start collecting a frame. `size_px` is the render target size in physical pixels.
    pub fn begin_frame(&mut self, size_px: (u32, u32), scale: f32) {
        self.size_px = (size_px.0.max(1), size_px.1.max(1));
        self.scale = scale;
        self.vertices.clear();
        self.batches.clear();
        self.background = None;
    }

    fn push_quad(&mut self, tex: usize, p0: [f32; 2], p1: [f32; 2], uv0: [f32; 2], uv1: [f32; 2], c: Rgba) {
        let color = [
            c.r as f32 / 255.0,
            c.g as f32 / 255.0,
            c.b as f32 / 255.0,
            c.a as f32 / 255.0,
        ];
        let v = |x: f32, y: f32, u: f32, w: f32| Vertex {
            pos: [x, y],
            uv: [u, w],
            color,
        };
        let start = self.vertices.len() as u32;
        self.vertices.extend_from_slice(&[
            v(p0[0], p0[1], uv0[0], uv0[1]),
            v(p1[0], p0[1], uv1[0], uv0[1]),
            v(p0[0], p1[1], uv0[0], uv1[1]),
            v(p1[0], p0[1], uv1[0], uv0[1]),
            v(p1[0], p1[1], uv1[0], uv1[1]),
            v(p0[0], p1[1], uv0[0], uv1[1]),
        ]);
        match self.batches.last_mut() {
            Some(b) if b.tex == tex => b.end = start + 6,
            _ => self.batches.push(Batch {
                tex,
                start,
                end: start + 6,
            }),
        }
    }

    /// Quad in logical coordinates.
    fn quad(&mut self, tex: usize, r: Rect, uv0: [f32; 2], uv1: [f32; 2], c: Rgba) {
        let s = self.scale;
        self.push_quad(
            tex,
            [r.x * s, r.y * s],
            [(r.x + r.w) * s, (r.y + r.h) * s],
            uv0,
            uv1,
            c,
        );
    }

    fn raster_px(&self, font_size: f32) -> u16 {
        (font_size * self.scale).round().clamp(1.0, 512.0) as u16
    }

    /// Draw the collected frame into `view`, clearing it first.
    pub fn render(&mut self, view: &wgpu::TextureView) {
        // glyph bitmaps rasterized this frame
        for u in self.uploads.drain(..) {
            self.queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &self.atlas_tex,
                    mip_level: 0,
                    origin: wgpu::Origin3d { x: u.x, y: u.y, z: 0 },
                    aspect: wgpu::TextureAspect::All,
                },
                &u.rgba,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(u.w * 4),
                    rows_per_image: Some(u.h),
                },
                wgpu::Extent3d {
                    width: u.w,
                    height: u.h,
                    depth_or_array_layers: 1,
                },
            );
        }

        let (w, h) = self.size_px;
        self.queue.write_buffer(
            &self.screen_buf,
            0,
            bytemuck::cast_slice(&[w as f32, h as f32, 0.0, 0.0]),
        );
        if let Some(bg) = &self.background {
            self.queue.write_buffer(&self.bg_buf, 0, bytemuck::bytes_of(bg));
        }

        let bytes: &[u8] = bytemuck::cast_slice(&self.vertices);
        if bytes.len() as u64 > self.vertex_cap {
            self.vertex_cap = (bytes.len() as u64).next_power_of_two();
            self.vertex_buf = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("vertices"),
                size: self.vertex_cap,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }
        if !bytes.is_empty() {
            self.queue.write_buffer(&self.vertex_buf, 0, bytes);
        }

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("frame") });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("frame"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            if self.background.is_some() {
                pass.set_pipeline(&self.bg_pipeline);
                pass.set_bind_group(0, &self.bg_bind, &[]);
                pass.draw(0..3, 0..1);
            }

            pass.set_pipeline(&self.quad_pipeline);
            pass.set_bind_group(0, &self.screen_bind, &[]);
            pass.set_vertex_buffer(0, self.vertex_buf.slice(..bytes.len() as u64));
            for b in &self.batches {
                pass.set_bind_group(1, &self.tex_binds[b.tex], &[]);
                pass.draw(b.start..b.end, 0..1);
            }
        }
        self.queue.submit([encoder.finish()]);
    }

    /// Render the current frame offscreen and return RGBA8 pixels (for screenshots).
    pub fn capture(&mut self) -> Vec<u8> {
        let (w, h) = self.size_px;
        let tex = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("capture"),
            size: wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        self.render(&tex.create_view(&Default::default()));

        let row = (w * 4).next_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);
        let buf = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("readback"),
            size: (row * h) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("readback") });
        encoder.copy_texture_to_buffer(
            tex.as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer: &buf,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(row),
                    rows_per_image: Some(h),
                },
            },
            wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
        );
        self.queue.submit([encoder.finish()]);

        let slice = buf.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |r| {
            let _ = tx.send(r);
        });
        self.device
            .poll(wgpu::PollType::wait_indefinitely())
            .expect("device poll");
        rx.recv().expect("map callback").expect("map buffer");

        let data = slice.get_mapped_range().expect("mapped range");
        let swap_rb = matches!(
            self.format,
            wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Bgra8UnormSrgb
        );
        let mut out = Vec::with_capacity((w * h * 4) as usize);
        for y in 0..h as usize {
            let line = &data[y * row as usize..y * row as usize + (w * 4) as usize];
            for px in line.chunks_exact(4) {
                if swap_rb {
                    out.extend_from_slice(&[px[2], px[1], px[0], px[3]]);
                } else {
                    out.extend_from_slice(px);
                }
            }
        }
        out
    }
}

impl TextMeasure for Gfx {
    fn measure_text(&self, text: &str, font_size: f32) -> Vec2 {
        let px = self.raster_px(font_size);
        let width: f32 = text.chars().map(|c| self.atlas.advance(c, px)).sum();
        Vec2::new(width / self.scale, font_size)
    }
}

impl Renderer for Gfx {
    fn draw_background(&mut self, time: f32, resolution: Vec2, theme: &Theme) {
        let res = [resolution.x * self.scale, resolution.y * self.scale];
        self.background = Some(BackgroundUniforms::new(res, time, self.bg_speed, theme));
    }

    fn draw_rect(&mut self, r: Rect, c: Rgba) {
        self.quad(WHITE_TEX, r, [0.0; 2], [1.0; 2], c);
    }

    fn draw_rect_outline(&mut self, r: Rect, t: f32, c: Rgba) {
        // drawn inward, like raylib
        let inner_h = (r.h - 2.0 * t).max(0.0);
        self.draw_rect(Rect::new(r.x, r.y, r.w, t), c);
        self.draw_rect(Rect::new(r.x, r.y + r.h - t, r.w, t), c);
        self.draw_rect(Rect::new(r.x, r.y + t, t, inner_h), c);
        self.draw_rect(Rect::new(r.x + r.w - t, r.y + t, t, inner_h), c);
    }

    fn draw_sprite(&mut self, sprite: Sprite, dst: Rect, tint: Rgba) {
        let i = Sprite::ALL.iter().position(|s| *s == sprite).unwrap();
        self.quad(SPRITE_TEX + i, dst, [0.0; 2], [1.0; 2], tint);
    }

    fn draw_text(&mut self, text: &str, pos: Vec2, font_size: f32, c: Rgba) {
        let px = self.raster_px(font_size);
        let baseline = (pos.y * self.scale + self.atlas.ascent(px)).round();
        let mut pen = pos.x * self.scale;
        let atlas_px = ATLAS_SIZE as f32;

        for ch in text.chars() {
            let mut uploads = Vec::new();
            let Some(g) = self.atlas.glyph(ch, px, &mut uploads) else {
                continue;
            };
            self.uploads.append(&mut uploads);
            if g.w > 0 && g.h > 0 {
                let x0 = (pen + g.xmin as f32).round();
                let y1 = baseline - g.ymin as f32;
                let y0 = y1 - g.h as f32;
                self.push_quad(
                    ATLAS_TEX,
                    [x0, y0],
                    [x0 + g.w as f32, y1],
                    [g.x as f32 / atlas_px, g.y as f32 / atlas_px],
                    [(g.x + g.w) as f32 / atlas_px, (g.y + g.h) as f32 / atlas_px],
                    c,
                );
            }
            pen += g.advance;
        }
    }
}
