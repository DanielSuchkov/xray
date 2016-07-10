#![allow(dead_code)]
use camera::PerspectiveCamera;
use framebuffer::FrameBuffer;
use math::{Vec2f, Vec3f};
use rand::{Rng, thread_rng};
use scene::Scene;
use rayon::prelude::*;

mod cpu_pt_mis;
mod eyelight;

pub use self::cpu_pt_mis::CpuPtMis;
pub use self::eyelight::EyeLight;

pub trait Render<S: Scene> {
    fn new(cam: PerspectiveCamera, scene: S) -> Self;
    fn iterate(&self, iter_nb: usize, frame: &mut FrameBuffer);
}

pub trait CpuStRender {
    fn iterate_over_screen(&self, iter_nb: usize, frame: &mut FrameBuffer) {
        let res_x = self.get_view_size().x as usize;
        frame.as_mut_slice().iter_mut().enumerate().all(|(pix_nb, pix)| {
            let (x, y) = (pix_nb % res_x, pix_nb / res_x);
            let jitter = if iter_nb == 0 { Vec2f::new(0.5, 0.5) } else {
                Vec2f::new(thread_rng().next_f32(), thread_rng().next_f32())
            };

            let sample = Vec2f::new(x as f32, y as f32) + jitter;
            let color = self.trace_from_screen(sample);
            *pix = *pix + color;
            true
        });
    }

    fn trace_from_screen(&self, sample: Vec2f) -> Vec3f;
    fn get_view_size(&self) -> Vec2f;
}

pub trait CpuMtRender where Self: Sync {
    fn iterate_over_screen(&self, iter_nb: usize, frame: &mut FrameBuffer) {
        let res = self.get_view_size();
        let (res_x, /*res_y*/_) = (res.x as usize, res.y as usize);
        let fb_slice = frame.as_mut_slice();
        // for pix_nb in 0..(res_x * res_y)
        fb_slice.par_iter_mut().enumerate().for_each(|(pix_nb, pix)| { // FUCK YOU. THERE IS NO SHARING HERE. IT'S SYNTACTICALLY IMPOSSIBLE!
            let (x, y) = (pix_nb % res_x, pix_nb / res_x);
            let jitter = if iter_nb == 0 { Vec2f::new(0.5, 0.5) } else {
                Vec2f::new(thread_rng().next_f32(), thread_rng().next_f32())
            };

            let sample = Vec2f::new(x as f32, y as f32) + jitter;
            let color = self.trace_from_screen(sample);
            *pix = *pix + color;
        });
    }

    fn trace_from_screen(&self, sample: Vec2f) -> Vec3f;
    fn get_view_size(&self) -> Vec2f;
}
