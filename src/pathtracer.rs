#![allow(dead_code)]
use framebuffer::FrameBuffer;
// use geometry::{GeometryList};
// use math::{Vec2u, Vec3f};
// use render::Render;
use scene::Scene;

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

