extern crate sfml;
extern crate nalgebra;
extern crate num;
extern crate rand;
extern crate rayon;

pub mod brdf;
pub mod camera;
pub mod framebuffer;
pub mod geometry;
pub mod light;
pub mod math;
pub mod render;
pub mod scene;
pub mod utility;
pub mod materials_and_colors;

use sfml::graphics::{RenderWindow, Color, RenderTarget, Texture, Sprite};
use sfml::window::{VideoMode, ContextSettings, event, window_style};

use camera::{Camera, PerspectiveCamera, CameraBuilder};
use geometry::{GeometryList, Sphere, Torus, Triangle, DFieldsSubstr, DFieldsBlend, RoundBox};
use math::{Vec3f, Vec2u, Zero};
use render::Render;
use light::{PointLight, BackgroundLight};
#[allow(unused_imports)]
use render::{EyeLight, CpuPt, CpuPtMis, CpuPtDl};
use scene::Scene;
use std::io::prelude::*;
use materials_and_colors::*;

fn f32_to_u8(f: f32) -> u8 {
    (f.min(1.0) * 255.0) as u8
}

const CB: [Vec3f; 8] = [
    Vec3f { x: -1.0, y:  1.0, z: -1.0 }, // 0
    Vec3f { x:  1.0, y:  1.0, z: -1.0 }, // 1
    Vec3f { x:  1.0, y:  1.0, z:  1.0 }, // 2
    Vec3f { x: -1.0, y:  1.0, z:  1.0 }, // 3
    Vec3f { x: -1.0, y: -1.0, z: -1.0 }, // 4
    Vec3f { x:  1.0, y: -1.0, z: -1.0 }, // 5
    Vec3f { x:  1.0, y: -1.0, z:  1.0 }, // 6
    Vec3f { x: -1.0, y: -1.0, z:  1.0 }  // 7
];

fn add_cornell_box<S>(scene: &mut S, scale: f32) where S: Scene {
    // floor
    scene.add_object(Triangle::new(CB[5] * scale, CB[4] * scale, CB[7] * scale), WHITE_DIFFUSE);
    scene.add_object(Triangle::new(CB[7] * scale, CB[6] * scale, CB[5] * scale), WHITE_DIFFUSE);

    // ceiling
    scene.add_object(Triangle::new(CB[2] * scale, CB[3] * scale, CB[0] * scale), WHITE_DIFFUSE);
    scene.add_object(Triangle::new(CB[0] * scale, CB[1] * scale, CB[2] * scale), WHITE_DIFFUSE);

    // back wall
    scene.add_object(Triangle::new(CB[2] * scale, CB[6] * scale, CB[7] * scale), WHITE_DIFFUSE);
    scene.add_object(Triangle::new(CB[7] * scale, CB[3] * scale, CB[2] * scale), WHITE_DIFFUSE);

    // left wall
    scene.add_object(Triangle::new(CB[3] * scale, CB[7] * scale, CB[4] * scale), RED_DIFFUSE);
    scene.add_object(Triangle::new(CB[4] * scale, CB[0] * scale, CB[3] * scale), RED_DIFFUSE);

    // right wall
    scene.add_object(Triangle::new(CB[1] * scale, CB[5] * scale, CB[6] * scale), GREEN_DIFFUSE);
    scene.add_object(Triangle::new(CB[6] * scale, CB[2] * scale, CB[1] * scale), GREEN_DIFFUSE);
}

