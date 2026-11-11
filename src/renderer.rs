use std::sync::Arc;

use anyhow::{Context, Result};
use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;
use winit::window::Window;

use crate::assets::Frame;
use crate::camera::Camera;
use crate::player::Player;
use crate::world::{
    self, Instance, Layers, Vertex, GRID_HEIGHT, LOGICAL_HEIGHT, LOGICAL_WIDTH, MASK_PAD_WIDTH,
};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct LightUniform {
    sun_dir: [f32; 2],
    elevation: f32,
    time: f32,
    cam_xy: [f32; 2],
    day_hour: f32,
    sat: f32,
    shadow: [f32; 4],
    sun_tint: [f32; 4],
    fog: [f32; 4],
    disc: [f32; 4],
    rim: [f32; 4],
    wind: [f32; 2],
    rain: f32,
    _weather_pad: f32,
}

#[derive(Clone, Copy)]
struct Grade {
    shadow: [f32; 3],
    light: [f32; 3],
    fog: [f32; 3],
    disc: [f32; 3],
    rim: [f32; 3],
    sat: f32,
    haze: f32,
    shadow_str: f32,
}

fn mix3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    let t = t.clamp(0.0, 1.0);
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

fn mix_grade(a: Grade, b: Grade, t: f32) -> Grade {
    let t = t.clamp(0.0, 1.0);
    let s = t * t * (3.0 - 2.0 * t);
    Grade {
        shadow: mix3(a.shadow, b.shadow, s),
        light: mix3(a.light, b.light, s),
        fog: mix3(a.fog, b.fog, s),
        disc: mix3(a.disc, b.disc, s),
        rim: mix3(a.rim, b.rim, s),
        sat: a.sat + (b.sat - a.sat) * s,
        haze: a.haze + (b.haze - a.haze) * s,
        shadow_str: a.shadow_str + (b.shadow_str - a.shadow_str) * s,
    }
}

fn rgb8(r: u8, g: u8, b: u8) -> [f32; 3] {
    [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0]
}

