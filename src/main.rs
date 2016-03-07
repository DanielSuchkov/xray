#![allow(unused_imports)]
extern crate sfml;
extern crate nalgebra;
extern crate num;
extern crate rand;

pub mod brdf;
pub mod camera;
pub mod framebuffer;
pub mod geometry;
pub mod light;
pub mod math;
pub mod pathtracer;
pub mod render;
pub mod scene;

use sfml::graphics::{RenderWindow, Color, RenderTarget, Texture, Sprite};
use sfml::window::{VideoMode, ContextSettings, event, window_style};

use brdf::Material;
use camera::{PerspectiveCamera, CameraBuilder, Camera};
use geometry::{GeometryList, Sphere, Triangle};
use math::{Vec3f, Vec2u, One, Zero, vec3_from_value};
use render::Render;
use light::{/*AreaLight, */PointLight, BackgroundLight};
use render::EyeLight;
use pathtracer::CpuPathTracer;
use scene::Scene;

fn f32_to_u8(f: f32) -> u8 {
    (f.min(1.0) * 255.0) as u8
}

// fn vec3f_to_color(v: Vec3f) -> Color {
//     Color::new_rgb(f32_to_u8(v.x), f32_to_u8(v.y), f32_to_u8(v.z))
// }

fn main() {
    let res = Vec2u::new(250, 250);
    let mut window = RenderWindow::new(
            VideoMode::new_init(res.x as u32, res.y as u32, 32),
            "XRay",
            window_style::CLOSE,
            &ContextSettings::default())
        .expect("Cannot create a new Render Window.");

    let cam = CameraBuilder::<PerspectiveCamera>::new()
        .with_view_size(res)
        .with_pos(Vec3f::new(0.0, 0.0, -8.6))
        .with_look_at(Vec3f::new(0.0, 0.0, 1.0))
        .with_up(Vec3f::new(0.0, 1.0, 0.0))
        .with_fov(45.0)
        .with_znear(0.1)
        .with_zfar(10000.0)
        .build();

    let white_diffuse = Material {
        diffuse: vec3_from_value(0.99),
        specular: Zero::zero(),
        phong_exp: 1.0
    };

    let green_diffuse = Material {
        diffuse: Vec3f::new(0.156863, 0.803922, 0.172549),
        // diffuse: Vec3f::new(0.0, 1.0, 0.0),
        specular: Zero::zero(),
        phong_exp: 1.0
    };

    let red_diffuse = Material {
        diffuse: Vec3f::new(0.803922, 0.152941, 0.172549),
        // diffuse: Vec3f::new(1.0, 0.0, 0.0),
        specular: Zero::zero(),
        phong_exp: 1.0
    };

    let blue_diffuse = Material {
        diffuse: Vec3f::new(0.1, 0.9, 0.9),
        specular: Zero::zero(),
        phong_exp: 1.0
    };

    let margenta_diffuse = Material {
        diffuse: Vec3f::new(0.8, 0.2, 0.6),
        specular: Zero::zero(),
        phong_exp: 1.0
    };

    let dark_mirror = Material {
        diffuse: vec3_from_value(0.0), // Vec3f::new(0.5, 0.5, 0.2) * 0.7,
        specular: vec3_from_value(0.50), // Vec3f::new(0.5, 0.5, 0.2) * 0.3,
        phong_exp: 1000.0
    };

    let cb = [
        Vec3f::new(-2.5,  2.5, -2.5), // 0
        Vec3f::new( 2.5,  2.5, -2.5), // 1
        Vec3f::new( 2.5,  2.5,  2.5), // 2
        Vec3f::new(-2.5,  2.5,  2.5), // 3
        Vec3f::new(-2.5, -2.5, -2.5), // 4
        Vec3f::new( 2.5, -2.5, -2.5), // 5
        Vec3f::new( 2.5, -2.5,  2.5), // 6
        Vec3f::new(-2.5, -2.5,  2.5)  // 7
    ];

    let daylight_color = Vec3f::new(0.65, 0.6, 0.45);
    let mut scene = scene::DefaultScene::<GeometryList>::new(
        BackgroundLight { intensity: daylight_color, scale: 4.0 }
    );

    // scene.add_light(AreaLight::new(
    //     Vec3f::new(1.0, 2.48, -2.48), Vec3f::new(-1.0, -2.48, 2.48), Vec3f::new(-1.0, 2.48, -2.48),
    //     daylight_color * 5.0
    // ));

    // scene.add_light(PointLight {
    //     position: Vec3f::new(0.0, 1.5, 0.0),
    //     intensity: daylight_color * 8.0
    // });

    {
        // floor
        scene.add_object(Triangle::new(cb[5], cb[4], cb[7]), white_diffuse);
        scene.add_object(Triangle::new(cb[7], cb[6], cb[5]), white_diffuse);

        // ceiling
        scene.add_object(Triangle::new(cb[2], cb[3], cb[0]), white_diffuse);
        scene.add_object(Triangle::new(cb[0], cb[1], cb[2]), white_diffuse);

        // back wall
        scene.add_object(Triangle::new(cb[2], cb[6], cb[7]), white_diffuse);
        scene.add_object(Triangle::new(cb[7], cb[3], cb[2]), white_diffuse);

        // left wall
        scene.add_object(Triangle::new(cb[3], cb[7], cb[4]), red_diffuse);
        scene.add_object(Triangle::new(cb[4], cb[0], cb[3]), red_diffuse);

        // right wall
        scene.add_object(Triangle::new(cb[1], cb[5], cb[6]), green_diffuse);
        scene.add_object(Triangle::new(cb[6], cb[2], cb[1]), green_diffuse);
    }

    scene.add_object(Sphere { center: Vec3f::new(-1.0, -1.4, 0.2), radius: 0.8 }, blue_diffuse);
    scene.add_object(Sphere { center: Vec3f::new(1.0, -1.9, 0.0), radius: 0.6 }, margenta_diffuse);

    let mut ren = CpuPathTracer::new(cam, scene);
    let mut iter_nb = 0;
    let mut pixels = (0..(res.x * res.y * 4)).map(|_| 255u8).collect::<Vec<_>>();

    let mut tex = Texture::new(res.x as u32, res.y as u32).expect("cant create texture");
    while window.is_open() {
        iter_nb += 1;
        for event in window.events() {
            match event {
                event::Closed => window.close(),
                _             => {}
            }
        }

        ren.iterate(iter_nb);
        let fb = ren.get_framebuffer().as_slice();

        let k = 1.0 / iter_nb as f32;
        for pix in 0..(res.x * res.y) {
            let col = fb[pix];
            pixels[pix * 4]     = f32_to_u8(col.x * k);
            pixels[pix * 4 + 1] = f32_to_u8(col.y * k);
            pixels[pix * 4 + 2] = f32_to_u8(col.z * k);
        }
        println!("{:?}", iter_nb);
        tex.update_from_pixels(&pixels, res.x as u32, res.y as u32, 0, 0);
        let sprite = Sprite::new_with_texture(&tex).expect("cant create sprite");
        window.clear(&Color::new_rgb(0, 0, 0));
        window.draw(&sprite);
        window.display();
    }
}
