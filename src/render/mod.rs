#![allow(dead_code)]
use camera::PerspectiveCamera;
use framebuffer::FrameBuffer;
use math::{Vec2f, Vec3f};
use rand::{Rng, thread_rng};
use scene::Scene;

mod cpu_pt_mis;
mod eyelight;

pub use self::cpu_pt_mis::CpuPtMis;
pub use self::eyelight::EyeLight;

pub trait Render<S: Scene> {
    fn new(cam: PerspectiveCamera, scene: S) -> Self;
    fn iterate(&mut self, iter_nb: usize);
    fn get_framebuffer(&self) -> &FrameBuffer;
}

pub trait CpuRender {
    fn iterate_over_screen(&mut self, iter_nb: usize) {
        let res = self.get_view_size();
        let (res_x, res_y) = (res.x as usize, res.y as usize);
        for pix_nb in 0..(res_x * res_y) {
            let (x, y) = (pix_nb % res_x, pix_nb / res_x);
            let jitter = if iter_nb == 0 { Vec2f::new(0.5, 0.5) } else {
                Vec2f::new(thread_rng().next_f32(), thread_rng().next_f32())
            };

            let sample = Vec2f::new(x as f32, y as f32) + jitter;
            let color = self.trace_from_screen(sample);
            self.get_mut_framebuffer().add_color((x, y), color);
        }
    }

    fn trace_from_screen(&self, sample: Vec2f) -> Vec3f;
    fn get_mut_framebuffer(&mut self) -> &mut FrameBuffer;
    fn get_view_size(&self) -> Vec2f;
}

