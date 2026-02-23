pub mod camera;
pub mod point_cloud;
pub mod renderer;

#[cfg(feature = "python")]
mod python_bindings {
    use numpy::PyReadonlyArray2;
    use pyo3::prelude::*;

    use crate::camera::Camera;
    use crate::point_cloud::{GpuPoint, PointCloud};
    use crate::renderer::Renderer;

    use std::sync::Arc;
    use winit::application::ApplicationHandler;
    use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
    use winit::event_loop::{ActiveEventLoop, EventLoop};
    use winit::window::{Window, WindowId};

    struct App {
        window: Option<Arc<Window>>,
        renderer: Option<Renderer>,
        camera: Camera,
        cloud: PointCloud,
        left_pressed: bool,
        right_pressed: bool,
        last_mouse: Option<(f64, f64)>,
    }

    impl App {
        fn from_cloud(cloud: PointCloud) -> Self {
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
            self.camera
                .set_aspect(size.width as f32, size.height as f32);

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
                        self.camera
                            .set_aspect(size.width as f32, size.height as f32);
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

    fn positions_to_points(
        positions: PyReadonlyArray2<f32>,
        colors: Option<PyReadonlyArray2<u8>>,
    ) -> PyResult<Vec<GpuPoint>> {
        let pos = positions.as_array();
        let n = pos.shape()[0];

        if pos.shape()[1] != 3 {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "positions must have shape (N, 3)",
            ));
        }

        if let Some(ref c) = colors {
            let c_arr = c.as_array();
            if c_arr.shape()[0] != n {
                return Err(pyo3::exceptions::PyValueError::new_err(
                    "colors length must match positions length",
                ));
            }
            let cols = c_arr.shape()[1];
            if cols != 3 && cols != 4 {
                return Err(pyo3::exceptions::PyValueError::new_err(
                    "colors must have shape (N, 3) or (N, 4)",
                ));
            }
        }

        let mut points = Vec::with_capacity(n);

        match colors {
            Some(ref c) => {
                let c_arr = c.as_array();
                let cols = c_arr.shape()[1];
                for i in 0..n {
                    let x = pos[[i, 0]];
                    let y = pos[[i, 1]];
                    let z = pos[[i, 2]];
                    let r = c_arr[[i, 0]];
                    let g = c_arr[[i, 1]];
                    let b = c_arr[[i, 2]];
                    let a = if cols == 4 { c_arr[[i, 3]] } else { 255 };
                    points.push(GpuPoint {
                        position: [x, y, z],
                        color: (r as u32)
                            | ((g as u32) << 8)
                            | ((b as u32) << 16)
                            | ((a as u32) << 24),
                    });
                }
            }
            None => {
                // Auto-color based on position (normalized to bounding box)
                let mut min = [f32::MAX; 3];
                let mut max = [f32::MIN; 3];
                for i in 0..n {
                    for j in 0..3 {
                        let v = pos[[i, j]];
                        if v < min[j] {
                            min[j] = v;
                        }
                        if v > max[j] {
                            max[j] = v;
                        }
                    }
                }
                let range = [
                    (max[0] - min[0]).max(1e-6),
                    (max[1] - min[1]).max(1e-6),
                    (max[2] - min[2]).max(1e-6),
                ];

                for i in 0..n {
                    let x = pos[[i, 0]];
                    let y = pos[[i, 1]];
                    let z = pos[[i, 2]];
                    let r = ((x - min[0]) / range[0] * 255.0) as u8;
                    let g = ((y - min[1]) / range[1] * 255.0) as u8;
                    let b = ((z - min[2]) / range[2] * 255.0) as u8;
                    points.push(GpuPoint::new(x, y, z, r, g, b));
                }
            }
        }

        Ok(points)
    }

    #[pyfunction]
    #[pyo3(signature = (positions, colors=None))]
    fn show(
        positions: PyReadonlyArray2<f32>,
        colors: Option<PyReadonlyArray2<u8>>,
    ) -> PyResult<()> {
        let points = positions_to_points(positions, colors)?;
        let cloud = PointCloud::from_points(points);

        env_logger::try_init().ok();
        log::info!("Showing {} points from Python", cloud.len());

        let event_loop = EventLoop::new().map_err(|e| {
            pyo3::exceptions::PyRuntimeError::new_err(format!("Failed to create event loop: {e}"))
        })?;
        let mut app = App::from_cloud(cloud);
        event_loop.run_app(&mut app).map_err(|e| {
            pyo3::exceptions::PyRuntimeError::new_err(format!("Event loop error: {e}"))
        })?;

        Ok(())
    }

    #[pyfunction]
    fn show_demo() -> PyResult<()> {
        let cloud = PointCloud::generate_demo(50);

        env_logger::try_init().ok();
        log::info!("Showing demo with {} points from Python", cloud.len());

        let event_loop = EventLoop::new().map_err(|e| {
            pyo3::exceptions::PyRuntimeError::new_err(format!("Failed to create event loop: {e}"))
        })?;
        let mut app = App::from_cloud(cloud);
        event_loop.run_app(&mut app).map_err(|e| {
            pyo3::exceptions::PyRuntimeError::new_err(format!("Event loop error: {e}"))
        })?;

        Ok(())
    }

    #[pymodule]
    pub fn point_cloud_viewer(m: &Bound<'_, PyModule>) -> PyResult<()> {
        m.add_function(wrap_pyfunction!(show, m)?)?;
        m.add_function(wrap_pyfunction!(show_demo, m)?)?;
        Ok(())
    }
}
