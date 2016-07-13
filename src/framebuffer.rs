#![allow(dead_code)]
use math::{Vec3f, Vec2u, Zero, Mat3f};
use math::vector_traits::*;
use std::borrow::Borrow;
use std::f32::{EPSILON, INFINITY};

#[derive(Debug, Clone)]
pub struct RgbFrameBuffer {
    buffer: Vec<Vec3f>,
    resolution: Vec2u,
}

#[derive(Debug, Clone)]
pub struct YxyFrameBuffer {
    buffer: Vec<Vec3f>,
    resolution: Vec2u,
}

#[derive(Debug, Clone)]
pub struct FrameLuminosity {
    min: f32,
    max: f32,
    log_avg: f32,
}

impl RgbFrameBuffer {
    pub fn new(resolution: Vec2u) -> RgbFrameBuffer {
        let n = resolution.x * resolution.y;
        RgbFrameBuffer {
            buffer: (0..n).map(|_| Zero::zero()).collect(),
            resolution: resolution
        }
    }

    pub fn add_color(&mut self, coords: (usize, usize), color: Vec3f) {
        let idx = self.idx(coords);
        self.buffer[idx] = self.buffer[idx] + color;
    }

    pub fn set_color(&mut self, coords: (usize, usize), color: Vec3f) {
        let idx = self.idx(coords);
        self.buffer[idx] = color;
    }

    pub fn idx(&self, coords: (usize, usize)) -> usize {
        assert!(coords.0 < self.resolution.x);
        assert!(coords.1 < self.resolution.y);
        coords.0 + coords.1 * self.resolution.x
    }

    pub fn as_slice(&self) -> &[Vec3f] {
        self.buffer.as_ref()
    }

    pub fn as_mut_slice(&mut self) -> &mut [Vec3f] {
        self.buffer.as_mut()
    }

    pub fn to_yxy_inplace(&self, frame: &mut YxyFrameBuffer, k: f32) -> FrameLuminosity {
        assert!(self.resolution == frame.resolution);

        let rgb_to_yxy = Mat3f::new(
            0.5141364, 0.3238786,  0.16036376,
            0.265068,  0.67023428, 0.06409157,
            0.0241188, 0.1228178,  0.84442666,
        );

        let mut max = EPSILON;
        let mut min = INFINITY;
        let mut sum = 0.0;

        for i in 0..self.buffer.len() {
            let result = rgb_to_yxy * (self.buffer[i] * k);

            let w = result.fold(|x, y| x + y);
            frame.buffer[i] = if w > 0.0 {
                Vec3f::new(result.y, result.x / w, result.y / w)
            } else {
                Zero::zero()
            };

            max = max.max(frame.buffer[i].x);
            min = min.min(frame.buffer[i].x);
            sum += (2.3e-5 + frame.buffer[i].x).ln();
        }

        let log_avg = sum / (self.resolution.x * self.resolution.y) as f32;

        FrameLuminosity { min: min, max: max, log_avg: log_avg }
    }

    pub unsafe fn as_yxy(self) -> YxyFrameBuffer {
        YxyFrameBuffer { resolution: self.resolution, buffer: self.buffer }
    }
}

impl YxyFrameBuffer {
    pub fn new(resolution: Vec2u) -> YxyFrameBuffer {
        let n = resolution.x * resolution.y;
        YxyFrameBuffer {
            buffer: (0..n).map(|_| Zero::zero()).collect(),
            resolution: resolution
        }
    }

    pub fn add_color(&mut self, coords: (usize, usize), color: Vec3f) {
        let idx = self.idx(coords);
        self.buffer[idx] = self.buffer[idx] + color;
    }

    pub fn set_color(&mut self, coords: (usize, usize), color: Vec3f) {
        let idx = self.idx(coords);
        self.buffer[idx] = color;
    }

    pub fn idx(&self, coords: (usize, usize)) -> usize {
        assert!(coords.0 < self.resolution.x);
        assert!(coords.1 < self.resolution.y);
        coords.0 + coords.1 * self.resolution.x
    }

    pub fn as_slice(&self) -> &[Vec3f] {
        self.buffer.as_ref()
    }

    pub fn as_mut_slice(&mut self) -> &mut [Vec3f] {
        self.buffer.as_mut()
    }

    pub fn into_rgb(self) -> RgbFrameBuffer {
        let mut buffer = self.buffer;

        let yxy_to_rgb = Mat3f::new(
             2.5651, -1.1665, -0.3986,
            -1.0217,  1.9777,  0.0439,
             0.0753, -0.2543,  1.1892,
        );

        for i in 0..buffer.len() {
            let mut result = [0.0, 0.0];
            let y = buffer[i].x;
            result[0] = buffer[i].y;
            result[1] = buffer[i].z;

            let c = if (y > EPSILON) && (result[0] > EPSILON) && (result[1] > EPSILON) {
                let x = (result[0] * y) / result[1];
                let z = (x / result[0]) - x - y;
                Vec3f::new(x, y, z)
            } else {
                Vec3f::new(EPSILON, y, EPSILON)
            };

            buffer[i] = yxy_to_rgb * c;
        }

        RgbFrameBuffer { resolution: self.resolution, buffer: buffer }
    }
}

impl Borrow<[Vec3f]> for RgbFrameBuffer {
    fn borrow(&self) -> &[Vec3f] {
        self.as_slice()
    }
}

impl Borrow<[Vec3f]> for YxyFrameBuffer {
    fn borrow(&self) -> &[Vec3f] {
        self.as_slice()
    }
}

fn bias(b: f32, x: f32) -> f32 {
    return x.powf(b);
}

pub fn log_tone_mapping(frame: &mut YxyFrameBuffer, lum: FrameLuminosity) {
    let bias_param = 0.7f32;
    let cont_param = 0.7;
    let exposure = 1.0;
    let exp_adapt = 1.0;
    let avg_lum = lum.log_avg.exp() / exp_adapt;
    let bias_p = bias_param.ln() / (0.5f32).ln();
    let lmax = lum.max.powf(1.0 / cont_param) / avg_lum;
    let divider = (lmax + 1.0).log10();
    for mut pix in frame.buffer.iter_mut() {
        pix.x = pix.x.powf(1.0 / cont_param);
        pix.x /= avg_lum;
        if exposure != 1.0 {
            pix.x *= exposure;
        }

        let interpol = (2.0 + bias(bias_p, pix.x / lmax) * 8.0).ln();
        pix.x = ((pix.x + 1.0).ln() / interpol) / divider;
    }
}
