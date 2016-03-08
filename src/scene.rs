#![allow(dead_code)]
use brdf::Material;
use geometry;
use geometry::{/*BSphere, */Geometry, GeometryManager, Ray, Surface, SurfaceIntersection};
use light::{Light, BackgroundLight};
use math::{vec3_from_value, Vec3f, EPS_RAY, One};
use math::vector_traits::*;

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
    fn add_light<L>(&mut self, light: L) where L: Light + 'static;
    fn add_luminous_object<L, G>(&mut self, light: L, geo: G)
        where L: Light + 'static, G: Geometry + 'static;

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
        let ray = Ray { orig: ray.orig + ray.dir * EPS_RAY, dir: ray.dir };
        self.geo.was_occluded(&ray, dist - 2.0 * EPS_RAY)
    }

    fn add_object<G>(&mut self, geo: G, material: Material)
        where G: Geometry + 'static {
        let material_id = self.materials.len() as i32;
        self.materials.push(material);
        self.geo.add_geometry(Surface {
            geometry: geo,
            properties: SurfaceProperties::Material(material_id)
        });
    }

    // fn bounding_sphere(&self) -> BSphere {
    //     let aabb = self.geo.build_aabbox();
    //     let radius2 = (aabb.max - aabb.min).sqnorm();
    //     BSphere {
    //         center: (aabb.min + aabb.max) * 0.5,
    //         radius: radius2.sqrt(),
    //         inv_radius_sqr: 1.0 / radius2
    //     }
    // }

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

    fn add_luminous_object<L, G>(&mut self, light: L, geo: G)
        where L: Light + 'static,
              G: Geometry + 'static {
        let light_id = self.lights.len() as i32;
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
