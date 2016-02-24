extern crate sfml;
extern crate cgmath;

use sfml::graphics::{RenderWindow, Color, RenderTarget};
use sfml::window::{VideoMode, ContextSettings, event, window_style};
use cgmath::{Vector, Vector3, EuclideanVector};
use std::f32;
use std::ops::{Div, Mul};

type Vec3f = Vector3<f32>;

trait FloatExtra {
    fn to_radian(self) -> Self;
}

impl<T> FloatExtra for T where T: Div<Output=T> + Mul<Output=T> + From<f32> {
    fn to_radian(self) -> T {
        self * From::from(std::f32::consts::PI) / From::from(180.0)
    }
}

fn mix(a: f32, b: f32, mix: f32) -> f32 {
    b * mix + a * (1.0 - mix)
}

fn f32_to_u8(f: f32) -> u8 {
    (f.min(1.0) * 255.0) as u8
}

fn vec3f_to_color(v: Vec3f) -> Color {
    Color::new_rgb(f32_to_u8(v.x), f32_to_u8(v.y), f32_to_u8(v.z))
}

#[derive(Debug, Clone)]
struct Ray {
    origin: Vec3f,
    dir: Vec3f,
}

#[derive(Debug, Clone)]
struct Sphere {
    center: Vec3f,
    radius: f32,
    surface_color: Vec3f,
    emission_color: Vec3f,
    transparancy: f32,
    reflection: f32,
}

impl Sphere {
    fn new(center: Vec3f, radius: f32, surface_color: Vec3f) -> Sphere {
        Sphere {
            center: center,
            radius: radius,
            surface_color: surface_color,
            emission_color: Vector::from_value(0.0),
            transparancy: 0.0,
            reflection: 0.0,
        }
    }

    fn with_transparancy(mut self, transp: f32) -> Sphere {
        self.transparancy = transp;
        self
    }

    fn with_reflectivity(mut self, reflect: f32) -> Sphere {
        self.reflection = reflect;
        self
    }

    fn with_emission(mut self, emission_color: Vec3f) -> Sphere {
        self.emission_color = emission_color;
        self
    }

    fn r2(&self) -> f32 {
        self.radius * self.radius
    }

    fn intersect(&self, ray: &Ray) -> Option<(f32, f32)> {
        let l = self.center - ray.origin;
        let tca = l.dot(ray.dir);
        if tca < 0.0 {
            return None;
        }
        let d2 = l.dot(l) - tca * tca;
        if d2 > self.r2() {
            return None;
        }
        let thc = (self.r2() - d2).sqrt();
        let t0 = tca - thc;
        let t1 = tca + thc;
        Some((t0, t1))
    }
}

fn find_nearest_intersection<'a>(ray: &Ray, spheres: &'a [Sphere])
    -> (Option<&'a Sphere>, f32) {
    let mut sphere = None;
    let mut tnear = f32::INFINITY;
    for obj in spheres {
        if let Some((mut t0, t1)) = obj.intersect(ray) {
            if t0 < 0.0 {
                t0 = t1;
            }
            if t0 < tnear {
                tnear = t0;
                sphere = Some(obj);
            }
        }
    }
    (sphere, tnear)
}

