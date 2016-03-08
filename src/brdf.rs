#![allow(dead_code)]
use math::{Vec3f, Vec2f, Zero, EPS_COSINE, EPS_PHONG};
use math::vector_traits::*;
use std::f32::consts::{FRAC_1_PI, PI};
use geometry::{Frame, Ray};
use std::ops::Add;

#[derive(Debug, Clone, PartialEq, Copy)]
pub struct Material {
    pub diffuse: Vec3f,
    pub specular: Vec3f,
    pub phong_exp: f32
}

#[derive(Debug, Clone)]
pub struct Brdf {
    material: Material,
    own_basis: Frame,
    out_dir_local: Vec3f, // "out" in physical meaning, in fact - incoming
    prob: Probabilities
}

pub struct BrdfSample {
    pub in_dir_world: Vec3f, // "in" in physical meaning, i.e. from light to eye
    pub cos_theta_in: f32,
    pub radiance_factor: Vec3f,
}

#[derive(Debug, Clone)]
struct Probabilities {
    diffuse: f32,
    phong: f32,
    continuation: f32,
}

impl Brdf {
    pub fn new(out_dir_world: &Vec3f, hit_normal: &Vec3f, material: &Material) -> Option<Brdf> {
        let own_basis = Frame::from_z(hit_normal);
        let out_dir_local = own_basis.to_local(&-*out_dir_world);
        if out_dir_local.z < EPS_COSINE {
            None
        } else {
            Some(Brdf {
                material: *material,
                own_basis: own_basis,
                out_dir_local: out_dir_local,
                prob: Probabilities::new(material)
            })
        }
    }

    pub fn sample(&self, rnd: (f32, f32, f32)) -> Option<BrdfSample> {
        let sample_rnds = (rnd.1, rnd.2);
        if rnd.0 < self.prob.diffuse {
            self.lambert_sample(sample_rnds)
        } else {
            self.phong_sample(sample_rnds)
        }
    }

    fn lambert_sample(&self, rnd: (f32, f32)) -> Option<BrdfSample> {
        let in_dir_local = cos_hemisphere_sample_w(rnd);
        let cos_theta_in = in_dir_local.z;
        if cos_theta_in < EPS_COSINE {
            None
        } else {
            let in_dir_world = self.own_basis.to_world(&in_dir_local);
            Some(BrdfSample {
                in_dir_world: in_dir_world,
                cos_theta_in: cos_theta_in,
                radiance_factor: self.material.diffuse
            })
        }
    }

    fn phong_sample(&self, rnd: (f32, f32)) -> Option<BrdfSample> {
        // get dir around refl. dir, move it to normals basis and then move it to world coords
        let in_dir_local_reflect = pow_cos_hemisphere_sample_w(self.material.phong_exp, rnd);
        let reflect_dir = self.out_dir_local.reflect_local();
        let reflect_basis = Frame::from_z(&reflect_dir);
        let in_dir_local = reflect_basis.to_world(&in_dir_local_reflect);
        let in_dir_world = self.own_basis.to_world(&in_dir_local);
        // if in_dir_world.z < EPS_COSINE {
        //     None
        // } else {
            Some(BrdfSample {
                in_dir_world: in_dir_world,
                cos_theta_in: in_dir_local.z,
                radiance_factor: self.material.specular
            })
        // }
    }
}

impl Material {
    pub fn new_identity() -> Material {
        Material {
            diffuse: Zero::zero(),
            specular: Zero::zero(),
            phong_exp: 0.0
        }
    }

    fn albedo_diffuse(&self) -> f32 {
        luminance(&self.diffuse)
    }

    fn albedo_specular(&self) -> f32 {
        luminance(&self.specular)
    }

    fn total_albedo(&self) -> f32 {
        (self.albedo_specular() + self.albedo_diffuse()).min(1.0)
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
                continuation: total_albedo
            }
        }
    }
}

fn luminance(a_rgb: &Vec3f) -> f32 {
    // a_rgb.x + a_rgb.y + a_rgb.z
    0.212671 * a_rgb.x + 0.715160 * a_rgb.y + 0.072169 * a_rgb.z
}

pub fn cos_hemisphere_sample_w(rnd: (f32, f32)) -> Vec3f { // -> (Vec3f, f32) {
    let phi = rnd.0 * 2.0 * PI;
    let costheta = rnd.1.sqrt();
    let sintheta = (1.0 - costheta * costheta).sqrt();

    let ret = Vec3f::new(sintheta * phi.cos(), sintheta * phi.sin(), costheta);
    ret
    // (ret, ret.z * FRAC_1_PI)
}

pub fn pow_cos_hemisphere_sample_w(n: f32, rnd: (f32, f32)) -> Vec3f {
    let phi = rnd.0 * 2.0 * PI;
    let cos_theta = rnd.1.powf(1.0 / (n + 1.0));
    let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
    Vec3f {
        x: phi.cos() * sin_theta, y: phi.sin() * sin_theta, z: cos_theta
    }
}