#[allow(dead_code)]
fn setup_mis_showcase() -> scene::DefaultScene<GeometryList> {
    let mut scene = scene::DefaultScene::<GeometryList>::new(
        BackgroundLight { intensity: DAYLIGHT_COLOR * 0.25 }
    );

    scene.add_luminous_object(
        Sphere { center: Vec3f::new(0.0, 25.0, 0.0), radius: 5.0 },
        DAYLIGHT_COLOR * 40.0
    );

    add_cornell_box(&mut scene, 25.0);

    scene.add_object(Sphere { center: Vec3f::new(-16.0, -18.0, 2.0), radius: 7.0 }, MIRROR);
    scene.add_object(Sphere { center: Vec3f::new(0.0, -18.0, 0.0), radius: 7.0 }, WHITE_CERAMICS);
    scene.add_object(Sphere { center: Vec3f::new(16.0, -18.0, 0.0), radius: 7.0 }, WHITE_DIFFUSE);

    scene
}

#[allow(dead_code)]
fn setup_pointlight_showcase() -> scene::DefaultScene<GeometryList> {
    let mut scene = scene::DefaultScene::<GeometryList>::new(
        BackgroundLight { intensity: DAYLIGHT_COLOR * 0.0 }
    );

    scene.add_light(PointLight {
        position: Vec3f::new(-20.0, -5.0, 0.0),
        intensity: DAYLIGHT_COLOR * 1500.0
    });

    scene.add_light(PointLight {
        position: Vec3f::new(20.0, -10.0, -10.0),
        intensity: DAYLIGHT_COLOR * 1500.0
    });

    scene.add_light(PointLight {
        position: Vec3f::new(-20.0, 10.0, 20.0),
        intensity: MARGENTA_COLOR * 1000.0
    });

    scene.add_light(PointLight {
        position: Vec3f::new(20.0, 10.0, 20.0),
        intensity: SKY_BLUE_COLOR * 1000.0
    });

    scene.add_luminous_object(
        Sphere { center: Vec3f::new(10.0, -3.0, 5.0), radius: 5.0 },
        GOLDEN_COLOR * 7.0
    );

    add_cornell_box(&mut scene, 25.0);

    scene.add_object(Sphere { center: Vec3f::new(-16.0, -18.0, 2.0), radius: 7.0 }, MIRROR);
    scene.add_object(Sphere { center: Vec3f::new(0.0, -18.0, 7.0), radius: 7.0 }, GOLDEN_SPEC);
    scene.add_object(Sphere { center: Vec3f::new(16.0, -18.0, 0.0), radius: 7.0 }, SKY_BLUE_DIFFUSE);

    scene
}

#[allow(dead_code)]
fn setup_df_showcase() -> scene::DefaultScene<GeometryList> {
    let mut scene = scene::DefaultScene::<GeometryList>::new(
        BackgroundLight { intensity: DAYLIGHT_COLOR * 0.5 }
    );

    scene.add_luminous_object(
        Sphere { center: Vec3f::new(0.0, 25.0, 0.0), radius: 5.0 },
        DAYLIGHT_COLOR * 30.0
    );

    add_cornell_box(&mut scene, 25.0);

    scene.add_isosurface(
        DFieldsSubstr {
            a: DFieldsBlend {
                a: DFieldsBlend {
                    a: Torus { radius: 3.0, thickness: 1.5, center: Vec3f::new(-5.0, 0.0, 0.0) },
                    b: Torus { radius: 5.0, thickness: 2.5, center: Vec3f::new( 5.0, 0.0, 0.0) },
                    k: 5.0,
                    pos: Vec3f::zero()
                },
                b: Sphere { center: Vec3f::new(12.0, 2.0, -4.0), radius: 4.0 },
                k: 5.0,
                pos: Vec3f::zero()
            },
            b: Sphere { center: Vec3f::new(2.0, 4.0, 1.0), radius: 5.0 },
            // pos: Vec3f::new(-13.0, -16.0, -8.0)
            pos: Vec3f::new(-3.0, -7.0, 5.0),
        },
        GOLDEN_SPEC
    );

    scene.add_isosurface(
        DFieldsSubstr {
            a: RoundBox {
                pos: Vec3f::zero(),
                dim: Vec3f::new(4.0, 4.0, 4.0),
                r: 3.0
            },
            b: Torus {
                radius: 4.0,
                thickness: 4.0,
                center: Vec3f::new(0.0, 0.0, -3.0)
            },
            pos: Vec3f::new(-13.0, -18.0, -5.0)
        },
        WHITE_CERAMICS
    );

    scene.add_isosurface(
        DFieldsSubstr {
            a: DFieldsSubstr {
                a: RoundBox {
                    pos: Vec3f::zero(),
                    dim: Vec3f::new(4.0, 4.0, 4.0),
                    r: 2.0
                },
                b: Sphere { center: Vec3f::new(0.0, 4.0, 0.0), radius: 5.0 },
                pos: Vec3f::zero(),
            },
            b: Sphere { center: Vec3f::new(0.0, 0.0, -4.0), radius: 5.0 },
            pos: Vec3f::new(12.0, -19.0, -4.0)
        },
        MIRROR
    );

    scene
}

