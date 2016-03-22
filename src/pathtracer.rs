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

fn balance_heuristic(current_pdf_w: f32, other_pdf_w: f32) -> f32 {
    current_pdf_w / (current_pdf_w + other_pdf_w)
}

fn power_heuristic(current_pdf_w: f32, other_pdf_w: f32) -> f32 {
    let current_pdf_2 = current_pdf_w * current_pdf_w;
    let other_pdf_2 = other_pdf_w * other_pdf_w;
    (current_pdf_2) / (current_pdf_2 + other_pdf_2)
}

fn max_heuristic(current_pdf_w: f32, other_pdf_w: f32) -> f32 {
    if current_pdf_w >= other_pdf_w {
        1.0
    } else {
        0.0
    }
}

fn mis2(current_pdf_w: f32, other_pdf_w: f32) -> f32 {
    // max_heuristic(current_pdf_w, other_pdf_w)
    // balance_heuristic(current_pdf_w, other_pdf_w)
    power_heuristic(current_pdf_w, other_pdf_w)
}

const MAX_PATH_LENGTH: u32 = 100;

impl<S> CpuPathTracer<S> where S: Scene {
    fn uniform_sample_one_light(&mut self, p: &Vec3f, brdf: &Brdf) -> Vec3f {
        let mut ld = Vec3f::zero();

        let lights_nb = self.scene.get_lights_nb() as u32;
        let light_nb = (self.rng.next_u32() % lights_nb) as i32;
        let light_pick_prob = 1.0 / lights_nb as f32;
        // let light_pick_prob = 1.0;
        let rand_light = self.scene.get_light(light_nb);

        // brdf sampling
        let sample_rnds = (self.rng.next_f32(), self.rng.next_f32(), self.rng.next_f32());
        if let Some(sample) = brdf.sample(sample_rnds) {
            let brdf_ray = Ray { dir: sample.wi, orig: *p + sample.wi * EPS_RAY };
            if let Some(isect) = self.scene.nearest_intersection(&brdf_ray) {
                match isect.surface {
                    SurfaceProperties::Light(light_id) if light_nb == light_id => {
                        if let Some(rad) = rand_light.radiate(&brdf_ray) {
                            let weight = mis2(sample.pdf, rad.pdf * light_pick_prob);
                            ld = ld + sample.radiance * rad.radiance * weight;
                        }
                    },
                    _ => {}
                }
            } else if light_nb == 0 {
                rand_light.radiate(&brdf_ray).map(|rad| {
                    let weight = mis2(sample.pdf, rad.pdf * light_pick_prob);
                    ld = ld + sample.radiance * rad.radiance * weight;
                });
            };
        }

        // light sampling
        let rands = (self.rng.next_f32(), self.rng.next_f32());
        if let Some(illum) = rand_light.illuminate(p, rands) {
            if let Some(brdf_eval) = brdf.eval(&illum.l_dir) {
                let shadow_ray = Ray { orig: *p, dir: illum.l_dir };
                if !self.scene.was_occluded(&shadow_ray, illum.l_dist) {
                    let weight = mis2(illum.pdf * light_pick_prob, brdf_eval.pdf);
                    ld = ld + illum.radiance * brdf_eval.radiance * weight;
                }
            }
        }
        ld * lights_nb as f32
    }
}

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
                let isect = match self.scene.nearest_intersection(&ray) {
                    Some(isect) => isect,
                    None => {
                        if path_length == 0 {
                            if let Some(back_rad) = self.scene.get_background_light().radiate(&ray) {
                               color = back_rad.radiance;
                           }
                        }
                        break 'current_path;
                    }
                };
                let hit_point = ray.orig + ray.dir * isect.dist;
                let brdf = match isect.surface {
                    SurfaceProperties::Material(mat_id) => {
                        match Brdf::new(&ray.dir, &isect.normal, self.scene.get_material(mat_id)) {
                            Some(brdf) => brdf,
                            None       => break 'current_path
                        }
                    },
                    SurfaceProperties::Light(light_id) => {
                        if path_length == 0 {
                            if let Some(rad) = self.scene.get_light(light_id).radiate(&ray) {
                                color = rad.radiance;
                            }
                        }
                        break 'current_path;
                    }
                };

                color = color + self.uniform_sample_one_light(&hit_point, &brdf) * path_weight;

                let sample_rnds = (self.rng.next_f32(), self.rng.next_f32(), self.rng.next_f32());
                if let Some(sample) = brdf.sample(sample_rnds) {
                    path_weight = path_weight * sample.radiance;
                    ray.dir = sample.wi;
                    ray.orig = hit_point + ray.dir * EPS_RAY;
                } else {
                    break 'current_path;
                }

                let russian_roulette = path_weight.sqnorm() * 100.0 < self.rng.next_f32();
                if path_length > MAX_PATH_LENGTH || russian_roulette {
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
