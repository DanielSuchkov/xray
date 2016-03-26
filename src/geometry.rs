#![allow(dead_code)]
use math::vector_traits::*;
use math::{Vec3f, ortho, vec3_from_value, EPS_RAY, smin_exp, smin_poly, smin_pow};
use scene::SurfaceProperties;
use std::f32;
use std::rc::Rc;

const EPS_DIST_FIELD: f32 = 5e-7;
const DELTA_GRAD: f32 = 1e-3;
const MAX_DFIELD_STEPS: usize = 512;

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

#[derive(Debug, Clone)]
pub struct DFieldIsosurface<D: DistanceField> {
    pub dfield: D,
    pub properties: SurfaceProperties
}

pub struct DFieldDisplace<D, F>
    where D: DistanceField,
          F: Fn(&Vec3f) -> f32 {
    pub a: D,
    pub disp: F
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

pub struct DFieldsSubstr<A, B>
    where A: DistanceField, B: DistanceField {
    pub a: A,
    pub b: B,
    pub pos: Vec3f,
}

pub struct DFieldsUnion<A, B>
    where A: DistanceField, B: DistanceField {
    pub a: A,
    pub b: B,
    pub pos: Vec3f,
}

pub struct DFieldsBlend<A, B>
    where A: DistanceField, B: DistanceField {
    pub a: A,
    pub b: B,
    pub k: f32,
    pub pos: Vec3f,
}

pub struct Torus {
    pub radius: f32,
    pub thickness: f32,
    pub center: Vec3f
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

pub trait DistanceField {
    fn dist(&self, point: &Vec3f) -> f32;

    fn grad(&self, p: &Vec3f, delta: f32) -> Vec3f {
        let p = *p;
        let dx = Vec3f::new(delta, 0.0, 0.0);
        let dy = Vec3f::new(0.0, delta, 0.0);
        let dz = Vec3f::new(0.0, 0.0, delta);
        let dfdx = self.dist(&(p + dx)) - self.dist(&(p - dx));
        let dfdy = self.dist(&(p + dy)) - self.dist(&(p - dy));
        let dfdz = self.dist(&(p + dz)) - self.dist(&(p - dz));
        Vec3f { x: dfdx, y: dfdy, z: dfdz } / (2.0 * delta)
    }
}

pub trait Isosurface {
    fn dist(&self, point: &Vec3f) -> f32;
    fn grad(&self, p: &Vec3f, delta: f32) -> Vec3f;
    fn surface_properties(&self) -> SurfaceProperties;
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

impl DistanceField for Sphere {
    fn dist(&self, point: &Vec3f) -> f32 {
        (*point - self.center).norm() - self.radius
    }
}

impl DistanceField for Torus {
    fn dist(&self, point: &Vec3f) -> f32 {
        let point = *point - self.center;
        let q = Vec2::new(Vec2::new(point.x, point.y).norm() - self.radius, point.z);
        q.norm() - self.thickness
    }
}

impl DistanceField for RoundBox {
    fn dist(&self, point: &Vec3f) -> f32 {
        let p = *point - self.pos;
        let abs_pb = Vec3f{
            x: (p.x.abs() - self.dim.x).max(0.0),
            y: (p.y.abs() - self.dim.y).max(0.0),
            z: (p.z.abs() - self.dim.z).max(0.0)
        };
        abs_pb.norm() - self.r
    }
}

impl<A, B> DistanceField for DFieldsSubstr<A, B>
    where A: DistanceField, B: DistanceField {
    fn dist(&self, point: &Vec3f) -> f32 {
        let point = *point - self.pos;
        self.a.dist(&point).max(-self.b.dist(&point))
    }
}

impl<A, B> DistanceField for DFieldsUnion<A, B>
    where A: DistanceField, B: DistanceField {
    fn dist(&self, point: &Vec3f) -> f32 {
        let point = *point - self.pos;
        self.a.dist(&point).min(self.b.dist(&point))
    }
}

impl<A, B> DistanceField for DFieldsBlend<A, B>
    where A: DistanceField, B: DistanceField {
    fn dist(&self, point: &Vec3f) -> f32 {
        let point = *point - self.pos;
        smin_poly(self.a.dist(&point), self.b.dist(&point), self.k)
    }
}

impl<D, F> DistanceField for DFieldDisplace<D, F>
    where D: DistanceField,
          F: Fn(&Vec3f) -> f32 {
    fn dist(&self, point: &Vec3f) -> f32 {
        let d1 = self.a.dist(point);
        let d2 = (self.disp)(point);
        d1 + d2
    }
}

impl<D> Isosurface for DFieldIsosurface<D> where D: DistanceField {
    fn dist(&self, point: &Vec3f) -> f32 {
        self.dfield.dist(point)
    }

    fn grad(&self, p: &Vec3f, delta: f32) -> Vec3f {
        self.dfield.grad(p, delta)
    }

    fn surface_properties(&self) -> SurfaceProperties {
        self.properties
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
                d = d.min(dist);
                if dist < EPS_DIST_FIELD {
                    let grad = df.grad(&new_point, DELTA_GRAD);
                    return Some(SurfaceIntersection {
                        normal: grad.normalize(),
                        dist: t + d,
                        surface: df.surface_properties()
                    })
                }
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
        let isect = self.nearest_geo_isect(ray);
        self.nearest_isosuface_isect(ray, isect.map_or(10000.0, |isec| isec.dist)).or(isect)
    }

    fn was_occluded(&self, ray: &Ray, dist: f32) -> bool {
        let occluded_by_geo = self.geometries.iter()
            .map(|ref g| g.intersect(&ray))
            .any(|isect| isect.map_or(false, |isec| {
                isec.dist < dist
            }));

        if occluded_by_geo {
            true
        } else {
            self.nearest_isosuface_isect(ray, dist).is_some()
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

mod tests {
    use super::*;
    use math::{Vec3f, EPS_RAY};
    use scene::SurfaceProperties;

    #[test]
    fn occlusion_sphere() {
        let mut geos = GeometryList::new();
        let sphere = Sphere { center: Vec3f::new(0.0, 0.0, 0.0), radius: 2.0 };
        geos.add_geometry(Surface { geometry: sphere, properties: SurfaceProperties::Material(0) });
        let ray = Ray { orig: Vec3f::new(0.0, 0.0, -5.0), dir: Vec3f::new(0.0, 0.0, 1.0) };
        assert!(geos.was_occluded(&ray, 4.0));
        assert!(!geos.was_occluded(&ray, 3.0 - EPS_RAY));
        assert!(geos.was_occluded(&ray, 3.0 + EPS_RAY));
    }

    #[test]
    fn occlusion_sphere_on_surface() {
        let mut geos = GeometryList::new();
        let sphere = Sphere { center: Vec3f::new(0.0, 0.0, 0.0), radius: 2.0 };
        geos.add_geometry(Surface { geometry: sphere, properties: SurfaceProperties::Material(0) });
        let ray = Ray { orig: Vec3f::new(0.0, 0.0, -2.0), dir: Vec3f::new(0.0, 0.0, 1.0) };
        assert!(geos.was_occluded(&ray, EPS_RAY));
    }

    #[test]
    fn occlusion_sphere_near_surface() {
        let mut geos = GeometryList::new();
        let sphere = Sphere { center: Vec3f::new(0.0, 0.0, 0.0), radius: 2.0 };
        geos.add_geometry(Surface { geometry: sphere, properties: SurfaceProperties::Material(0) });
        let ray = Ray { orig: Vec3f::new(0.0, 0.0, -2.0 - EPS_RAY * 2.0), dir: Vec3f::new(0.0, 0.0, 1.0) };
        assert!(!geos.was_occluded(&ray, EPS_RAY));
    }

    #[test]
    fn occlusion_tri_sphere() {
        let mut geos = GeometryList::new();
        let ident_mat = SurfaceProperties::Material(0);
        let sphere = Sphere { center: Vec3f::new(0.0, 0.0, 0.0), radius: 2.0 };
        geos.add_geometry(Surface { geometry: sphere, properties: ident_mat });

        let tri = Triangle::new(Vec3f::new(1.0, -1.0, -3.0) , Vec3f::new(-1.0, -1.0, -3.0), Vec3f::new(-1.0, 1.0, -3.0));
        geos.add_geometry(Surface { geometry: tri, properties: ident_mat });

        let ray = Ray { orig: Vec3f::new(0.0, 0.0, 2.1), dir: Vec3f::new(0.0, 0.0, 1.0) };
        assert!(!geos.was_occluded(&ray, 0.01));

        let ray_from_tri = Ray { orig: Vec3f::new(0.0, 0.0, -3.5), dir: Vec3f::new(0.0, 0.0, 1.0) };
        assert!(geos.was_occluded(&ray_from_tri, 2.0));
    }

     #[test]
    fn occlusion_tri() {
        let mut geos = GeometryList::new();
        let ident_mat = SurfaceProperties::Material(0);

        let tri = Triangle::new(Vec3f::new(1.0, -1.0, -3.0) , Vec3f::new(-1.0, -1.0, -3.0), Vec3f::new(-1.0, 1.0, -3.0));
        geos.add_geometry(Surface { geometry: tri, properties: ident_mat });

        let ray = Ray { orig: Vec3f::new(0.0, 0.0, 2.1), dir: Vec3f::new(0.0, 0.0, 1.0) };
        let was_occluded = if let Some(isect) = geos.nearest_intersection(&ray) {
            println!("{:?}", isect.dist);
            if isect.dist < 1.0 {
                true
            } else {
                false
            }
        } else {
            false
        };

        assert!(!was_occluded);

        let ray_from_tri = Ray { orig: Vec3f::new(0.0, 0.0, -3.5), dir: Vec3f::new(0.0, 0.0, 1.0) };
        assert!(geos.was_occluded(&ray_from_tri, 2.0));
    }
}