fn grade_at(hour: f32) -> Grade {
    // Keys follow 2D day/night practice: grade the world (not a black veil).
    // Noon stays close to the authored palette. Night is actually dark + moonlight.
    // Warmth lives in dawn/golden/sunset only. Shadows stay purple (LEAF_SH).
    let leaf_hi = rgb8(186, 196, 72);
    let leaf_sh = rgb8(56, 40, 72);
    let gold = rgb8(236, 214, 70);
    let dirt_hi = rgb8(176, 128, 78);
    let bark_hi = rgb8(156, 112, 58);
    let water_hi = rgb8(168, 206, 186);
    let water_mid = rgb8(46, 112, 120);
    let water_sh = rgb8(44, 36, 78);
    let sand_hi = rgb8(228, 212, 150);
    let birch = rgb8(214, 214, 204);
    let petal = rgb8(214, 92, 120);

    let midnight = Grade {
        shadow: [0.40, 0.38, 0.62],
        light: [0.58, 0.70, 0.90],
        fog: water_sh,
        disc: mix3(birch, water_hi, 0.40),
        rim: leaf_sh,
        sat: 0.86,
        haze: 0.11,
        shadow_str: 0.26,
    };
    let late_night = Grade {
        shadow: [0.44, 0.42, 0.64],
        light: [0.62, 0.74, 0.92],
        fog: mix3(water_sh, water_mid, 0.30),
        disc: mix3(birch, water_hi, 0.22),
        rim: water_sh,
        sat: 0.88,
        haze: 0.09,
        shadow_str: 0.30,
    };
    let dawn = Grade {
        shadow: [0.58, 0.48, 0.70],
        light: [1.06, 0.90, 0.86],
        fog: mix3(water_hi, petal, 0.22),
        disc: mix3(gold, petal, 0.30),
        rim: mix3(dirt_hi, petal, 0.30),
        sat: 1.04,
        haze: 0.055,
        shadow_str: 0.52,
    };
    let sunrise = Grade {
        shadow: [0.62, 0.52, 0.70],
        light: [1.12, 1.02, 0.78],
        fog: mix3(sand_hi, dirt_hi, 0.28),
        disc: gold,
        rim: bark_hi,
        sat: 1.05,
        haze: 0.045,
        shadow_str: 0.68,
    };
    let morning = Grade {
        shadow: [0.78, 0.74, 0.90],
        light: [1.06, 1.08, 0.94],
        fog: mix3(water_hi, sand_hi, 0.30),
        disc: mix3(gold, leaf_hi, 0.45),
        rim: dirt_hi,
        sat: 1.02,
        haze: 0.022,
        shadow_str: 0.84,
    };
    let noon = Grade {
        shadow: [0.84, 0.82, 0.92],
        light: [1.05, 1.07, 0.96],
        fog: mix3(water_hi, sand_hi, 0.18),
        disc: mix3(gold, leaf_hi, 0.55),
        rim: gold,
        sat: 0.98,
        haze: 0.012,
        shadow_str: 0.92,
    };
    let afternoon = Grade {
        shadow: [0.76, 0.70, 0.88],
        light: [1.08, 1.04, 0.88],
        fog: mix3(sand_hi, dirt_hi, 0.18),
        disc: gold,
        rim: bark_hi,
        sat: 1.02,
        haze: 0.028,
        shadow_str: 0.80,
    };
    let golden = Grade {
        shadow: [0.58, 0.46, 0.72],
        light: [1.14, 1.04, 0.72],
        fog: mix3(sand_hi, gold, 0.18),
        disc: gold,
        rim: bark_hi,
        sat: 1.08,
        haze: 0.05,
        shadow_str: 0.72,
    };
    let sunset = Grade {
        shadow: [0.50, 0.40, 0.68],
        light: [1.10, 0.88, 0.62],
        fog: mix3(dirt_hi, leaf_sh, 0.35),
        disc: gold,
        rim: mix3(bark_hi, leaf_sh, 0.35),
        sat: 1.06,
        haze: 0.07,
        shadow_str: 0.58,
    };
    let dusk = Grade {
        shadow: [0.44, 0.40, 0.64],
        light: [0.72, 0.70, 0.90],
        fog: mix3(water_sh, water_mid, 0.40),
        disc: mix3(birch, water_hi, 0.45),
        rim: leaf_sh,
        sat: 0.90,
        haze: 0.09,
        shadow_str: 0.34,
    };
    let evening = Grade {
        shadow: [0.40, 0.38, 0.62],
        light: [0.60, 0.68, 0.88],
        fog: water_sh,
        disc: mix3(birch, water_hi, 0.30),
        rim: leaf_sh,
        sat: 0.87,
        haze: 0.10,
        shadow_str: 0.28,
    };
    let keys = [
        (0.0, midnight),
        (3.5, late_night),
        (5.4, dawn),
        (6.5, sunrise),
        (8.5, morning),
        (12.0, noon),
        (15.2, afternoon),
        (17.0, golden),
        (18.2, sunset),
        (19.6, dusk),
        (21.5, evening),
        (24.0, midnight),
    ];
    let h = hour.rem_euclid(24.0);
    for pair in keys.windows(2) {
        let (h0, g0) = pair[0];
        let (h1, g1) = pair[1];
        if h >= h0 && h <= h1 {
            let t = if h1 > h0 { (h - h0) / (h1 - h0) } else { 0.0 };
            return mix_grade(g0, g1, t);
        }
    }
    midnight
}

fn vec3_pad(c: [f32; 3], w: f32) -> [f32; 4] {
    [c[0], c[1], c[2], w]
}

