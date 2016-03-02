#![allow(dead_code)]

use geometry::{GeometryManager, Ray, Intersection};

pub type MaterialID = i32;
pub type LightID = i32;

#[derive(Debug, Clone)]
pub struct DefaultScene<T> where T: GeometryManager {
    geo: T
}

pub trait Scene {
    fn new() -> Self;
    fn nearest_intersection(&self, ray: &Ray) -> Option<Intersection>;
}

impl<T> Scene for DefaultScene<T> where T: GeometryManager {
    fn new() -> DefaultScene<T> {
        DefaultScene {
            geo: T::new()
        }
    }

    fn nearest_intersection(&self, ray: &Ray) -> Option<Intersection> {
        self.geo.nearest_intersection(ray)
    }
}
