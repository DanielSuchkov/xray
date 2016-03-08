#![allow(dead_code)]
use math::{Vec3f, Zero, EPS_COSINE};
use math::vector_traits::*;
use geometry::{Frame, Geometry, Ray};
use utility::uniform_sphere_sample;
use brdf;
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

pub trait Light {
    // out_ray - "out" in physical meaning, in trace from eye to light it's "incoming"
    fn radiate(&self, out_ray: &Ray) -> Option<Radiation>;
    fn illuminate(&self, hit_pnt: &Vec3f, rnd: (f32, f32)) -> Option<Illumination>;
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
            radiance: self.intensity / dist_sq, // this divider have to be in pdf
            dir_to_light: vec_to_light / dist_to_light,
            dist_to_light: dist_to_light
        })
    }
}
