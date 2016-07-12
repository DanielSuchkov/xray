#![allow(dead_code)]
use math::vector_traits::*;
use math::{Vec2f, Vec3f, ortho};
use scene::SurfaceProperties;
use std::f32;

pub mod distance_fields;
pub use self::distance_fields::*;

#[cfg(test)]
mod tests;

pub const EPS_DIST_FIELD: f32 = 1e-4;
pub const EPS_RAY_GEO: f32 = 1e-4;
pub const EPS_RAY_DF: f32 = 1e-2;
pub const DELTA_GRAD: f32 = 1e-4;
pub const MAX_DFIELD_STEPS: usize = 1024;

#[derive(Debug, Clone, Copy)]
pub struct SurfaceIntersection {
    pub normal: Vec3f, // normal at intersection point
    pub dist: f32, // distance to nearest intersection point
    pub surface: SurfaceProperties,
}

#[derive(Debug, Clone, Copy)]
pub struct Intersection {
    pub normal: Vec3f, // normal at intersection point
    pub dist: f32, // distance to nearest intersection point
}

#[derive(Debug, Clone)]
pub struct Surface<G: Geometry> {
    pub geometry: G,
    pub properties: SurfaceProperties,
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
    geometries: Vec<Box<GeometrySurface>>,
    dfields: Vec<Box<Isosurface>>
}

pub struct Torus {
    pub radius: f32,
    pub thickness: f32,
    pub center: Vec3f
}

///@TODO: impl DField for it
pub struct Cone {
    pub c: Vec2f,
    pub pos: Vec3f,
}

pub struct RoundBox {
    pub dim: Vec3f,
    pub pos: Vec3f,
    pub r: f32,
}


pub trait Geometry {
    fn intersect(&self, ray: &Ray) -> Option<Intersection>;
}

pub trait GeometrySurface {
    fn intersect(&self, ray: &Ray) -> Option<SurfaceIntersection>;
}

pub trait GeometryManager {
    fn new() -> Self;
    fn nearest_intersection(&self, ray: &Ray) -> Option<SurfaceIntersection>;
    fn was_occluded(&self, ray: &Ray, dist: f32) -> bool;
    fn add_geometry<G>(&mut self, object: G) where G: GeometrySurface + 'static;
    fn add_isosurface<I>(&mut self, object: I) where I: Isosurface + 'static;
}


impl<G> GeometrySurface for Surface<G> where G: Geometry {
    fn intersect(&self, ray: &Ray) -> Option<SurfaceIntersection> {
        self.geometry.intersect(ray).map(|isect| SurfaceIntersection {
            normal: isect.normal,
            dist: isect.dist,
            surface: self.properties,
        })
    }
}

impl Ray {
    pub fn advance(&self, delta: f32) -> Ray {
        Ray { dir: self.dir, orig: self.orig + self.dir * delta }
    }
}

impl DField for Sphere {
    fn dist(&self, point: &Vec3f) -> f32 {
        (*point - self.center).norm() - self.radius
    }
}

impl DField for Torus {
    fn dist(&self, point: &Vec3f) -> f32 {
        let point = *point - self.center;
        let q = Vec2::new(Vec2::new(point.x, point.y).norm() - self.radius, point.z);
        q.norm() - self.thickness
    }
}

impl DField for RoundBox {
    fn dist(&self, point: &Vec3f) -> f32 {
        let p = *point - self.pos;
        let abs_pb = p.zip(&self.dim, |x, y| (x.abs() - y).max(0.0));
        abs_pb.norm() - self.r
    }
}

impl Sphere {
    pub fn r2(&self) -> f32 {
        self.radius * self.radius
    }
}

impl Geometry for Sphere {
    fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        let p = ray.orig - self.center;

        let r2 = self.r2();
        let p_d = p.dot(&ray.dir);

        if p_d > 0.0 || p.dot(&p) < r2 {
            return None;
        }

        let a = p - ray.dir * p_d;
        let a2 = a.dot(&a);

        if a2 > r2 {
            return None;
        }

        let h = (r2 - a2).sqrt();
        let i = a - ray.dir * h;

        let intersection = self.center + i;
        let normal = i.normalize();

        Some(Intersection {
            normal: normal,
            dist: (intersection - ray.orig).norm(),
        })
    }
}

