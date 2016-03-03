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

use sfml::graphics::{RenderWindow, Color, RenderTarget};
use sfml::window::{VideoMode, ContextSettings, event, window_style};

use brdf::Material;
use camera::{PerspectiveCamera, CameraBuilder, Camera};
// use framebuffer::FrameBuffer;
use geometry::{GeometryList, Sphere, Triangle};
use math::{Vec3f, Vec2u};
use render::{EyeLight, Render};
use scene::Scene;

fn f32_to_u8(f: f32) -> u8 {
    (f.min(1.0) * 255.0) as u8
}

fn vec3f_to_color(v: Vec3f) -> Color {
    Color::new_rgb(f32_to_u8(v.x), f32_to_u8(v.y), f32_to_u8(v.z))
}

fn main() {
    let res = Vec2u::new(800, 600);
    let mut window = RenderWindow::new(
            VideoMode::new_init(res.x as u32, res.y as u32, 32),
            "XRay",
            window_style::CLOSE,
            &ContextSettings::default())
        .expect("Cannot create a new Render Window.");

    let mut scene = scene::DefaultScene::<GeometryList>::new();

    let cam = CameraBuilder::<PerspectiveCamera>::new()
        .with_view_size(res)
        .with_pos(Vec3f::new(0.0, 0.0, -8.6))
        .with_look_at(Vec3f::new(0.0, 0.0, 1.0))
        .with_up(Vec3f::new(0.0, 1.0, 0.0))
        .with_fov(45.0)
        .with_znear(0.1)
        .with_zfar(10000.0)
        .build();


    scene.add_object(
        Sphere { center: Vec3f::new(0.8, -1.5, 0.0), radius: 0.8 },
        Material::new_identity()
    );
    scene.add_object(
        Sphere { center: Vec3f::new(-0.8, -1.5, 0.2), radius: 0.6 },
        Material::new_identity()
    );

    let cb = [
        Vec3f::new(-2.5,  2.5, -2.5), /*0*/
        Vec3f::new( 2.5,  2.5, -2.5), /*1*/
        Vec3f::new( 2.5,  2.5,  2.5), /*2*/
        Vec3f::new(-2.5,  2.5,  2.5), /*3*/
        Vec3f::new(-2.5, -2.5, -2.5), /*4*/
        Vec3f::new( 2.5, -2.5, -2.5), /*5*/
        Vec3f::new( 2.5, -2.5,  2.5), /*6*/
        Vec3f::new(-2.5, -2.5,  2.5)  /*7*/
    ];

    // floor
    scene.add_object(Triangle::new(cb[7], cb[4], cb[5]), Material::new_identity());
    scene.add_object(Triangle::new(cb[5], cb[6], cb[7]), Material::new_identity());

    // ceiling
    scene.add_object(Triangle::new(cb[0], cb[1], cb[2]), Material::new_identity());
    scene.add_object(Triangle::new(cb[2], cb[3], cb[0]), Material::new_identity());

    // back wall
    scene.add_object(Triangle::new(cb[2], cb[6], cb[7]), Material::new_identity());
    scene.add_object(Triangle::new(cb[7], cb[3], cb[2]), Material::new_identity());

    // left wall
    scene.add_object(Triangle::new(cb[3], cb[7], cb[4]), Material::new_identity());
    scene.add_object(Triangle::new(cb[4], cb[0], cb[3]), Material::new_identity());

    // right wall
    scene.add_object(Triangle::new(cb[1], cb[5], cb[6]), Material::new_identity());
    scene.add_object(Triangle::new(cb[6], cb[2], cb[1]), Material::new_identity());

    let mut ren = EyeLight::new(cam, scene);
    let mut iter_nb = 0;

    let mut im = sfml::graphics::Image::new(res.x as u32, res.y as u32).expect("Shit...");
    let mut tex = sfml::graphics::Texture::new_from_image(&im).expect("Dam it");
    while window.is_open() {
        iter_nb += 1;
        for event in window.events() {
            match event {
                event::Closed => window.close(),
                _             => {}
            }
        }

        ren.iterate(iter_nb);
        let fb = ren.get_framebuffer();
        let fb_stg = fb.as_slice();
        for x in 0..res.x {
            for y in 0..res.y{
                let col = fb_stg[fb.idx(Vec2u::new(x, y))];
                im.set_pixel(x as u32, y as u32, &vec3f_to_color(col / iter_nb as f32));
            }
        }
        println!("{:?}", iter_nb);
        tex.update_from_image(&im, 0, 0);
        let sprite = sfml::graphics::Sprite::new_with_texture(&tex).expect("Ugh...");
        window.clear(&Color::new_rgb(0, 200, 200));
        window.draw(&sprite);
        window.display()
    }
}
