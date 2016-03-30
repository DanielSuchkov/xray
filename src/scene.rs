#![allow(dead_code)]
use brdf::Material;
use geometry;
use geometry::{
    Geometry, GeometryManager, Ray, Surface, SurfaceIntersection,
    Isosurface, DistanceField, DFieldIsosurface
};
use light::{Light, BackgroundLight, LuminousObject, Luminous};
use math::{vec3_from_value, Vec3f, One};
use math::vector_traits::*;
use std::rc::Rc;

pub type MaterialID = i32;
pub type LightID = i32;

#[derive(Debug, Clone, Copy)]
pub enum SurfaceProperties {
    Material(MaterialID),
    Light(LightID),
}

// #[derive(Debug, Clone)]
pub struct DefaultScene<T> where T: GeometryManager {
    geo: T,
    materials: Vec<Material>,
    lights: Vec<Box<Light>>,
}

pub trait Scene {
    fn nearest_intersection(&self, ray: &Ray) -> Option<SurfaceIntersection>;
    fn was_occluded(&self, ray: &Ray, dist: f32) -> bool;

    fn add_object<G>(&mut self, geo: G, material: Material) where G: Geometry + 'static;
    fn add_isosurface<D>(&mut self, dfield: D, material: Material)
        where D: DistanceField + 'static;
    fn add_light<L>(&mut self, light: L) where L: Light + 'static;
    fn add_luminous_object<G>(&mut self, geo: G, intensity: Vec3f)
        where G: Geometry + Luminous + Clone + 'static;

    // fn bounding_sphere(&self) -> BSphere;
    fn get_material(&self, m_id: MaterialID) -> &Material;
    fn get_light(&self, m_id: LightID) -> &Box<Light>;
    fn get_lights_nb(&self) -> usize;
    fn get_background_light(&self) -> &Box<Light>;
}

impl<T> Scene for DefaultScene<T> where T: GeometryManager {
    fn nearest_intersection(&self, ray: &Ray) -> Option<SurfaceIntersection> {
        self.geo.nearest_intersection(ray)
    }

    fn was_occluded(&self, ray: &Ray, dist: f32) -> bool {
        self.geo.was_occluded(&ray, dist)
    }

    fn add_object<G>(&mut self, geo: G, material: Material)
        where G: Geometry + 'static {
        let material_id = self.materials.len() as i32;
        self.materials.push(material);
        self.geo.add_geometry(Surface {
            geometry: geo,
            properties: SurfaceProperties::Material(material_id)
        })
    }

    fn add_isosurface<D>(&mut self, dfield: D, material: Material)
        where D: DistanceField + 'static {
        let material_id = self.materials.len() as i32;
        self.materials.push(material);
        self.geo.add_isosurface(DFieldIsosurface {
            dfield: dfield,
            properties: SurfaceProperties::Material(material_id)
        })
    }

    fn get_material(&self, m_id: MaterialID) -> &Material {
        &self.materials[m_id as usize]
    }

    fn add_light<L>(&mut self, light: L) where L: Light + 'static {
        self.lights.push(Box::new(light));
    }

    fn get_light(&self, m_id: LightID) -> &Box<Light> {
        &self.lights[m_id as usize]
    }

    fn get_lights_nb(&self) -> usize {
        self.lights.len()
    }

    fn get_background_light(&self) -> &Box<Light> {
        &self.lights[0]
    }

    fn add_luminous_object<G>(&mut self, geo: G, intensity: Vec3f)
        where G: Geometry + Luminous + 'static + Clone {
        let light_id = self.lights.len() as i32;
        let light = LuminousObject { object: geo.clone(), intensity: intensity };
        self.lights.push(Box::new(light));
        self.geo.add_geometry(Surface {
            geometry: geo,
            properties: SurfaceProperties::Light(light_id)
        })
    }
}

impl<T: GeometryManager> DefaultScene<T> {
    pub fn new(backlight: BackgroundLight) -> DefaultScene<T> {
        DefaultScene {
            geo: T::new(),
            materials: Vec::new(),
            lights: vec![Box::new(backlight)]
        }
    }
}
