use bytemuck::{Pod, Zeroable};

/// GPU-ready point: position (x, y, z) + packed RGBA color.
/// 16 bytes per point — minimal memory, aligned for GPU.
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct GpuPoint {
    pub position: [f32; 3],
    pub color: u32, // packed RGBA8
}

impl GpuPoint {
    #[inline]
    pub fn new(x: f32, y: f32, z: f32, r: u8, g: u8, b: u8) -> Self {
        Self {
            position: [x, y, z],
            color: (r as u32) | ((g as u32) << 8) | ((b as u32) << 16) | (0xFF << 24),
        }
    }
}

pub struct PointCloud {
    pub points: Vec<GpuPoint>,
}

impl PointCloud {
    pub fn new() -> Self {
        Self {
            points: Vec::new(),
        }
    }

    /// Generate a demo point cloud: a colored cube grid.
    pub fn generate_demo(count_per_axis: u32) -> Self {
        let total = (count_per_axis as usize).pow(3);
        let mut points = Vec::with_capacity(total);
        let step = 2.0 / count_per_axis as f32;

        for ix in 0..count_per_axis {
            let x = -1.0 + ix as f32 * step;
            let r = (ix as f32 / count_per_axis as f32 * 255.0) as u8;
            for iy in 0..count_per_axis {
                let y = -1.0 + iy as f32 * step;
                let g = (iy as f32 / count_per_axis as f32 * 255.0) as u8;
                for iz in 0..count_per_axis {
                    let z = -1.0 + iz as f32 * step;
                    let b = (iz as f32 / count_per_axis as f32 * 255.0) as u8;
                    points.push(GpuPoint::new(x, y, z, r, g, b));
                }
            }
        }

        Self { points }
    }

    #[inline]
    pub fn len(&self) -> u32 {
        self.points.len() as u32
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        bytemuck::cast_slice(&self.points)
    }
}
