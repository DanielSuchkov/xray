#![allow(dead_code)]
use rand::{StdRng, Rng};
use framebuffer::FrameBuffer;
use math::{Vec2u, Vec3f};
use render::Render;
use scene::Scene;
use camera::PerspectiveCamera;

struct CpuPathTracer<S: Scene> {
    frame: FrameBuffer,
    scene: S,
    camera: PerspectiveCamera,
    rng: StdRng,
}

impl<S: Scene> Render<S> for CpuPathTracer<S> {
    fn new(cam: PerspectiveCamera, scene: S) -> CpuPathTracer<S> {
        let resolution = cam.get_view_size();
        let resolution = Vec2u::new(resolution.x as usize, resolution.y as usize);
        CpuPathTracer {
            rng: StdRng::new().expect("cant create random generator"),
            camera: cam,
            scene: scene,
            frame: FrameBuffer::new(resolution),
        }
    }

    fn iterate(&mut self, _iter_nb: usize) {

    }

    fn get_framebuffer(&self) -> &FrameBuffer {
        &self.frame
    }
}

