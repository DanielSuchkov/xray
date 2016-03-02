#![allow(dead_code)]

use geometry::{GeometryManager, Ray, Intersection, Geometry};
use brdf::Material;

pub type MaterialID = i32;
pub type LightID = i32;

#[derive(Debug, Clone, Copy)]
pub enum SurfaceProperties {
    Material(MaterialID),
    Light(LightID)
}

#[derive(Debug, Clone)]
pub struct DefaultScene<T> where T: GeometryManager {
    geo: T,
    materials: Vec<MaterialID>,
    lights: Vec<LightID>
}

pub trait Scene {
    fn new() -> Self;
    fn nearest_intersection(&self, ray: &Ray) -> Option<Intersection>;
    fn add_object<O: Geometry>(&mut self, obj: O);
}

impl<T> Scene for DefaultScene<T> where T: GeometryManager {
    fn new() -> DefaultScene<T> {
        DefaultScene {
            geo: T::new(),
            materials: Vec::new(),
            lights: Vec::new()
        }
    }

    fn nearest_intersection(&self, ray: &Ray) -> Option<Intersection> {
        self.geo.nearest_intersection(ray)
    }

    fn add_object<O>(&mut self, obj: O) where O: Geometry + 'static {
        self.geo.add_geometry(obj);
    }
}
