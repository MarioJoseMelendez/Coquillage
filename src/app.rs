use std::sync::Arc;
use std::time::Instant;

use anyhow::Result;
use glam::Vec2;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

use crate::camera::Camera;
use crate::player::Player;
use crate::renderer::Renderer;
use crate::world;

#[derive(Default)]
struct Input {
    up: bool,
    down: bool,
    left: bool,
    right: bool,
}

pub struct App {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    camera: Camera,
    player: Option<Player>,
    input: Input,
    last_frame: Instant,
}

impl App {
    fn new() -> Self {
        Self {
            window: None,
            renderer: None,
            camera: Camera::new(world::world_pixel_size()),
            player: None,
            input: Input::default(),
            last_frame: Instant::now(),
        }
    }

    fn move_axis(&self) -> Vec2 {
        let mut delta = Vec2::ZERO;
        if self.input.left {
            delta.x -= 1.0;
        }
        if self.input.right {
            delta.x += 1.0;
        }
        if self.input.up {
            delta.y -= 1.0;
        }
        if self.input.down {
            delta.y += 1.0;
        }
        if delta.length_squared() > 0.0 {
            delta = delta.normalize();
        }
        delta
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.renderer.is_some() {
            return;
        }

        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("Coquillage — Phase 8")
                        .with_inner_size(winit::dpi::LogicalSize::new(1280.0, 720.0))
                        .with_min_inner_size(winit::dpi::LogicalSize::new(640.0, 360.0)),
                )
                .expect("create window"),
        );
        let mut renderer = pollster::block_on(Renderer::new(window.clone()))
            .expect("initialize renderer");
        let size = window.inner_size();
        renderer.resize(size.width, size.height);
        let player = Player::new(renderer.spawn_point());
        self.camera.look_at(player.center());
        self.window = Some(window);
        self.renderer = Some(renderer);
        self.player = Some(player);
        self.last_frame = Instant::now();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let Some(renderer) = self.renderer.as_mut() {
                    renderer.resize(size.width, size.height);
                }
            }
            WindowEvent::ScaleFactorChanged { .. } => {
                if let (Some(window), Some(renderer)) = (self.window.as_ref(), self.renderer.as_mut())
                {
                    let size = window.inner_size();
                    renderer.resize(size.width, size.height);
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state,
                        repeat: false,
                        ..
                    },
                ..
            } => {
                let pressed = state == ElementState::Pressed;
                match code {
                    KeyCode::Escape => {
                        if pressed {
                            event_loop.exit();
                        }
                    }
                    KeyCode::KeyW | KeyCode::ArrowUp => self.input.up = pressed,
                    KeyCode::KeyS | KeyCode::ArrowDown => self.input.down = pressed,
                    KeyCode::KeyA | KeyCode::ArrowLeft => self.input.left = pressed,
                    KeyCode::KeyD | KeyCode::ArrowRight => self.input.right = pressed,
                    KeyCode::KeyQ if pressed => {
                        if let Some(renderer) = self.renderer.as_mut() {
                            renderer.nudge_sun(-0.18);
                        }
                    }
                    KeyCode::KeyE if pressed => {
                        if let Some(renderer) = self.renderer.as_mut() {
                            renderer.nudge_sun(0.18);
                        }
                    }
                    KeyCode::KeyR if pressed => {
                        if let Some(renderer) = self.renderer.as_mut() {
                            renderer.toggle_rain();
                        }
                    }
                    _ => {}
                }
            }
            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                let dt = (now - self.last_frame).as_secs_f32().min(0.05);
                self.last_frame = now;
                let axis = self.move_axis();
                if let (Some(player), Some(renderer)) =
                    (self.player.as_mut(), self.renderer.as_ref())
                {
                    player.tick(axis, dt, |x, y, w, h| renderer.blocked_aabb(x, y, w, h));
                }
                if let Some(player) = self.player.as_ref() {
                    self.camera.follow(player.center(), dt);
                }

                let fps_title = if let (Some(renderer), Some(player)) =
                    (self.renderer.as_mut(), self.player.as_ref())
                {
                    if let Err(err) = renderer.render(&self.camera, player, dt) {
                        log::error!("render failed: {err:#}");
                        event_loop.exit();
                    }
                    renderer.take_fps()
                } else {
                    None
                };
                if let Some(fps) = fps_title {
                    if let Some(window) = self.window.as_ref() {
                        window.set_title(&format!("Coquillage — Phase 8  ·  {fps:.0} fps"));
                    }
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }
}

pub fn run() -> Result<()> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App::new();
    event_loop.run_app(&mut app)?;
    Ok(())
}
