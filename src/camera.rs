use glam::{Mat4, Vec3};

pub struct Camera {
    /// Spherical coordinates around target
    pub yaw: f32,   // radians
    pub pitch: f32,  // radians
    pub distance: f32,
    pub target: Vec3,
    pub fov_y: f32,  // radians
    pub aspect: f32,
    pub near: f32,
    pub far: f32,
}

impl Camera {
    pub fn new(aspect: f32) -> Self {
        Self {
            yaw: 0.0,
            pitch: 0.3,
            distance: 4.0,
            target: Vec3::ZERO,
            fov_y: std::f32::consts::FRAC_PI_4,
            aspect,
            near: 0.01,
            far: 1000.0,
        }
    }

    pub fn eye(&self) -> Vec3 {
        let x = self.distance * self.pitch.cos() * self.yaw.cos();
        let y = self.distance * self.pitch.sin();
        let z = self.distance * self.pitch.cos() * self.yaw.sin();
        self.target + Vec3::new(x, y, z)
    }

    pub fn view_proj(&self) -> Mat4 {
        let view = Mat4::look_at_rh(self.eye(), self.target, Vec3::Y);
        let proj = Mat4::perspective_rh(self.fov_y, self.aspect, self.near, self.far);
        proj * view
    }

    /// Orbit: rotate around target
    pub fn orbit(&mut self, dx: f32, dy: f32) {
        self.yaw += dx * 0.005;
        self.pitch += dy * 0.005;
        // Clamp pitch to avoid gimbal lock
        self.pitch = self.pitch.clamp(-1.5, 1.5);
    }

    /// Zoom: change distance
    pub fn zoom(&mut self, delta: f32) {
        self.distance *= 1.0 - delta * 0.1;
        self.distance = self.distance.clamp(0.1, 500.0);
    }

    /// Pan: translate target
    pub fn pan(&mut self, dx: f32, dy: f32) {
        let forward = (self.target - self.eye()).normalize();
        let right = forward.cross(Vec3::Y).normalize();
        let up = right.cross(forward).normalize();
        let speed = self.distance * 0.002;
        self.target += right * (-dx * speed) + up * (dy * speed);
    }

    pub fn set_aspect(&mut self, width: f32, height: f32) {
        self.aspect = width / height;
    }
}