fn trace(ray: &Ray, objects: &[Sphere], depth: i32, max_depth: i32) -> Vec3f {
    let (sphere, tnear) = find_nearest_intersection(ray, objects);

    let sphere = match sphere {
        Some(sph) => sph,
        None => return Vec3f::from_value(2.0)
    };

    let point_hit = ray.origin + ray.dir * tnear;
    let mut noraml_hit = (point_hit - sphere.center).normalize();

    let mut is_inside = false;
    if ray.dir.dot(noraml_hit) > 0.0 {
        noraml_hit.neg_self();
        is_inside = true;
    }
    let normal_bias = noraml_hit * (1.0e-4f32);

    let mut surface_color = Vec3f::from_value(0.0);
    if (sphere.transparancy > 0.0 || sphere.reflection > 0.0) && depth < max_depth {
        let facing_ratio = -ray.dir.dot(noraml_hit);
        let frensel_effect = mix((1.0 - facing_ratio).powi(3), 1.0, 0.1);
        let refl_dir = (ray.dir - noraml_hit * 2.0 * ray.dir.dot(noraml_hit)).normalize();
        let refl_ray = Ray { origin: point_hit + normal_bias, dir: refl_dir };
        let refl_col = trace(&refl_ray, objects, depth + 1, max_depth);
        let refr_col = if sphere.transparancy > 0.0 {
            let refr_intens = 1.15;
            let eta = if is_inside { refr_intens } else { 1.0 / refr_intens };
            let cosi = -noraml_hit.dot(ray.dir);
            let k = 1.0 - eta * eta * (1.0 - cosi * cosi);
            let refr_dir = (ray.dir * eta + noraml_hit * (eta * cosi - k.sqrt())).normalize();
            let refr_ray = Ray { origin: point_hit - normal_bias, dir: refr_dir };
            trace(&refr_ray, objects, depth + 1, max_depth)
        } else {
            Vec3f::from_value(0.0)
        };
        surface_color = (refl_col * frensel_effect * sphere.reflection
            + refr_col * (1.0 - frensel_effect) * sphere.transparancy) * sphere.surface_color;
    } else {
        for i in 0..objects.len() {
            if objects[i].emission_color.x > 0.0 {
                let mut transmission = 1.0;
                let light_dir = (objects[i].center - point_hit).normalize();
                for j in 0..objects.len() {
                    if i != j {
                        let light_ray = Ray { origin: point_hit + normal_bias, dir: light_dir };
                        if objects[j].intersect(&light_ray).is_some() {
                            transmission = 0.0;
                            break;
                        }
                    }
                }
                surface_color = surface_color + sphere.surface_color * transmission
                    * noraml_hit.dot(light_dir).max(0.0) * objects[i].emission_color;
            }
        }
    }
    surface_color + sphere.emission_color
}

fn render<F>(width: usize, height: usize, objects: &[Sphere], mut draw_pixel: F)
    where F: FnMut(usize, usize, Vec3f) {
    let inv_width = 1.0 / width as f32;
    let inv_height = 1.0 / height as f32;
    let fov: f32 = 30.0;
    let aspect_ratio = width as f32 / height as f32;
    let angle = (fov.to_radian() * 0.5).tan();
    for y in 0..height {
        for x in 0..width {
            let xx = (2.0 * ((x as f32 + 0.5) * inv_width) - 1.0) * angle * aspect_ratio;
            let yy = (1.0 - 2.0 * ((y as f32 + 0.5) * inv_height)) * angle;
            let ray = Ray { origin: Vec3f::from_value(0.0), dir: Vec3f::new(xx, yy, -1.0).normalize() };
            draw_pixel(x, y, trace(&ray, objects, 0, 8));
        }
    }
}

fn main() {
    let mut window = RenderWindow::new(
            VideoMode::new_init(800, 600, 32),
            "XRay",
            window_style::CLOSE,
            &ContextSettings::default())
        .expect("Cannot create a new Render Window.");

    let mut im = sfml::graphics::Image::new(800, 600).expect("Shit...");

    let spheres = vec![
        Sphere::new(Vec3f::new(0.0, -10004.0, -20.0), 10000.0, Vec3f::new(0.20, 0.20, 0.20)),
        Sphere::new(Vec3f::new(0.0,      0.0, -20.0), 4.0, Vec3f::new(1.00, 0.32, 0.36))
            .with_reflectivity(0.9)
            .with_transparancy(0.5),
        Sphere::new(Vec3f::new(5.0,     -1.0, -15.0), 2.0, Vec3f::new(1.00, 0.86, 0.46))
            .with_reflectivity(0.9),
        Sphere::new(Vec3f::new(5.0,      0.0, -25.0), 3.0, Vec3f::new(0.65, 0.77, 0.97))
            .with_reflectivity(0.9),
        Sphere::new(Vec3f::new(-5.5,     0.0, -15.0), 3.0, Vec3f::new(0.90, 0.90, 0.90))
            .with_reflectivity(0.9),
        Sphere::new(Vec3f::new(0.0,     20.0, -30.0), 3.0, Vec3f::from_value(0.0))
            .with_emission(Vec3f::from_value(4.0))
    ];

    render(800, 600, &spheres, |x, y, col| im.set_pixel(x as u32, y as u32, &vec3f_to_color(col)));

    let tex = sfml::graphics::Texture::new_from_image(&im).expect("Dam it");
    let sprite = sfml::graphics::Sprite::new_with_texture(&tex).expect("Ugh...");

    while window.is_open() {
        for event in window.events() {
            match event {
                event::Closed => window.close(),
                _             => {}
            }
        }

        window.clear(&Color::new_rgb(0, 200, 200));
        window.draw(&sprite);
        window.display()
    }
}
