#![allow(dead_code)]
use math::{Vec3f, Zero, EPS_COSINE};
use math::vector_traits::*;
use geometry::{Frame};
use brdf;
// use std::f32;

#[derive(Debug, Clone)]
pub struct Illumination {
    pub dir_to_light: Vec3f,
    pub dist_to_light: f32,
    pub dir_pdf_w: f32,
    pub intensity: Vec3f,
}

#[derive(Debug, Clone)]
pub struct Radiance {
    pub intensity: Vec3f,
    pub dir_pdf_a: f32,
}

pub trait Light {
    fn illuminate(&self, receiving_pnt: &Vec3f, rands: (f32, f32)) -> Option<Illumination>;
    fn get_radiance(&self, dir: &Vec3f, hit: &Vec3f) -> Option<Radiance>;
    fn is_delta(&self) -> bool;
}

#[derive(Debug, Clone)]
pub struct AreaLight {
    p0: Vec3f,
    e1: Vec3f,
    e2: Vec3f,
    frame: Frame,
    intensity: Vec3f,
    inv_area: f32,
}

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

impl AreaLight {
    pub fn new(p0: Vec3f, p1: Vec3f, p2: Vec3f, intensity: Vec3f) -> AreaLight {
        let e1 = p1 - p0;
        let e2 = p2 - p0;
        let normal = e1.cross(&e2);
        AreaLight {
            p0: p0,
            e1: e1,
            e2: e2,
            frame: Frame::from_z(normal.normalize()),
            inv_area: 2.0 / normal.norm(),
            intensity: intensity
        }
    }
}

impl Light for AreaLight {
    fn illuminate(&self, receiving_pnt: &Vec3f, rands: (f32, f32)) -> Option<Illumination> {
        let uv = brdf::uniform_triangle_sample(rands);
        let light_pnt = self.p0 + self.e1 * uv.x + self.e2 * uv.y;
        let dir_to_light = light_pnt - *receiving_pnt;
        let dist_sqr = dir_to_light.sqnorm();
        let dir_to_light = dir_to_light.normalize();
        let cos_normal_dir = self.frame.normal().dot(&-dir_to_light);
        if cos_normal_dir <= EPS_COSINE {
            None
        } else {
            Some(Illumination {
                dir_to_light: dir_to_light,
                dist_to_light: dist_sqr.sqrt(),
                dir_pdf_w: self.inv_area * dist_sqr / cos_normal_dir,
                intensity: self.intensity
            })
        }
    }

    fn get_radiance(&self, dir: &Vec3f, _hit: &Vec3f) -> Option<Radiance> {
        let cos_out_l = self.frame.normal().dot(&-dir.clone()).max(0.0);
        if cos_out_l < EPS_COSINE {
            None
        } else {
            Some(Radiance {
                intensity: self.intensity,
                dir_pdf_a: self.inv_area
            })
        }
    }

    fn is_delta(&self) -> bool {
        false
    }
}

impl Light for BackgroundLight {
    fn illuminate(&self, _receiving_pnt: &Vec3f, rands: (f32, f32)) -> Option<Illumination> {
        let (dir, dir_pdf_w) = brdf::uniform_sphere_sample_w(rands);
        Some(Illumination {
            dir_to_light: dir,
            dir_pdf_w: dir_pdf_w,
            dist_to_light: 1.0e35,
            intensity: self.intensity * self.scale
        })
    }

    fn get_radiance(&self, _dir: &Vec3f, _hit: &Vec3f) -> Option<Radiance> {
        let dir_pdf_w = brdf::uniform_sphere_pdf_w();
        Some(Radiance {
            intensity: self.intensity * self.scale,
            dir_pdf_a: dir_pdf_w, // it's ok only for background light
        })
    }

    fn is_delta(&self) -> bool {
        false
    }
}

impl Light for PointLight {
    fn illuminate(&self, receiving_pnt: &Vec3f, _rands: (f32, f32)) -> Option<Illumination> {
        let dir_to_light = self.position - *receiving_pnt;
        let dir_pdf_w  = dir_to_light.sqnorm();
        let dist = dir_to_light.norm();
        Some(Illumination {
            dir_to_light: dir_to_light / dist,
            dir_pdf_w: dir_pdf_w,
            dist_to_light: dist,
            intensity: self.intensity
        })
    }

    fn get_radiance(&self, _dir: &Vec3f, _hit: &Vec3f) -> Option<Radiance> {
        None
    }

    fn is_delta(&self) -> bool {
        true
    }
}
