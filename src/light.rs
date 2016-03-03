#![allow(dead_code)]
use math::{Vec2f, Vec3f};
// use math::vector_traits::*;
use geometry::BSphere;

pub struct Illumination {
    dir_to_light: Vec3f,
    dist_to_light: f32,
    direct_pdf: f32,
    intensity: Vec3f,
}

pub trait Light {
    fn illuminate(scene_bounds: &BSphere, rands: Vec2f, direct_pdf: f32) -> Illumination;
}

// struct
