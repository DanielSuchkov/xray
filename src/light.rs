#![allow(dead_code)]
use math::Vec3f;
use math::vector_traits::*;
use geometry::{Frame, Geometry, Ray, Sphere};
use utility::*;
use std::f32::consts::{FRAC_1_PI, PI};
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub struct BackgroundLight {
    pub intensity: Vec3f,
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
    pub pdf: f32,
}

pub struct Radiation {
    pub radiance: Vec3f,
    pub pdf: f32,
}

#[derive(Debug)]
pub struct LuminousObject<L: Luminous + Geometry + Debug> {
    pub object: L,
    pub intensity: Vec3f,
}

pub trait Light : Debug {
    // out_ray - "out" in physical meaning, in trace from eye to light it's "incoming"
    fn radiate(&self, out_ray: &Ray) -> Option<Radiation>; //< for brdf sampling
    fn illuminate(&self, hit_pnt: &Vec3f, rnd: (f32, f32)) -> Option<Illumination>; //< for light sampling
}

pub trait Luminous {
    // dir from hit_pnt, weight and pdf
    fn select_dir(&self, hit_pnt: &Vec3f, rnd: (f32, f32)) -> (Vec3f, f32, f32);
    fn dir_pdf(&self, ray: &Ray) -> f32;
}

impl Light for BackgroundLight {
    fn radiate(&self, _out_ray: &Ray) -> Option<Radiation> {
        Some(Radiation {
            radiance: self.intensity,
            pdf: 0.25 * FRAC_1_PI,
        })
    }

    fn illuminate(&self, _hit_pnt: &Vec3f, rnd: (f32, f32)) -> Option<Illumination> {
        let (dir, pdf) = uniform_hemisphere_sample_w(rnd);
        Some(Illumination {
            radiance: self.intensity,
            l_dir: -dir,
            l_dist: 1e38,
            pdf: pdf
        })
    }
}

impl Light for PointLight {
    fn radiate(&self, _out_ray: &Ray) -> Option<Radiation> {
        panic!("Wat?!");
    }

    fn illuminate(&self, hit_pnt: &Vec3f, _rnd: (f32, f32)) -> Option<Illumination> {
        let vec_to_light = self.position - *hit_pnt;
        let dist_sq = vec_to_light.sqnorm();
        let l_dist = dist_sq.sqrt();
        Some(Illumination {
            radiance: self.intensity / dist_sq * FRAC_1_PI, //< am i need for it (../(r^2*pi)) or not?
            l_dir: vec_to_light / l_dist,
            l_dist: l_dist,
            pdf: 1.0,
        })
    }
}

impl Luminous for Sphere {
    fn select_dir(&self, hit_pnt: &Vec3f, rnd: (f32, f32)) -> (Vec3f, f32, f32) { // dir, weight and pdf
        let w = self.center - *hit_pnt;
        let w2 = w.sqnorm();
        let cos_theta_max = (1.0 - (self.r2() / w2).min(1.0)).sqrt();
        let frac = 1.0 - cos_theta_max;
        let omega = 2.0 * PI * frac;
        let w_basis = Frame::from_z(&w);
        let ld_local = uniform_cone_sample(cos_theta_max, rnd);
        let ld = w_basis.to_world(&ld_local).normalize();
        let pdf = FRAC_1_PI * 0.5 / (1.0 - cos_theta_max);
        (ld, omega, pdf)
    }

    fn dir_pdf(&self, ray: &Ray) -> f32 {
        let w = self.center - ray.orig;
        let w2 = w.sqnorm();
        // let cos_theta = ray.dir.dot(&w).abs();
        let cos_theta_max = (1.0 - (self.r2() / w2).min(1.0)).sqrt();
        // let sin_theta_max2 = (self.r2() / w2).min(1.0).max(0.0);
        (FRAC_1_PI * 0.5 / (1.0 - cos_theta_max)).max(0.0)
        // cos_theta * FRAC_1_PI / sin_theta_max2
    }
}

impl<L> Light for LuminousObject<L> where L: Luminous + Geometry + Debug {
    fn radiate(&self, out_ray: &Ray) -> Option<Radiation> {
        Some(Radiation {
            radiance: self.intensity,
            pdf: self.object.dir_pdf(out_ray),
        })
    }

    fn illuminate(&self, hit_pnt: &Vec3f, rnd: (f32, f32)) -> Option<Illumination> {
        let (ld, omega, pdf) = self.object.select_dir(hit_pnt, rnd);
        if let Some(isect) = self.object.intersect(&Ray { orig: *hit_pnt, dir: ld }) {
            Some(Illumination {
                radiance: self.intensity * omega,
                l_dir: ld,
                l_dist: isect.dist,
                pdf: pdf
            })
        } else {
            None
        }
    }
}
