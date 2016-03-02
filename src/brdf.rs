#![allow(dead_code)]

use math::Vec3f;
use math::vector_traits::*;
use std::f32;

pub struct Material {
    pub diffuse: Vec3f,
    pub specular: Vec3f,
    pub phong_exponent: f32
}

fn luminance(a_rgb: &Vec3f) -> f32 {
    0.212671 * a_rgb.x + 0.715160 * a_rgb.y + 0.072169 * a_rgb.z
}

impl Material {
    fn albedo_diffuse(&self) -> f32 {
        luminance(&self.diffuse)
    }

    fn albedo_specular(&self) -> f32 {
        luminance(&self.specular)
    }

    fn total_albedo(&self) -> f32 {
        self.albedo_specular() + self.albedo_diffuse()
    }
}

fn ortho(v: &Vec3f) -> Vec3f {
    // http://lolengine.net/blog/2013/09/21/picking-orthogonal-vector-combing-coconuts
    if v.x.abs() > v.z.abs() {
        Vec3f::new(-v.y, v.x, 0.0)
    } else {
        Vec3f::new(0.0, -v.z, v.y)
    }
}

fn get_cosine_lambert_sample(normal: Vec3f, rnd: (f32, f32)) -> Vec3f {
    let phi = rnd.0 * f32::consts::PI;
    let costheta = rnd.1.sqrt();
    let sintheta = (1.0 - costheta * costheta).sqrt();
    // Create vector aligned with z=(0,0,1)
    let sample = (sintheta * phi.cos(), sintheta * phi.sin(), costheta);

    // Create orthonormal basis around normal vector
    let o1 = ortho(&normal).normalize();
    let o2 = normal.cross(&o1).normalize();
    // Apply random vector to our basis
    o1 * sample.0 + o2 * sample.1 + normal * sample.2
}

fn get_phong_sample(normal: Vec3f, out_dir: Vec3f, phong_exp: f32, rnd: (f32, f32)) -> Vec3f {
    let reflect_dir = out_dir.reflect(&normal);

    let phi = rnd.0 * 2.0 * f32::consts::PI;
    let costheta = rnd.1.powf(1.0 / (phong_exp + 1.0));
    let sintheta = (1.0 - costheta * costheta).sqrt();
    // Create vector aligned with z=(0,0,1)
    let sample = (sintheta * phi.cos(), sintheta * phi.sin(), costheta);

    // Create orthonormal basis around reflection vector
    let o1 = ortho(&reflect_dir).normalize();
    let o2 = reflect_dir.cross(&o1).normalize();
    // Apply random vector to our basis
    o1 * sample.0 + o2 * sample.1 + reflect_dir * sample.2
}