pub struct Renderer {
    #[allow(dead_code)]
    pub window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    surface_configured: bool,
    _world_texture: wgpu::Texture,
    world_view: wgpu::TextureView,
    _atlas_texture: wgpu::Texture,
    _normal_texture: wgpu::Texture,
    _cover_texture: wgpu::Texture,
    camera_buffer: wgpu::Buffer,
    world_bind_group: wgpu::BindGroup,
    present_bind_group: wgpu::BindGroup,
    world_pipeline: wgpu::RenderPipeline,
    present_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
    instances: Vec<Instance>,
    cover: Vec<Instance>,
    player_frames: std::collections::HashMap<String, Frame>,
    layers: Layers,
    casters: Vec<world::ShadowCaster>,
    player_caster: usize,
    instance_count: u32,
    shadow_start: u32,
    shadow_count: u32,
    cover_start: u32,
    light_buffer: wgpu::Buffer,
    time: f32,
    hour_shift: f32,
    rain_on: bool,
    rain: f32,
    last_cam: [f32; 2],
    fps: f32,
    fps_acc: f32,
    fps_n: u32,
    fps_dirty: bool,
}

impl Renderer {
    pub async fn new(window: Arc<Window>) -> Result<Self> {
        let size = window.inner_size();
        let width = size.width.max(1);
        let height = size.height.max(1);

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..wgpu::InstanceDescriptor::new_without_display_handle()
        });
        let surface = instance
            .create_surface(window.clone())
            .context("create wgpu surface")?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
                ..Default::default()
            })
            .await
            .context("request GPU adapter")?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("zaralle-device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_defaults()
                    .using_resolution(adapter.limits()),
                memory_hints: Default::default(),
                ..Default::default()
            })
            .await
            .context("request GPU device")?;

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(wgpu::TextureFormat::is_srgb)
            .unwrap_or(surface_caps.formats[0]);
        let mut config = surface
            .get_default_config(&adapter, width, height)
            .context("surface default config")?;
        config.format = surface_format;
        config.present_mode = wgpu::PresentMode::AutoVsync;
        config.usage = wgpu::TextureUsages::RENDER_ATTACHMENT;

        let (world_texture, world_view) = create_world_target(&device);
        let terrain = world::build().context("build terrain atlas from PNG assets")?;
        let mut player_frames = std::collections::HashMap::new();
        for id in [
            "player_s", "player_s2", "player_n", "player_n2", "player_e", "player_e2",
            "player_w", "player_w2",
        ] {
            let frame = *terrain
                .atlas
                .frames
                .get(id)
                .with_context(|| format!("atlas missing `{id}`"))?;
            player_frames.insert(id.to_string(), frame);
        }
        let atlas_texture = device.create_texture_with_data(
            &queue,
            &wgpu::TextureDescriptor {
                label: Some("tile-atlas"),
                size: wgpu::Extent3d {
                    width: terrain.atlas.width,
                    height: terrain.atlas.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            &terrain.atlas.pixels,
        );
        let atlas_view = atlas_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let normal_texture = device.create_texture_with_data(
            &queue,
            &wgpu::TextureDescriptor {
                label: Some("normal-atlas"),
                size: wgpu::Extent3d {
                    width: terrain.atlas.width,
                    height: terrain.atlas.height,
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
            &terrain.atlas.normals,
        );
        let normal_view = normal_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let cover_texture = device.create_texture_with_data(
            &queue,
            &wgpu::TextureDescriptor {
                label: Some("rain-cover-mask"),
                size: wgpu::Extent3d {
                    width: MASK_PAD_WIDTH,
                    height: GRID_HEIGHT,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::R8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            &terrain.rain_mask,
        );
        let cover_view = cover_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("camera-uniform"),
            size: std::mem::size_of::<CameraUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let light_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("light-uniform"),
            size: std::mem::size_of::<LightUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let world_bind_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("world-bind-layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let present_bind_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("present-bind-layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                ],
            });

        let world_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("world-bind-group"),
            layout: &world_bind_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&atlas_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&normal_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: light_buffer.as_entire_binding(),
                },
            ],
        });
        let present_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("present-bind-group"),
            layout: &present_bind_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&world_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: light_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&cover_view),
                },
            ],
        });

        let world_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("world-shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/world.wgsl").into()),
        });
        let present_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("present-shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/present.wgsl").into()),
        });

        let world_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("world-pipeline-layout"),
                bind_group_layouts: &[Some(&world_bind_layout)],
                immediate_size: 0,
            });
        let present_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("present-pipeline-layout"),
                bind_group_layouts: &[Some(&present_bind_layout)],
                immediate_size: 0,
            });

        let vertex_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2],
        };
        let instance_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Instance>() as u64,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &wgpu::vertex_attr_array![
                2 => Float32x2,
                3 => Float32x2,
                4 => Float32x2,
                5 => Float32x2,
                6 => Float32,
                7 => Float32
            ],
        };

        let world_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("world-pipeline"),
            layout: Some(&world_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &world_shader,
                entry_point: Some("vs_main"),
                buffers: &[Some(vertex_layout), Some(instance_layout)],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &world_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        let present_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("present-pipeline"),
            layout: Some(&present_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &present_shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &present_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        let vertices = world::quad_vertices();
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("quad-vertices"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let cover_start = terrain.cover_start;
        let mut instances = terrain.instances;
        let cover = instances.split_off(cover_start as usize);
        let casters = terrain.casters;
        let player_caster = terrain.player_caster as usize;
        let layers = terrain.layers;
        let shadow_start = terrain.shadow_start;
        let shadow_count = terrain.shadow_count;
        let mut gpu_instances = instances.clone();
        gpu_instances.extend_from_slice(&cover);
        gpu_instances.push(Instance::zeroed());
        let instance_count = gpu_instances.len() as u32;
        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("tile-instances"),
            contents: bytemuck::cast_slice(&gpu_instances),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        Ok(Self {
            window,
            surface,
            device,
            queue,
            config,
            surface_configured: false,
            _world_texture: world_texture,
            world_view,
            _atlas_texture: atlas_texture,
            _normal_texture: normal_texture,
            _cover_texture: cover_texture,
            camera_buffer,
            world_bind_group,
            present_bind_group,
            world_pipeline,
            present_pipeline,
            vertex_buffer,
            instance_buffer,
            instances,
            cover,
            player_frames,
            layers,
            casters,
            player_caster,
            instance_count,
            shadow_start,
            shadow_count,
            cover_start,
            light_buffer,
            time: 0.0,
            hour_shift: 0.0,
            rain_on: true,
            rain: 1.0,
            last_cam: [f32::NAN, f32::NAN],
            fps: 0.0,
            fps_acc: 0.0,
            fps_n: 0,
            fps_dirty: false,
        })
    }

    pub fn nudge_sun(&mut self, delta: f32) {
        // Q/E skip one hour on the 24-min day.
        self.hour_shift += delta.signum();
    }

    pub fn toggle_rain(&mut self) {
        self.rain_on = !self.rain_on;
    }

    pub fn spawn_point(&self) -> glam::Vec2 {
        let world = world::world_pixel_size();
        let prefer = glam::Vec2::new(
            (world.x - LOGICAL_WIDTH).max(0.0) * 0.35 + LOGICAL_WIDTH * 0.5,
            (world.y - LOGICAL_HEIGHT).max(0.0) * 0.4 + LOGICAL_HEIGHT * 0.5,
        );
        self.layers.spawn_point(prefer, &self.cover)
    }

    pub fn blocked_aabb(&self, x: f32, y: f32, w: f32, h: f32) -> bool {
        self.layers.blocked_aabb(x, y, w, h)
    }

    pub fn take_fps(&mut self) -> Option<f32> {
        if self.fps_dirty {
            self.fps_dirty = false;
            Some(self.fps)
        } else {
            None
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
        self.surface_configured = true;
    }

    pub fn render(&mut self, camera: &Camera, player: &Player, dt: f32) -> Result<()> {
        let size = self.window.inner_size();
        if size.width != self.config.width || size.height != self.config.height {
            self.resize(size.width, size.height);
        }
        if !self.surface_configured {
            return Ok(());
        }

        self.time += dt;
        self.fps_acc += dt;
        self.fps_n += 1;
        if self.fps_acc >= 1.0 {
            self.fps = self.fps_n as f32 / self.fps_acc;
            self.fps_acc = 0.0;
            self.fps_n = 0;
            self.fps_dirty = true;
        }
        // 24 minutes = 24 hours. Start mid-morning so the pond is lit.
        let day_hour = (self.time / 60.0 + 10.0 + self.hour_shift).rem_euclid(24.0);
        let sun_up = (6.0..18.0).contains(&day_hour);
        let solar = if sun_up {
            (day_hour - 6.0) / 12.0
        } else {
            0.0
        };
        let moon_path = if sun_up {
            0.0
        } else if day_hour >= 18.0 {
            (day_hour - 18.0) / 12.0
        } else {
            (day_hour + 6.0) / 12.0
        };
        let sun_elev = if sun_up {
            (solar * std::f32::consts::PI).sin()
        } else {
            0.0
        };
        let moon_elev = if sun_up {
            0.0
        } else {
            (moon_path * std::f32::consts::PI).sin()
        };
        let az = if sun_up {
            -1.0 + solar * 2.0
        } else {
            -1.0 + moon_path * 2.0
        };
        let mut sun_dir = glam::Vec2::new(az, 0.72).normalize();
        sun_dir.x = sun_dir.x.abs().max(0.20);
        sun_dir.y = sun_dir.y.abs().max(0.45);
        sun_dir = sun_dir.normalize();
        let elevation = if sun_up {
            0.28 + sun_elev * 0.64
        } else {
            0.22 + moon_elev * 0.40
        };

        let cam = camera.position.to_array();
        if cam != self.last_cam {
            let uniform = CameraUniform {
                view_proj: camera.view_proj().to_cols_array_2d(),
            };
            self.queue
                .write_buffer(&self.camera_buffer, 0, bytemuck::bytes_of(&uniform));
            self.last_cam = cam;
        }
        let grade = grade_at(day_hour);
        let target_rain = if self.rain_on { 1.0 } else { 0.0 };
        self.rain += (target_rain - self.rain) * (1.0 - (-dt * 2.4).exp());
        let rain = self.rain.clamp(0.0, 1.0);
        let water_mid = [46.0 / 255.0, 112.0 / 255.0, 120.0 / 255.0];
        let gust = 0.72 + 0.28 * (self.time * 0.37).sin();
        let wind_amp = (0.55 + rain * 0.70) * gust;
        let fog = mix3(grade.fog, water_mid, rain * 0.40);
        let haze = grade.haze + rain * 0.035;
        let sat = grade.sat * (1.0 - rain * 0.06);
        let sun = mix3(grade.light, [0.90, 0.96, 1.02], rain * 0.22);
        let light = LightUniform {
            sun_dir: sun_dir.to_array(),
            elevation,
            time: self.time,
            cam_xy: camera.position.to_array(),
            day_hour,
            sat,
            shadow: vec3_pad(grade.shadow, haze),
            sun_tint: vec3_pad(sun, 0.0),
            fog: vec3_pad(fog, 0.0),
            disc: vec3_pad(grade.disc, 0.0),
            rim: vec3_pad(grade.rim, grade.shadow_str),
            wind: [0.92 * wind_amp, 0.08 * wind_amp],
            rain,
            _weather_pad: 0.0,
        };
        self.queue
            .write_buffer(&self.light_buffer, 0, bytemuck::bytes_of(&light));

        if let Some(caster) = self.casters.get_mut(self.player_caster) {
            let c = player.center();
            caster.origin = [c.x, player.foot_y() - 3.0];
        }
        world::update_dynamic_shadows(
            &mut self.instances,
            &self.casters,
            sun_dir,
            elevation,
            grade.shadow_str,
        );
        world::update_cover_fade(&mut self.cover, player.center());
        let frame_id = player.frame_id();
        let sprite = *self
            .player_frames
            .get(frame_id)
            .expect("player frame in atlas");
        let player_inst = world::instance_from_frame(player.pos.x, player.pos.y, &sprite);
        let mut actors = self.cover.clone();
        actors.push(player_inst);
        actors.sort_by(|a, b| {
            let ay = a.world_xy[1] + a.size_xy[1];
            let by = b.world_xy[1] + b.size_xy[1];
            ay.partial_cmp(&by)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(
                    b.size_xy[1]
                        .partial_cmp(&a.size_xy[1])
                        .unwrap_or(std::cmp::Ordering::Equal),
                )
        });
        let stride = std::mem::size_of::<Instance>() as u64;
        // Shadows move; cover+player are y-sorted each frame.
        if self.shadow_count > 0 {
            let start = self.shadow_start as usize;
            let end = start + self.shadow_count as usize;
            self.queue.write_buffer(
                &self.instance_buffer,
                start as u64 * stride,
                bytemuck::cast_slice(&self.instances[start..end]),
            );
        }
        self.queue.write_buffer(
            &self.instance_buffer,
            self.cover_start as u64 * stride,
            bytemuck::cast_slice(&actors),
        );
        self.instance_count = self.cover_start + actors.len() as u32;

        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame)
            | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame,
            wgpu::CurrentSurfaceTexture::Timeout
            | wgpu::CurrentSurfaceTexture::Occluded
            | wgpu::CurrentSurfaceTexture::Validation => return Ok(()),
            wgpu::CurrentSurfaceTexture::Outdated => {
                let size = self.window.inner_size();
                self.resize(size.width, size.height);
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Lost => anyhow::bail!("surface lost"),
        };
        let tex = frame.texture.size();
        if tex.width != self.config.width || tex.height != self.config.height {
            let size = self.window.inner_size();
            self.resize(size.width.max(tex.width), size.height.max(tex.height));
            return Ok(());
        }
        let frame_view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("frame-encoder"),
            });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("world-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.world_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.04,
                            g: 0.06,
                            b: 0.04,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                ..Default::default()
            });
            pass.set_pipeline(&self.world_pipeline);
            pass.set_bind_group(0, &self.world_bind_group, &[]);
            pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            pass.draw(0..6, 0..self.instance_count);
        }

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("present-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &frame_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.015,
                            g: 0.016,
                            b: 0.02,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                ..Default::default()
            });

            let (x, y, w, h) = present_viewport(tex.width, tex.height);
            pass.set_viewport(x, y, w, h, 0.0, 1.0);
            pass.set_pipeline(&self.present_pipeline);
            pass.set_bind_group(0, &self.present_bind_group, &[]);
            pass.draw(0..3, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        self.queue.present(frame);
        Ok(())
    }
}

fn create_world_target(device: &wgpu::Device) -> (wgpu::Texture, wgpu::TextureView) {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("world-target"),
        size: wgpu::Extent3d {
            width: LOGICAL_WIDTH as u32,
            height: LOGICAL_HEIGHT as u32,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    (texture, view)
}

fn present_viewport(window_w: u32, window_h: u32) -> (f32, f32, f32, f32) {
    let lw = LOGICAL_WIDTH;
    let lh = LOGICAL_HEIGHT;
    let w = window_w.max(1) as f32;
    let h = window_h.max(1) as f32;
    let int_scale = (window_w / lw as u32)
        .min(window_h / lh as u32)
        .max(1);
    let int_w = lw * int_scale as f32;
    let int_h = lh * int_scale as f32;
    let (dest_w, dest_h) = if (int_w * int_h) / (w * h) >= 0.78 {
        (int_w, int_h)
    } else {
        let s = (w / lw).min(h / lh);
        ((lw * s).floor().max(1.0), (lh * s).floor().max(1.0))
    };
    let x = ((w - dest_w) * 0.5).floor().max(0.0);
    let y = ((h - dest_h) * 0.5).floor().max(0.0);
    (x, y, dest_w.min(w), dest_h.min(h))
}

