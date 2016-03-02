extern crate sfml;
extern crate nalgebra;
extern crate num;
extern crate rand;

pub mod math;
pub mod geometry;
pub mod render;
pub mod pathtracer;
pub mod brdf;
pub mod scene;
pub mod framebuffer;
pub mod camera;

use sfml::graphics::{RenderWindow, Color, RenderTarget};
use sfml::window::{VideoMode, ContextSettings, event, window_style};

use scene::{Scene, DefaultScene};
use geometry::GeometryList;
// use math::{Vec3f, Vec2u};

// fn f32_to_u8(f: f32) -> u8 {
//     (f.min(1.0) * 255.0) as u8
// }

// fn vec3f_to_color(v: Vec3f) -> Color {
//     Color::new_rgb(f32_to_u8(v.x), f32_to_u8(v.y), f32_to_u8(v.z))
// }

fn main() {

    let mut window = RenderWindow::new(
            VideoMode::new_init(800, 600, 32),
            "XRay",
            window_style::CLOSE,
            &ContextSettings::default())
        .expect("Cannot create a new Render Window.");

    let mut im = sfml::graphics::Image::new(800, 600).expect("Shit...");
    let tex = sfml::graphics::Texture::new_from_image(&im).expect("Dam it");
    let sprite = sfml::graphics::Sprite::new_with_texture(&tex).expect("Ugh...");
    let mut scene = scene::DefaultScene::<GeometryList>::new();
    // scene.add_object(obj)
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
