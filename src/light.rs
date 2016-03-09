#![allow(dead_code)]
use math::{Vec3f, Zero, EPS_COSINE};
use math::vector_traits::*;
use geometry::{Frame, Geometry, Ray, Sphere};
use utility::*;
use brdf;
use std::f32::consts::FRAC_1_PI;
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
    pub dir_to_light: Vec3f,
    pub dist_to_light: f32,
    // pub dir_pdf_w: f32,
}

pub struct Radiation {
    pub radiance: Vec3f,
    // pub dir_pdf_w: f32,
}

pub struct LuminousObject<L: Luminous> {
    pub object: L,
    pub intensity: Vec3f,
}

pub trait Light {
    // out_ray - "out" in physical meaning, in trace from eye to light it's "incoming"
    fn radiate(&self, out_ray: &Ray) -> Option<Radiation>; //< for brdf sampling
    fn illuminate(&self, hit_pnt: &Vec3f, rnd: (f32, f32)) -> Option<Illumination>; //< for direct lighting
}

pub trait Luminous {
    fn emit_ray_orig(&self, hit_pnt: &Vec3f, rnd: (f32, f32)) -> Vec3f;
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
            dir_to_light: dir,
            dist_to_light: 1e35,
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
        let dist_to_light = dist_sq.sqrt();
        Some(Illumination {
            radiance: self.intensity / dist_sq * FRAC_1_PI, //< am i need for it (../(r^2*pi)) or not?
            dir_to_light: vec_to_light / dist_to_light,
            dist_to_light: dist_to_light
        })
    }
}

impl Luminous for Sphere {
    fn emit_ray_orig(&self, hit_pnt: &Vec3f, rnd: (f32, f32)) -> Vec3f {
        let local_dir = uniform_hemisphere_sample(rnd);
        let norm_to_disc = (*hit_pnt - self.center).normalize();
        let disc_basis = Frame::from_z(&norm_to_disc);
        let pnt_on_sphere = disc_basis.to_world(&local_dir) * self.radius + self.center;
        pnt_on_sphere
    }
}

impl<L> Light for LuminousObject<L> where L: Luminous {
    fn radiate(&self, _out_ray: &Ray) -> Option<Radiation> {
        Some(Radiation {
            radiance: self.intensity
        })
    }

    fn illuminate(&self, hit_pnt: &Vec3f, rnd: (f32, f32)) -> Option<Illumination> {
        let point_on_surface = self.object.emit_ray_orig(hit_pnt, rnd);
        let vec_to_light = point_on_surface - *hit_pnt;
        let dist_sq = vec_to_light.sqnorm();
        let dist_to_light = dist_sq.sqrt();
        Some(Illumination {
            radiance: self.intensity / dist_sq * (0.50 / FRAC_1_PI), //< am i need for it (../(r^2*pi)) or not?
            dir_to_light: vec_to_light / dist_to_light,
            dist_to_light: dist_to_light
        })
    }
}
