#![allow(dead_code)]
// use brdf::{Brdf};
use camera::PerspectiveCamera;
use framebuffer::FrameBuffer;
use geometry::{Frame, Ray, GeometryList, GeometryManager};
use math::vector_traits::*;
use math::{Vec2u, Vec3f, Vec2f, Zero, One, EPS_RAY, EPS_COSINE, vec3_from_value};
use rand::{StdRng, Rng, SeedableRng};
use render::Render;
use scene::{/*Scene, */SurfaceProperties};
use nalgebra::ApproxEq;

pub struct CpuPathTracer/*<S: Scene>*/ {
    frame: FrameBuffer,
    // scene: S,
    geo: GeometryList,
    camera: PerspectiveCamera,
    rng: StdRng,
}

// Power heuristic
fn mis2(brdf_pdf_w: f32, ligt_dir_pdf_w: f32) -> f32 {
    let brdf_pdf_2 = brdf_pdf_w * brdf_pdf_w;
    let light_dir_pdf_2 = ligt_dir_pdf_w * ligt_dir_pdf_w;
    (brdf_pdf_2) / (brdf_pdf_2 + light_dir_pdf_2)
}

const MAX_PATH_LENGTH: u32 = 100;

impl/*<S>*/ Render/*<S>*/ for CpuPathTracer/*<S> where S: Scene*/ {
    fn new(cam: PerspectiveCamera/*, scene: S*/) -> CpuPathTracer/*<S>*/ {
        let resolution = cam.get_view_size();
        let resolution = Vec2u::new(resolution.x as usize, resolution.y as usize);
        CpuPathTracer {
            rng: StdRng::new().expect("cant create random generator"),
            camera: cam,
            geo: GeometryList::new(),
            // scene: scene,
            frame: FrameBuffer::new(resolution),
        }
    }

    fn iterate(&mut self, iter_nb: usize) {
        let res = self.camera.get_view_size();
        self.rng.reseed(&[iter_nb]);
        let (res_x, res_y) = (res.x as usize, res.y as usize);
        for pix_nb in 0..(res_x * res_y) {
            let (x, y) = (pix_nb % res_x, pix_nb / res_x);
            let sample = Vec2f::new(x as f32, y as f32) + if iter_nb == 0 {
                Vec2f::new(0.5, 0.5)
            } else {
                Vec2f::new(self.rng.next_f32(), self.rng.next_f32())
            };

            let mut ray = self.camera.ray_from_screen(&sample);
            let mut path_length = 0;
            let mut path_weight = Vec3f::one();
            let mut color = Vec3f::zero();
            'current_path: loop {
                path_length += 1;
            }
            self.frame.add_color((x, y), color);
        }
    }

    fn get_framebuffer(&self) -> &FrameBuffer {
        &self.frame
    }
}
