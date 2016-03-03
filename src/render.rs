#![allow(dead_code)]
use camera::PerspectiveCamera;
use framebuffer::FrameBuffer;
use geometry::GeometryManager;
use math::vector_traits::*;
use math::{Vec2u, Vec2f, vec3_from_value};
use rand::{Rng, StdRng};
use scene::Scene;

pub trait Render<S: Scene> {
    fn new(cam: PerspectiveCamera, scene: S) -> Self;
    fn iterate(&mut self, iter_nb: usize);
    fn get_framebuffer(&self) -> &FrameBuffer;
}

pub struct EyeLight<S: Scene> {
    rng: StdRng,
    camera: PerspectiveCamera,
    scene: S,
    frame: FrameBuffer,
}

impl<S> Render<S> for EyeLight<S> where S: Scene {
    fn new(cam: PerspectiveCamera, scene: S) -> EyeLight<S> {
        let resolution = cam.get_view_size();
        let resolution = Vec2u::new(resolution.x as usize, resolution.y as usize);
        EyeLight {
            rng: StdRng::new().expect("cant create random generator"),
            camera: cam,
            scene: scene,
            frame: FrameBuffer::new(resolution),
        }
    }

    fn iterate(&mut self, iter_nb: usize) {
        let res = self.camera.get_view_size();
        let (res_x, res_y) = (res.x as usize, res.y as usize);
        for x in 0..res_x {
            for y in 0..res_y {
                let sample = Vec2f::new(x as f32, y as f32);
                let sample = sample + if iter_nb == 1 {
                    Vec2f::new(0.5, 0.5)
                } else {
                    Vec2f::new(self.rng.next_f32(), self.rng.next_f32())
                };

                let ray = self.camera.ray_from_screen(&sample);

                if let Some(isect) = self.scene.nearest_intersection(&ray) {
                    let l_dot_n = isect.normal.dot(&-ray.dir);
                    self.frame.add_color(
                        Vec2u::new(x, y), vec3_from_value(l_dot_n.max(-l_dot_n))
                    );
                } else {
                    self.frame.add_color(Vec2u::new(x, y), vec3_from_value(0.5));
                }
            }
        }
    }

    fn get_framebuffer(&self) -> &FrameBuffer {
        &self.frame
    }
}

// impl EyeLight {
//     fn new() -> EyeLight {

//     }
// }
