#![allow(dead_code)]
use camera::PerspectiveCamera;
use framebuffer::RgbFrameBuffer;
use math::{Vec2f, Vec3f};
use rand::{Rng, thread_rng};
use scene::Scene;
use rayon::prelude::*;

mod cpu_pt_mis;
mod eyelight;
mod cpu_pt;
mod cpu_pt_dl;

pub use self::cpu_pt_mis::CpuPtMis;
pub use self::eyelight::EyeLight;
pub use self::cpu_pt::CpuPt;
pub use self::cpu_pt_dl::CpuPtDl;

pub trait Render<S: Scene> {
    fn new(cam: PerspectiveCamera, scene: S) -> Self;
    fn iterate(&self, iter_nb: usize, frame: &mut RgbFrameBuffer);
}

pub trait CpuStRender {
    fn iterate_over_screen(&self, _iter_nb: usize, frame: &mut RgbFrameBuffer) {
        let res_x = self.get_view_size().x as usize;
        frame.as_mut_slice().iter_mut().enumerate().all(|(pix_nb, pix)| {
            let (x, y) = (pix_nb % res_x, pix_nb / res_x);
            let jitter = Vec2f::new(thread_rng().next_f32(), thread_rng().next_f32());
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
    fn iterate_over_screen(&self, _iter_nb: usize, frame: &mut RgbFrameBuffer) {
        let res_x = self.get_view_size().x as usize;
        frame.as_mut_slice().par_iter_mut().enumerate().for_each(|(pix_nb, pix)| {
            let (x, y) = (pix_nb % res_x, pix_nb / res_x);
            let jitter = Vec2f::new(thread_rng().next_f32(), thread_rng().next_f32());
            let sample = Vec2f::new(x as f32, y as f32) + jitter;
            let color = self.trace_from_screen(sample);
            *pix = *pix + color;
        });
    }

    fn trace_from_screen(&self, sample: Vec2f) -> Vec3f;
    fn get_view_size(&self) -> Vec2f;
}
