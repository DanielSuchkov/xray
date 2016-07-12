#![allow(dead_code)]
use math::{Vec3f, Vec2u, Zero};
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
}

impl YxyFrameBuffer {
    pub fn new(resolution: Vec2u) -> YxyFrameBuffer {
        let n = resolution.x * resolution.y;
        YxyFrameBuffer {
            buffer: (0..n).map(|_| Zero::zero()).collect(),
            resolution: resolution
        }
    }

    pub fn from_rgb(frame: RgbFrameBuffer) -> (YxyFrameBuffer, FrameLuminosity) {
        let resolution = frame.resolution;
        let mut arr = frame.buffer;
        let rgb_to_yxy = [
            [0.5141364, 0.3238786, 0.16036376],
            [0.265068, 0.67023428, 0.06409157],
            [0.0241188, 0.1228178, 0.84442666],
        ];

        let mut max = EPSILON;
        let mut min = INFINITY;
        let mut sum = 0.0;

        for mut curr in arr.iter_mut() {
            let mut result = [0.0, 0.0, 0.0];
            for i in 0..3 {
                result[i] += rgb_to_yxy[i][0] * curr.x;
                result[i] += rgb_to_yxy[i][1] * curr.y;
                result[i] += rgb_to_yxy[i][2] * curr.z;
            }
            let w = result[0] + result[1] + result[2];
            if w > 0.0 {
                curr.x = result[1];
                curr.y = result[0] / w;
                curr.z = result[1] / w;
            } else {
                curr.x = 0.0;
                curr.y = 0.0;
                curr.z = 0.0;
            }
            max = max.max(curr.x);
            min = min.min(curr.y);
            sum += (2.3e-5 + curr.x).ln();
        }
        let log_avg = sum / (resolution.x * resolution.y) as f32;

        (YxyFrameBuffer { buffer: arr, resolution: resolution },
         FrameLuminosity { min: min, max: max, log_avg: log_avg })
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
}

impl Borrow<[Vec3f]> for RgbFrameBuffer {
    fn borrow(&self) -> &[Vec3f] {
        self.as_slice()
    }
}
