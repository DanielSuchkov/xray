#![allow(unused_imports)]
#![allow(dead_code)]

use math::{FloatExtra, Vec3f, EuclideanVector, Vector};
use scene::{MaterialID, LightID};
use std::{f32, f64};

#[derive(Debug, Clone)]
pub enum SurfaceProperties {
    Material(MaterialID),
    Light(LightID)
}

#[derive(Debug, Clone)]
pub struct Intersection {
    pub normal: Vec3f, // normal at intersection point
    pub dist: f32, // distance to nearest intersection point
    pub surface: SurfaceProperties,
}

#[derive(Debug, Clone)]
pub struct AABBox {
    min: Vec3f,
    max: Vec3f,
}

#[derive(Debug, Clone)]
pub struct Ray {
    orig: Vec3f, // origin
    dir: Vec3f, // direction
}

#[derive(Debug, Clone)]
pub struct Sphere {
    center: Vec3f,
    radius: f32,
    material: MaterialID,
}

pub trait Geometry {
    fn intersect(&self, ray: &Ray) -> Option<Intersection>;
    fn build_aabbox(&self) -> AABBox {
        unimplemented!();
    }
}

impl Sphere {
    pub fn r2(&self) -> f32 {
        self.radius * self.radius
    }
}

impl Geometry for Sphere {
    fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        // we transform ray origin into object space (center == origin)
        let local_origin = ray.orig - self.center;

        let a = ray.dir.dot(ray.dir) as f64;
        let b = 2.0 * ray.dir.dot(local_origin) as f64;
        let c = (local_origin.dot(local_origin) - self.r2()) as f64;

        // Must use doubles, because when B ~ sqrt(B*B - 4*a*c)
        // the resulting t is imprecise enough to get around ray epsilons
        let disc: f64 = b * b - 4.0 * a * c;

        if disc < 0.0 {
            return None;
        }

        let disc_sqrt = disc.sqrt();
        let q = if b < 0.0 { (-b - disc_sqrt) / 2.0 } else { (-b + disc_sqrt) / 2.0 };

        let (t0, t1) = {
            let t0 = q / a;
            let t1 = c / q;
            if t0 > t1 { (t1, t0) } else { (t0, t1) }
        };

        let result_t = if t0 > 0.0 {
            t0 as f32
        } else if t1 > 0.0 {
            t1 as f32
        } else {
            return None;
        };

        Some(Intersection {
            normal: (local_origin + Vec3f::from_value(result_t) * ray.dir).normalize(),
            dist: result_t,
            surface: SurfaceProperties::Material(self.material)
        })
    }
}
