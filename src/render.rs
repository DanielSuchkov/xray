#![allow(dead_code)]

use rand::StdRng;
use framebuffer::FrameBuffer;
use scene::{Scene, DefaultScene};
use camera::PerspectiveCamera;
use geometry::{GeometryManager, GeometryList};

pub trait Render/*<T: Scene>*/ {
    // fn new(new_scene: T) -> Self;
    fn setup_scene(&mut self);
    fn iterate(&mut self);
    fn get_framebuffer(&self) -> &FrameBuffer;
}

struct EyeLight {
    rng: StdRng,
    camera: PerspectiveCamera,
    scene: DefaultScene<GeometryList>,
    frame: FrameBuffer,
}

impl Render for EyeLight {
    fn setup_scene(&mut self) {

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