#[allow(dead_code)]
fn setup_df_blend_showcase() -> scene::DefaultScene<GeometryList> {
    let mut scene = scene::DefaultScene::<GeometryList>::new(
        BackgroundLight { intensity: DAYLIGHT_COLOR * 0.5 }
    );

    scene.add_luminous_object(
        Sphere { center: Vec3f::new(0.0, 25.0, 0.0), radius: 5.0 },
        DAYLIGHT_COLOR * 30.0
    );

    add_cornell_box(&mut scene, 25.0);
    scene.add_isosurface(
        DFieldsBlend {
            a: Sphere { center: Vec3f::new(7.0, 0.0, 0.0), radius: 7.0 },
            b: Sphere { center: Vec3f::new(-7.0, 0.0, 0.0), radius: 7.0 },
            // pos: Vec3f::new(3.0, -11.0, 4.5),
            pos: Vec3f::new(0.0, -7.0, 0.0),
            k: 6.0
        },
        MIRROR
    );

    scene
}

fn main() {
    let res = Vec2u::new(1000, 1000);
    // let res = Vec2u::new(500, 500);
    // let res = Vec2u::new(250, 250);
    let mut window = RenderWindow::new(
            VideoMode::new_init(res.x as u32, res.y as u32, 32),
            "XRay",
            window_style::CLOSE,
            &ContextSettings::default())
        .expect("Cannot create a new Render Window.");

    let cam = CameraBuilder::<PerspectiveCamera>::new()
        .with_view_size(res)
        .with_pos(Vec3f::new(0.0, 0.0, -86.0))
        .with_look_at(Vec3f::new(0.0, 0.0, 1.0))
        .with_up(Vec3f::new(0.0, 1.0, 0.0))
        .with_fov(45.0)
        .with_znear(0.1)
        .with_zfar(10000.0)
        .build();

    let mut frame = cam.build_framebuffer();

    let scene = setup_mis_showcase();
    // let scene = setup_df_showcase();
    // let scene = setup_df_blend_showcase();
    // let scene = setup_pointlight_showcase();

    let ren = CpuPtDl::new(cam, scene);
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

        ren.iterate(iter_nb, &mut frame);
        let fb = frame.as_slice();

        let k = 1.0 / iter_nb as f32;
        for pix in 0..(res.x * res.y) {
            let col = fb[pix] * k;
            pixels[pix * 4]     = f32_to_u8(col.x);
            pixels[pix * 4 + 1] = f32_to_u8(col.y);
            pixels[pix * 4 + 2] = f32_to_u8(col.z);
        }
        print!("\r{} spp", iter_nb);
        std::io::stdout().flush().ok().expect("Could not flush stdout");
        tex.update_from_pixels(&pixels, res.x as u32, res.y as u32, 0, 0);
        let sprite = Sprite::new_with_texture(&tex).expect("cant create sprite");
        window.clear(&Color::new_rgb(0, 0, 0));
        window.draw(&sprite);
        window.display();
    }
}
