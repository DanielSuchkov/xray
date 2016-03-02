#![allow(dead_code)]
// use render::Render;
use framebuffer::FrameBuffer;
use scene::Scene;
// use geometry::{GeometryList};
// use math::{Vec3f, Vec2u};

struct CpuPathTracer<T: Scene> {
    frame: FrameBuffer,
    scene: T,

}

// impl<T: Scene> Render<T> for CpuPathTracer<T> {
//     fn setup_scene(&mut self) {

//     }

//     fn iterate(&mut self) {

//     }

//     fn get_framebuffer(&self) -> &FrameBuffer {
//         &self.frame
//     }
// }

