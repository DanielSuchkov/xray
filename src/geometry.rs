#![allow(dead_code)]

use math::vector_traits::*;
use math::{Vec3f, vec3_from_value, ortho};
use scene::SurfaceProperties;
use std::f32;

#[derive(Debug, Clone, Copy)]
pub struct SurfaceIntersection {
    pub normal: Vec3f, // normal at intersection point
    pub dist: f32, // distance to nearest intersection point
    pub surface: SurfaceProperties,
}

pub struct Intersection {
    pub normal: Vec3f, // normal at intersection point
    pub dist: f32, // distance to nearest intersection point
}

pub struct Surface<G: Geometry + 'static> {
    pub geometry: G,
    pub properties: SurfaceProperties,
}

#[derive(Debug, Clone, Copy)]
pub struct AABBox {
    pub min: Vec3f,
    pub max: Vec3f,
}

#[derive(Debug, Clone, Copy)]
pub struct BSphere {
    pub center: Vec3f,
    pub radius: f32,
    pub inv_radius_sqr: f32, // 1.0/(r^2)
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
}

#[derive(Debug, Clone)]
pub struct Triangle {
    pub vert: [Vec3f; 3],
    pub normal: Vec3f,
}

#[derive(Debug, Clone)]
pub struct Frame {
    ox: Vec3f,
    oy: Vec3f,
    oz: Vec3f,
}

pub struct GeometryList {
    geometries: Vec<Box<GeometrySurface>>
}

pub trait Geometry {
    fn intersect(&self, ray: &Ray) -> Option<Intersection>;
    fn build_aabbox(&self) -> AABBox;
}

pub trait GeometrySurface {
    fn intersect(&self, ray: &Ray) -> Option<SurfaceIntersection>;
    fn build_aabbox(&self) -> AABBox;
}

pub trait GeometryManager {
    fn new() -> Self;
    fn nearest_intersection(&self, ray: &Ray) -> Option<SurfaceIntersection>;
    fn add_geometry<GS>(&mut self, object: GS) where GS: GeometrySurface + 'static;
    fn build_aabbox(&self) -> AABBox;
}

impl<G> GeometrySurface for Surface<G> where G: Geometry {
    fn intersect(&self, ray: &Ray) -> Option<SurfaceIntersection> {
        self.geometry.intersect(ray).map(|isect| SurfaceIntersection {
            normal: isect.normal,
            dist: isect.dist,
            surface: self.properties,
        })
    }

    fn build_aabbox(&self) -> AABBox {
        self.geometry.build_aabbox()
    }
}

impl Sphere {
    pub fn r2(&self) -> f32 {
        self.radius * self.radius
    }
}

impl AABBox {
    fn new_infinity() -> AABBox {
        AABBox {
            min: vec3_from_value(f32::INFINITY),
            max: vec3_from_value(f32::NEG_INFINITY),
        }
    }

    fn grow_mut(&mut self, other: &AABBox) {
        self.min.x = self.min.x.min(other.min.x);
        self.min.y = self.min.y.min(other.min.y);
        self.min.z = self.min.z.min(other.min.z);
        self.max.x = self.max.x.max(other.max.x);
        self.max.y = self.max.y.max(other.max.y);
        self.max.z = self.max.z.max(other.max.z);
    }

    fn grow(&self, other: &AABBox) -> AABBox {
        let mut aabb = other.clone();
        aabb.grow_mut(self);
        aabb
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

    fn nearest_intersection(&self, ray: &Ray) -> Option<SurfaceIntersection> {
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

    fn add_geometry<GS>(&mut self, object: GS) where GS: GeometrySurface + 'static {
        self.geometries.push(Box::new(object));
    }

    fn build_aabbox(&self) -> AABBox {
        let mut aabb = AABBox::new_infinity();
        for geo in self.geometries.iter() {
            aabb.grow_mut(&geo.build_aabbox());
        }
        aabb
    }
}

impl Frame {
    fn new(ox: Vec3f, oy: Vec3f, oz: Vec3f) -> Frame {
        Frame { ox: ox, oy: oy, oz: oz }
    }

    fn new_identity() -> Frame {
        Frame {
            ox: Vec3f::new(1.0, 0.0, 0.0),
            oy: Vec3f::new(0.0, 1.0, 0.0),
            oz: Vec3f::new(0.0, 0.0, 1.0),
        }
    }

    fn new_from_z(oz: Vec3f) -> Frame {
        let oz = oz.normalize();
        let temp_ox = ortho(&oz);
        let oy = oz.cross(&temp_ox).normalize();
        let ox = oy.cross(&oz);
        Frame { ox: ox, oy: oy, oz: oz }
    }

    fn normal(&self) -> Vec3f {
        self.oz
    }

    fn tangent(&self) -> Vec3f {
        self.oy
    }

    fn binormal(&self) -> Vec3f {
        self.ox
    }

    fn to_world(&self, v: &Vec3f) -> Vec3f {
        self.ox * v.x + self.oy * v.y + self.oz * v.z
    }

    fn to_local(&self, v: &Vec3f) -> Vec3f {
        Vec3f {
            x: v.dot(&self.ox),
            y: v.dot(&self.oy),
            z: v.dot(&self.oz),
        }
    }
}
