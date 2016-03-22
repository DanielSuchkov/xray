#![allow(dead_code)]
use math::vector_traits::*;
use math::{Vec3f, ortho, vec3_from_value, EPS_RAY};
use scene::SurfaceProperties;
use std::f32;
use std::rc::Rc;

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

pub struct DFSphere {
    pub center: Vec3f,
    pub radius: f32,
}

pub struct GeometryList {
    geometries: Vec<Box<GeometrySurface>>
}

pub trait Geometry {
    fn intersect(&self, ray: &Ray) -> Option<Intersection>;
    // fn build_aabbox(&self) -> AABBox;
}

pub trait GeometrySurface {
    fn intersect(&self, ray: &Ray) -> Option<SurfaceIntersection>;
    // fn build_aabbox(&self) -> AABBox;
}

pub trait GeometryManager {
    fn new() -> Self;
    fn nearest_intersection(&self, ray: &Ray) -> Option<SurfaceIntersection>;
    fn was_occluded(&self, ray: &Ray, dist: f32) -> bool;
    fn add_geometry<G>(&mut self, object: G) where G: GeometrySurface + 'static;
    // fn build_aabbox(&self) -> AABBox;
}

pub trait DistanceField {
    fn dist(&self, point: &Vec3f) -> f32;
    fn normal(&self, point: &Vec3f) -> f32;
}

impl<G> GeometrySurface for Surface<G> where G: Geometry {
    fn intersect(&self, ray: &Ray) -> Option<SurfaceIntersection> {
        self.geometry.intersect(ray).map(|isect| SurfaceIntersection {
            normal: isect.normal,
            dist: isect.dist,
            surface: self.properties,
        })
    }

    // fn build_aabbox(&self) -> AABBox {
    //     self.geometry.build_aabbox()
    // }
}

impl DFSphere {
    fn dist(&self, point: &Vec3f) -> f32 {
        (*point - self.center).norm() - self.radius
    }
}

impl Geometry for DFSphere {
    // this is just as proof-of-concept.
    // 1) it should be in separate trait for distance fields
    // 2) d have to be devided by gradient value (see http://www.iquilezles.org/www/articles/distance/distance.htm)
    // 3) cheat with normals: they are calculated analiticaly
    //    but has to be calculated numericaly with differentials (for common case).
    fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        let mut t = 0.0;
        let mut dprev = 1e38;
        for _ in 0..10000 {
            let new_point = ray.orig + ray.dir * t;
            let d = self.dist(&new_point);
            if d < 1e-6 {
                return Some(Intersection {
                    dist: t,
                    normal: (new_point - self.center).normalize()
                })
            }
            if d > dprev {
                return None;
            }
            dprev = d;
            t += d;
        }
        None
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
        let p = ray.orig - self.center;

        let r2 = self.r2();
        let p_d = p.dot(&ray.dir);

        // The sphere is behind or surrounding the start point.
        if p_d > 0.0 || p.dot(&p) < r2 {
            return None;
        }

        // Flatten p into the plane passing through c perpendicular to the ray.
        // This gives the closest approach of the ray to the center.
        let a = p - ray.dir * p_d;

        let a2 = a.dot(&a);

        // Closest approach is outside the sphere.
        if a2 > r2 {
            return None;
        }

        // Calculate distance from plane where ray enters/exits the sphere.
        let h = (r2 - a2).sqrt();

        // Calculate intersection point relative to sphere center.
        let i = a - ray.dir * h;

        let intersection = self.center + i;
        let normal = i.normalize();
        // We've taken a shortcut here to avoid a second square root.
        // Note numerical errors can make the normal have length slightly different from 1.
        // If you need higher precision, you may need to perform a conventional normalization.

        Some(Intersection {
            normal: normal,
            dist: (intersection - ray.orig).norm(),
        })
    }

    // fn build_aabbox(&self) -> AABBox {
    //     AABBox {
    //         min: self.center - vec3_from_value(self.radius),
    //         max: self.center + vec3_from_value(self.radius)
    //     }
    // }
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

    // fn build_aabbox(&self) -> AABBox {
    //     let (mut min, mut max) = (self.vert[0], self.vert[1]);
    //     for &v in self.vert.iter() {
    //         for i in 0..3 {
    //             min[i] = min[i].min(v[i]);
    //             max[i] = max[i].max(v[i]);
    //         }
    //     }
    //     AABBox { min: min, max: max }
    // }
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
                curr.map_or(isect, |ref cur|
                    isect.map_or(curr, |ref isec| if isec.dist < cur.dist { isect } else { curr })
                )
            )
    }

    fn was_occluded(&self, ray: &Ray, dist: f32) -> bool {
        self.geometries.iter()
            .map(|ref g| g.intersect(&ray))
            .any(|isect| isect.map_or(false, |isec| {
                isec.dist < dist
            }))
    }

    fn add_geometry<G>(&mut self, object: G) where G: GeometrySurface + 'static {
        self.geometries.push(Box::new(object));
    }

    // fn build_aabbox(&self) -> AABBox {
    //     let mut aabb = AABBox::new_infinity();
    //     for geo in self.geometries.iter() {
    //         aabb.grow_mut(&geo.build_aabbox());
    //     }
    //     aabb
    // }
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
