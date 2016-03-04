#![allow(dead_code)]
use math::{Vec3f, Vec2f, Zero, EPS_COSINE};
// use math::vector_traits::*;
use std::f32;
use geometry::{Frame, Ray};

#[derive(Debug, Clone, PartialEq, Copy)]
pub struct Material {
    pub diffuse: Vec3f,
    pub specular: Vec3f,
    pub phong_exponent: f32
}

#[derive(Debug, Clone)]
struct Probabilities {
    diffuse: f32,
    phong: f32,
    continuation: f32,
}

#[derive(Debug, Clone)]
pub struct Brdf {
    material: Material,
    frame: Frame,
    local_dir: Vec3f,
    prob: Probabilities,
}

impl Material {
    pub fn new_identity() -> Material {
        Material {
            diffuse: Zero::zero(),
            specular: Zero::zero(),
            phong_exponent: 0.0
        }
    }

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

impl Probabilities {
    fn new(mat: &Material) -> Probabilities {
        let albedo_diffuse = mat.albedo_diffuse();
        let albedo_specular = mat.albedo_specular();
        let total_albedo = mat.total_albedo();
        if total_albedo < 1.0e-9 {
            Probabilities {
                diffuse: 0.0,
                phong: 0.0,
                continuation: 0.0
            }
        } else {
            Probabilities {
                diffuse: albedo_diffuse / total_albedo,
                phong: albedo_specular / total_albedo,
                continuation: total_albedo.min(1.0)
            }
        }
    }
}

impl Brdf {
    pub fn new(mat: Material, frame: Frame, ray: &Ray) -> Option<Brdf> {
        let local_dir = frame.to_local(&-ray.dir);
        let prob = Probabilities::new(&mat);
        if local_dir.x.abs() < EPS_COSINE || prob.continuation == 0.0 {
            None
        } else {
            Some(Brdf {
                material: mat,
                local_dir: local_dir,
                frame: frame,
                prob: prob
            })
        }
    }

    pub fn continuation_prob(&self) -> f32 {
        self.prob.continuation
    }
}

fn luminance(a_rgb: &Vec3f) -> f32 {
    0.212671 * a_rgb.x + 0.715160 * a_rgb.y + 0.072169 * a_rgb.z
}

// returns vector and pdf
pub fn cos_hemisphere_sample_w(rnd: (f32, f32)) -> (Vec3f, f32) {
    let phi = rnd.0 * 2.0 * f32::consts::PI;
    let costheta = rnd.1.sqrt();
    let sintheta = (1.0 - costheta * costheta).sqrt();

    let ret = Vec3f::new(sintheta * phi.cos(), sintheta * phi.sin(), costheta);
    (ret, ret.z * f32::consts::FRAC_1_PI)
}

// returns vector and pdf
pub fn cos_hemisphere_pow_sample_w(phong_exp: f32, rnd: (f32, f32)) -> (Vec3f, f32) {
    let phi = rnd.0 * 2.0 * f32::consts::PI;
    let costheta = rnd.1.powf(1.0 / (phong_exp + 1.0));
    let sintheta = (1.0 - costheta * costheta).sqrt();

    let ret = Vec3f::new(sintheta * phi.cos(), sintheta * phi.sin(), costheta);
    (ret, (phong_exp + 1.0) * costheta.powf(phong_exp) * (0.5 * f32::consts::FRAC_1_PI))
}

pub fn uniform_triangle_sample(rnd: (f32, f32)) -> Vec2f {
    let term = rnd.0.sqrt();
    Vec2f::new(1.0 - term, rnd.1 * term)
}

pub fn uniform_sphere_sample_w(rnd: (f32, f32)) -> (Vec3f, f32) {
    let phi = rnd.0 * 2.0 * f32::consts::PI;
    let term2 = 2.0 * (rnd.1 - rnd.1 * rnd.1).sqrt();

    (Vec3f::new(phi.cos() * term2, phi.sin() * term2, 1.0 - 2.0 * rnd.1),
        uniform_sphere_pdf_w())
}

pub fn uniform_sphere_pdf_w() -> f32 {
    0.25 * f32::consts::FRAC_1_PI
}

pub fn pdf_w_to_a(pdf_w: f32, dist: f32, cos_there: f32) -> f32 {
    pdf_w * cos_there.abs() / (dist * dist)
}

pub fn pdf_a_to_w(pdf_a: f32, dist: f32, cos_there: f32) -> f32 {
    pdf_a * (dist * dist) / cos_there.abs()
}