impl Triangle {
    pub fn new(p0: Vec3f, p1: Vec3f, p2: Vec3f) -> Triangle {
        Triangle {
            vert: [p0, p1, p2],
            normal: (p1 - p0).cross(&(p2 - p0)).normalize()
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

        if ((v0d <= 0.0)  && (v1d <= 0.0)  && (v2d <= 0.0)) ||
           ((v0d >= 0.0) && (v1d >= 0.0) && (v2d >= 0.0)) {
            let dist = self.normal.dot(&ao) / self.normal.dot(&ray.dir);
            if dist <= 0.0 {
                None
            } else {
                Some(Intersection {
                    normal: self.normal,
                    dist: dist,
                })
            }
        } else {
            None
        }
    }
}

impl GeometryList {
    fn nearest_geo_isect(&self, ray: &Ray) -> Option<SurfaceIntersection> {
        self.geometries.iter()
            .map(|ref g| g.intersect(&ray))
            .fold(None, |curr, isect|
                curr.map_or(isect, |ref cur|
                    isect.map_or(curr, |ref isec| if isec.dist < cur.dist { isect } else { curr })
                )
            )
    }

    fn nearest_isosuface_isect(&self, ray: &Ray, max_dist: f32) -> Option<SurfaceIntersection> {
        if self.dfields.is_empty() {
            return None;
        }

        let mut t = 0.0;
        for _ in 0..MAX_DFIELD_STEPS {
            let new_point = ray.orig + ray.dir * t;

            let mut d = max_dist;
            for ref df in self.dfields.iter() {
                // let grad = df.grad(&new_point, DELTA_GRAD);
                let dist = df.dist(&new_point)/* / grad.norm()*/;
                if dist < EPS_DIST_FIELD {
                    let new_point = ray.orig + ray.dir * (t + dist);
                    let grad = df.grad(&new_point, DELTA_GRAD);
                    return Some(SurfaceIntersection {
                        normal: grad.normalize(),
                        dist: t + dist,
                        surface: df.surface_properties()
                    })
                }
                d = d.min(dist);
            }

            t += d;
            if t > max_dist {
                return None;
            }
        }
        None
    }
}

impl GeometryManager for GeometryList {
    fn new() -> GeometryList {
        GeometryList {
            geometries: Vec::new(),
            dfields: Vec::new()
        }
    }

    fn nearest_intersection(&self, ray: &Ray) -> Option<SurfaceIntersection> {
        let ray_geo = ray.advance(EPS_RAY_GEO);;
        let isect = self.nearest_geo_isect(&ray_geo);
        let ray_df = ray.advance(EPS_RAY_DF);
        self.nearest_isosuface_isect(&ray_df, isect.map_or(10000.0, |isec| isec.dist)).or(isect)
    }

    fn was_occluded(&self, ray: &Ray, dist: f32) -> bool {
        let ray_geo = ray.advance(EPS_RAY_GEO);
        let dist_geo = dist - 2.0 * EPS_RAY_GEO;
        let occluded_by_geo = self.geometries.iter()
            .map(|ref g| g.intersect(&ray_geo))
            .any(|isect| isect.map_or(false, |isec| {
                isec.dist < dist_geo
            }));

        if occluded_by_geo {
            true
        } else {
            let ray_df = ray.advance(EPS_RAY_DF);
            let dist_df = dist - 2.0 * EPS_RAY_DF;
            self.nearest_isosuface_isect(&ray_df, dist_df).is_some()
        }
    }

    fn add_geometry<G>(&mut self, object: G) where G: GeometrySurface + 'static {
        self.geometries.push(Box::new(object));
    }

    fn add_isosurface<I>(&mut self, object: I) where I: Isosurface + 'static {
        self.dfields.push(Box::new(object));
    }
}

impl Frame {
    pub fn new(ox: Vec3f, oy: Vec3f, oz: Vec3f) -> Frame {
        Frame { ox: ox, oy: oy, oz: oz }
    }

    pub fn new_identity() -> Frame {
        Frame {
            ox: Vec3f::new(1.0, 0.0, 0.0),
            oy: Vec3f::new(0.0, 1.0, 0.0),
            oz: Vec3f::new(0.0, 0.0, 1.0),
        }
    }

    pub fn from_z(oz: &Vec3f) -> Frame {
        let oz = oz.normalize();
        let temp_ox = ortho(&oz);
        let oy = oz.cross(&temp_ox).normalize();
        let ox = oy.cross(&oz);
        Frame { ox: ox, oy: oy, oz: oz }
    }

    pub fn normal(&self) -> Vec3f {
        self.oz
    }

    pub fn tangent(&self) -> Vec3f {
        self.oy
    }

    pub fn binormal(&self) -> Vec3f {
        self.ox
    }

    pub fn to_world(&self, v: &Vec3f) -> Vec3f {
        self.ox * v.x + self.oy * v.y + self.oz * v.z
    }

    pub fn to_local(&self, v: &Vec3f) -> Vec3f {
        Vec3f {
            x: v.dot(&self.ox),
            y: v.dot(&self.oy),
            z: v.dot(&self.oz),
        }
    }
}
