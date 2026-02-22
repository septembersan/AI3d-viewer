mod camera;
mod point_cloud;
mod renderer;

use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

use camera::Camera;
use point_cloud::PointCloud;
use renderer::Renderer;

struct App {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    camera: Camera,
    cloud: PointCloud,
    // Input state
    left_pressed: bool,
    right_pressed: bool,
    last_mouse: Option<(f64, f64)>,
}

impl App {
    fn new() -> Self {
        // 50^3 = 125,000 points as demo
        let cloud = PointCloud::generate_demo(50);
        log::info!("Generated {} points", cloud.len());

        Self {
            window: None,
            renderer: None,
            camera: Camera::new(16.0 / 9.0),
            cloud,
            left_pressed: false,
            right_pressed: false,
            last_mouse: None,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let attrs = Window::default_attributes()
            .with_title("Point Cloud Viewer")
            .with_inner_size(winit::dpi::LogicalSize::new(1280, 720));

        let window = Arc::new(event_loop.create_window(attrs).unwrap());
        let size = window.inner_size();
        self.camera.set_aspect(size.width as f32, size.height as f32);

        self.renderer = Some(Renderer::new(window.clone(), &self.cloud));
        self.window = Some(window);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }

            WindowEvent::Resized(size) => {
                if let Some(renderer) = &mut self.renderer {
                    renderer.resize(size.width, size.height);
                    self.camera.set_aspect(size.width as f32, size.height as f32);
                }
            }

            WindowEvent::MouseInput { state, button, .. } => {
                let pressed = state == ElementState::Pressed;
                match button {
                    MouseButton::Left => self.left_pressed = pressed,
                    MouseButton::Right => self.right_pressed = pressed,
                    _ => {}
                }
                if !pressed {
                    self.last_mouse = None;
                }
            }

            WindowEvent::CursorMoved { position, .. } => {
                if let Some((lx, ly)) = self.last_mouse {
                    let dx = position.x - lx;
                    let dy = position.y - ly;

                    if self.left_pressed {
                        self.camera.orbit(dx as f32, dy as f32);
                    }
                    if self.right_pressed {
                        self.camera.pan(dx as f32, dy as f32);
                    }

                    if let Some(window) = &self.window {
                        window.request_redraw();
                    }
                }
                self.last_mouse = Some((position.x, position.y));
            }

            WindowEvent::MouseWheel { delta, .. } => {
                let scroll = match delta {
                    MouseScrollDelta::LineDelta(_, y) => y,
                    MouseScrollDelta::PixelDelta(pos) => pos.y as f32 * 0.01,
                };
                self.camera.zoom(scroll);
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }

            WindowEvent::RedrawRequested => {
                if let Some(renderer) = &mut self.renderer {
                    renderer.render(&self.camera);
                }
            }

            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();
    let mut app = App::new();
    event_loop.run_app(&mut app).unwrap();
}
