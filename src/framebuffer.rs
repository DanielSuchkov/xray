#![allow(dead_code)]
use math::{Vec3f, Vec2u, vec3_from_value};
use std::borrow::Borrow;

#[derive(Debug, Clone)]
pub struct FrameBuffer {
    frame: Vec<Vec3f>,
    resolution: Vec2u,
}

impl FrameBuffer {
    pub fn new(resolution: Vec2u) -> FrameBuffer {
        let n = resolution.x * resolution.y;
        FrameBuffer {
            frame: (0..n).map(|_| vec3_from_value(0.0)).collect(),
            resolution: resolution
        }
    }

    pub fn add_color(&mut self, coords: Vec2u, color: Vec3f) {
        let idx = self.idx(coords);
        assert!(idx != self.resolution.x * self.resolution.y);
        self.frame[idx] = self.frame[idx] + color;
    }

    pub fn set_color(&mut self, coords: Vec2u, color: Vec3f) {
        let idx = self.idx(coords);
        assert!(idx != self.resolution.x * self.resolution.y);
        self.frame[idx] = color;
    }

    pub fn idx(&self, coords: Vec2u) -> usize {
        coords.x + coords.y * self.resolution.x
    }

    pub fn as_slice(&self) -> &[Vec3f] {
        self.frame.as_ref()
    }
}

impl Borrow<[Vec3f]> for FrameBuffer {
    fn borrow(&self) -> &[Vec3f] {
        self.as_slice()
    }
}
