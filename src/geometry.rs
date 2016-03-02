#![allow(dead_code)]

use math::vector_traits::*;
use math::{Vec3f, vec3_from_value};
use scene::{MaterialID, LightID};
// use std::{f32, f64};

#[derive(Debug, Clone, Copy)]
pub enum SurfaceProperties {
    Material(MaterialID),
    Light(LightID)
}

#[derive(Debug, Clone, Copy)]
pub struct Intersection {
    pub normal: Vec3f, // normal at intersection point
    pub dist: f32, // distance to nearest intersection point
    pub surface: SurfaceProperties,
}

#[derive(Debug, Clone, Copy)]
pub struct AABBox {
    min: Vec3f,
    max: Vec3f,
}

#[derive(Debug, Clone, Copy)]
pub struct Ray {
    pub orig: Vec3f, // origin
    pub dir: Vec3f, // direction
}

#[derive(Debug, Clone)]
pub struct Sphere {
    pub center: Vec3f,
    pub radius: f32,
    pub surface: SurfaceProperties,
}

#[derive(Debug, Clone)]
pub struct Triangle {
    pub vert: [Vec3f; 3],
    pub normal: Vec3f,
    pub surface: SurfaceProperties,
}

pub struct GeometryList {
    geometries: Vec<Box<Geometry>>
}

pub trait Geometry {
    fn intersect(&self, ray: &Ray) -> Option<Intersection>;
    fn build_aabbox(&self) -> AABBox;
}

pub trait GeometryManager {
    fn new() -> Self;
    fn nearest_intersection(&self, ray: &Ray) -> Option<Intersection>;
    fn add_geometry<T>(&mut self, object: T) where T: Geometry + 'static;
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

        let a = ray.dir.dot(&ray.dir) as f64;
        let b = 2.0 * ray.dir.dot(&local_origin) as f64;
        let c = (local_origin.dot(&local_origin) - self.r2()) as f64;

        // Must use f64, because when b ~ sqrt(b*b - 4*a*c)
        // the resulting t is imprecise enough to get around ray epsilons
        let disc: f64 = b * b - 4.0 * a * c;

        if disc < 0.0 {
            return None;
        }

        let disc_sqrt = disc.sqrt();
        let q = if b < 0.0 { (-b - disc_sqrt) / 2.0 } else { (-b + disc_sqrt) / 2.0 };

        let (t0, t1) = {
            let (t0, t1) = (q / a, c / q);
            if t0 > t1 { (t1, t0) } else { (t0, t1) }
        };

        let res_t = if t0 > 0.0 {
            t0 as f32
        } else if t1 > 0.0 {
            t1 as f32
        } else {
            return None;
        };

        Some(Intersection {
            normal: (local_origin + vec3_from_value(res_t) * ray.dir).normalize(),
            dist: res_t,
            surface: self.surface
        })
    }

    fn build_aabbox(&self) -> AABBox {
        AABBox {
            min: self.center - vec3_from_value(self.radius),
            max: self.center + vec3_from_value(self.radius)
        }
    }
}

impl Geometry for Triangle {
    fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        let ao = self.vert[0] - ray.orig;
        let bo = self.vert[1] - ray.orig;
        let co = self.vert[2] - ray.orig;

        let v0 = co.cross(&bo);
        let v1 = bo.cross(&ao);
        let v2 = ao.cross(&co);

        let v0d = v0.dot(&ray.dir);
        let v1d = v1.dot(&ray.dir);
        let v2d = v2.dot(&ray.dir);

        if ((v0d < 0.0)  && (v1d < 0.0)  && (v2d < 0.0)) ||
           ((v0d >= 0.0) && (v1d >= 0.0) && (v2d >= 0.0)) {
            Some(Intersection {
                normal: self.normal,
                dist: self.normal.dot(&ao) / self.normal.dot(&ray.dir),
                surface: self.surface,
            })
        } else {
            None
        }
    }

    fn build_aabbox(&self) -> AABBox {
        let (mut min, mut max) = (self.vert[0], self.vert[1]);
        for &v in self.vert.iter() {
            for i in 0..3 {
                min[i] = min[i].min(v[i]);
                max[i] = max[i].max(v[i]);
            }
        }

        AABBox { min: min, max: max }
    }
}

impl GeometryManager for GeometryList {
    fn new() -> GeometryList {
        GeometryList {
            geometries: Vec::new()
        }
    }

    fn nearest_intersection(&self, ray: &Ray) -> Option<Intersection> {
        self.geometries.iter()
            .map(|ref g| g.intersect(&ray))
            .fold(None, |curr, isect|
                if let Some(cur) = curr {
                    isect.map(|isec| if isec.dist < cur.dist { isec } else { cur })
                } else {
                    isect
                }
            )
    }

    fn add_geometry<T>(&mut self, object: T)
        where T: Geometry + 'static {
        self.geometries.push(Box::new(object));
    }
}
