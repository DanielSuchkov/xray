#![allow(dead_code)]
use brdf::{Brdf};
use camera::PerspectiveCamera;
use framebuffer::FrameBuffer;
use geometry::{Frame, Ray};
use math::vector_traits::*;
use math::{Vec2u, Vec3f, Vec2f, Zero, One, EPS_RAY, vec3_from_value};
use rand::{StdRng, Rng, SeedableRng};
use render::Render;
use scene::{Scene, SurfaceProperties};
use nalgebra::ApproxEq;

pub struct CpuPathTracer<S: Scene> {
    frame: FrameBuffer,
    scene: S,
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

impl<S> Render<S> for CpuPathTracer<S> where S: Scene {
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

    fn iterate(&mut self, iter_nb: usize) {
        let res = self.camera.get_view_size();
        let lights_nb = self.scene.get_lights_nb();
        // self.rng.reseed(&[iter_nb]);
        let (res_x, res_y) = (res.x as usize, res.y as usize);
        for pix_nb in 0..(res_x * res_y) {
            let (x, y) = (pix_nb % res_x, pix_nb / res_x);
            let jitter = if iter_nb == 0 { Vec2f::new(0.5, 0.5) } else {
                Vec2f::new(self.rng.next_f32(), self.rng.next_f32())
            };

            let sample = Vec2f::new(x as f32, y as f32) + jitter;

            let mut ray = self.camera.ray_from_screen(&sample);
            let mut path_length = 0;
            let mut path_weight = Vec3f::one();
            let mut color = Vec3f::zero();
            'current_path: loop {
                let isect = if let Some(isect) = self.scene.nearest_intersection(&ray) {
                    isect
                } else {
                    let backlight = self.scene.get_background_light();
                    backlight.radiate(&ray).map(|rad| color = path_weight * rad.radiance);
                    // // for next event estimation
                    // if let Some(back_rad) = self.scene.get_background_light().radiate(&ray) {
                    //    if path_length == 0 {
                    //        color = back_rad.radiance;
                    //    }
                    // }
                    break 'current_path;
                };
                let hit_pos = ray.orig + ray.dir * isect.dist;
                let brdf = match isect.surface {
                    SurfaceProperties::Material(mat_id) => {
                        let material = self.scene.get_material(mat_id);
                        if let Some(brdf) = Brdf::new(&ray.dir, &isect.normal, material) {
                            brdf
                        } else {
                            break 'current_path;
                        }
                    },
                    SurfaceProperties::Light(light_id) => {
                        let light = self.scene.get_light(light_id);
                        if let Some(rad) = light.radiate(&ray) {
                            color = path_weight * rad.radiance;
                        }
                        // // for next event estimation
                        // if path_length == 0 {
                        //     if let Some(rad) = self.scene.get_light(light_id).radiate(&ray) {
                        //         color = rad.radiance;
                        //     }
                        // }
                        break 'current_path;
                    }
                };

                // // next event estimation
                // for i in 0..lights_nb {
                //     let rand_light = self.scene.get_light(i as i32);
                //     let rands = (self.rng.next_f32(), self.rng.next_f32());
                //     if let Some(illum) = rand_light.illuminate(&hit_pos, rands) {
                //         if let Some(brdf_eval) = brdf.eval(&illum.l_dir) {
                //             let shadow_ray = Ray { orig: hit_pos, dir: illum.l_dir };
                //             if !self.scene.was_occluded(&shadow_ray, illum.l_dist) {
                //                 color = color + illum.radiance * path_weight * brdf_eval.radiance;
                //             }
                //         }
                //     }
                // }

                let sample_rnds = (self.rng.next_f32(), self.rng.next_f32(), self.rng.next_f32());
                if let Some(sample) = brdf.sample(sample_rnds) {
                    path_weight = path_weight * sample.radiance_factor;
                    ray.dir = sample.in_dir_world;
                    ray.orig = hit_pos + ray.dir * EPS_RAY;
                } else {
                    break 'current_path;
                }

                if path_length > MAX_PATH_LENGTH || path_weight.sqnorm() * 100.0 < self.rng.next_f32() {
                    break 'current_path;
                }

                path_length += 1;
            }
            self.frame.add_color((x, y), color);
        }
    }

    fn get_framebuffer(&self) -> &FrameBuffer {
        &self.frame
    }
}
