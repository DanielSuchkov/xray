#![allow(dead_code)]

use rand::StdRng;
use framebuffer::FrameBuffer;
use scene::Scene;
use camera::PerspectiveCamera;
use geometry::GeometryManager;
use math::Vec2u;

pub trait Render<S: Scene> {
    fn new(cam: PerspectiveCamera, scene: S) -> Self;
    fn iterate(&mut self);
    fn get_framebuffer(&self) -> &FrameBuffer;
}

struct EyeLight<S: Scene> {
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
            rng: StdRng::new().expect("cant create ranger"),
            camera: cam,
            scene: scene,
            frame: FrameBuffer::new(resolution),
        }
    }

    fn iterate(&mut self) {

    }

    fn get_framebuffer(&self) -> &FrameBuffer {
        &self.frame
    }
}

// impl EyeLight {
//     fn new() -> EyeLight {

//     }
// }
