#![allow(dead_code)]

use math::Vec3f;

pub struct Material {
    pub diffuse: Vec3f,
    pub specular: Vec3f,
    pub phong_exponent: f32,
    pub ior: f32
}

