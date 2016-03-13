#![allow(dead_code)]
use math::{Vec3f, Zero, EPS_COSINE};
use math::vector_traits::*;
use geometry::{Frame, Geometry, Ray, Sphere};
use utility::*;
use brdf;
use std::f32::consts::{FRAC_1_PI, PI};
use std::rc::Rc;
// use std::f32;

#[derive(Debug, Clone)]
pub struct BackgroundLight {
    pub intensity: Vec3f,
    pub scale: f32
}

#[derive(Debug, Clone)]
pub struct PointLight {
    pub intensity: Vec3f,
    pub position: Vec3f,
}

pub struct Illumination {
    pub radiance: Vec3f,
    pub l_dir: Vec3f,
    pub l_dist: f32,
    // pub dir_pdf_w: f32,
}

pub struct Radiation {
    pub radiance: Vec3f,
    // pub dir_pdf_w: f32,
}

pub struct LuminousObject<L: Luminous + Geometry> {
    pub object: L,
    pub intensity: Vec3f,
}

pub trait Light {
    // out_ray - "out" in physical meaning, in trace from eye to light it's "incoming"
    fn radiate(&self, out_ray: &Ray) -> Option<Radiation>; //< for brdf sampling
    fn illuminate(&self, hit_pnt: &Vec3f, rnd: (f32, f32)) -> Option<Illumination>; //< for light sampling
}

pub trait Luminous {
    fn select_dir(&self, hit_pnt: &Vec3f, rnd: (f32, f32)) -> (Vec3f, f32); // dir from hit_pnt and weight
}

impl Light for BackgroundLight {
    fn radiate(&self, _out_ray: &Ray) -> Option<Radiation> {
        Some(Radiation {
            radiance: self.intensity * self.scale
        })
    }

    fn illuminate(&self, _hit_pnt: &Vec3f, rnd: (f32, f32)) -> Option<Illumination> {
        let dir = uniform_sphere_sample(rnd);
        Some(Illumination {
            radiance: self.intensity * self.scale,
            l_dir: dir,
            l_dist: 1e38
        })
    }
}

impl Light for PointLight {
    fn radiate(&self, _out_ray: &Ray) -> Option<Radiation> {
        Some(Radiation {
            radiance: self.intensity
        })
    }

    fn illuminate(&self, hit_pnt: &Vec3f, _rnd: (f32, f32)) -> Option<Illumination> {
        let vec_to_light = self.position - *hit_pnt;
        let dist_sq = vec_to_light.sqnorm();
        let l_dist = dist_sq.sqrt();
        Some(Illumination {
            radiance: self.intensity / dist_sq * FRAC_1_PI, //< am i need for it (../(r^2*pi)) or not?
            l_dir: vec_to_light / l_dist,
            l_dist: l_dist,
        })
    }
}

impl Luminous for Sphere {
    fn select_dir(&self, hit_pnt: &Vec3f, rnd: (f32, f32)) -> (Vec3f, f32) { // dir and weight
        let w = self.center - *hit_pnt;
        let w2 = w.sqnorm();
        let cos_alpha_max = (1.0 - (self.r2() / w2).min(1.0)).sqrt();
        let frac = 1.0 - cos_alpha_max;
        let omega = 2.0 * PI * frac;
        let w_basis = Frame::from_z(&w);
        let ld_local = uniform_cone_sample(cos_alpha_max, rnd);
        let ld = w_basis.to_world(&ld_local).normalize();
        assert!(omega >= 0.0);
        (ld, omega)
    }
}

impl<L> Light for LuminousObject<L> where L: Luminous + Geometry {
    fn radiate(&self, _out_ray: &Ray) -> Option<Radiation> {
        Some(Radiation {
            radiance: self.intensity
        })
    }

    fn illuminate(&self, hit_pnt: &Vec3f, rnd: (f32, f32)) -> Option<Illumination> {
        let (ld, omega) = self.object.select_dir(hit_pnt, rnd);
        if let Some(isect) = self.object.intersect(&Ray { orig: *hit_pnt, dir: ld }) {
            Some(Illumination {
                radiance: self.intensity * omega,
                l_dir: ld,
                l_dist: isect.dist
            })
        } else {
            None
        }
    }
}
