use brdf::Brdf;
use camera::{Camera, PerspectiveCamera};
use framebuffer::RgbFrameBuffer;
use geometry::Ray;
use math::vector_traits::*;
use math::{Vec3f, Vec2f, Zero, One};
use rand::{Rng, thread_rng};
use render::{Render, /*CpuStRender, */CpuMtRender};
use scene::{Scene, SurfaceProperties};
use std::f32::consts::PI;

const MAX_PATH_LENGTH: u32 = 100;

pub struct CpuPtDl<S: Scene> {
    scene: S,
    camera: PerspectiveCamera,
}

#[allow(dead_code)]
fn balance_heuristic2(current_pdf_w: f32, other_pdf_w: f32) -> f32 {
    current_pdf_w / (current_pdf_w + other_pdf_w)
}

#[allow(dead_code)]
fn power_heuristic2(current_pdf_w: f32, other_pdf_w: f32) -> f32 {
    let current_pdf_2 = current_pdf_w * current_pdf_w;
    let other_pdf_2 = other_pdf_w * other_pdf_w;
    (current_pdf_2) / (current_pdf_2 + other_pdf_2)
}

#[allow(dead_code)]
fn max_heuristic2(current_pdf_w: f32, other_pdf_w: f32) -> f32 {
    if current_pdf_w >= other_pdf_w {
        1.0
    } else {
        0.0
    }
}

fn mis2(current_pdf_w: f32, other_pdf_w: f32) -> f32 {
    power_heuristic2(current_pdf_w, other_pdf_w)
}

impl<S> CpuPtDl<S> where S: Scene {
    fn uniform_sample_one_light(&self, p: &Vec3f, brdf: &Brdf) -> Vec3f {
        let mut ld = Vec3f::zero();

        let lights_nb = self.scene.get_lights_nb() as u32;
        let light_nb = (thread_rng().next_u32() % lights_nb) as i32;
        let rand_light = self.scene.get_light(light_nb);

        // light sampling
        let rands = (thread_rng().next_f32(), thread_rng().next_f32());
        if let Some(illum) = rand_light.illuminate(p, rands) {
            if let Some(brdf_eval) = brdf.eval(&illum.l_dir) {
                let shadow_ray = Ray { orig: *p, dir: illum.l_dir };
                if !self.scene.was_occluded(&shadow_ray, illum.l_dist) {
                    ld = ld + illum.radiance * brdf_eval.radiance;
                }
            }
        }
        ld * lights_nb as f32
    }
}

unsafe impl<S> Sync for CpuPtDl<S> where S: Scene {}

impl<S> CpuMtRender for CpuPtDl<S> where S: Scene {
    fn get_view_size(&self) -> Vec2f {
        self.camera.get_view_size()
    }

    fn trace_from_screen(&self, sample: Vec2f) -> Vec3f {
        let mut ray = self.camera.ray_from_screen(&sample);
        let mut path_length = 0;
        let mut path_weight = Vec3f::one();
        let mut color = Vec3f::zero();
        'current_path: loop {
            let isect = match self.scene.nearest_intersection(&ray) {
                Some(isect) => isect,
                None => {
                    if path_length == 0 {
                        self.scene.get_background_light().radiate(&ray).map(|rad| { color = rad.radiance; });
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
                            let max_component = rad.radiance.x.max(rad.radiance.y.max(rad.radiance.z));
                            color = rad.radiance / max_component * PI;
                        }
                    }
                    break 'current_path;
                }
            };

            color = color + self.uniform_sample_one_light(&hit_point, &brdf) * path_weight;

            let sample_rnds = (thread_rng().next_f32(), thread_rng().next_f32(), thread_rng().next_f32());
            if let Some(sample) = brdf.sample(sample_rnds) {
                path_weight = path_weight * sample.radiance;
                ray.dir = sample.wi;
                ray.orig = hit_point;
            } else {
                break 'current_path;
            }

            let russian_roulette = path_weight.sqnorm() * 100.0 < thread_rng().next_f32();
            if path_length >= MAX_PATH_LENGTH || russian_roulette {
                break 'current_path;
            }

            path_length += 1;
        }
        color
    }
}

impl<S> Render<S> for CpuPtDl<S> where S: Scene {
    fn new(cam: PerspectiveCamera, scene: S) -> CpuPtDl<S> {
        CpuPtDl {
            camera: cam,
            scene: scene,
        }
    }

    fn iterate(&self, iter_nb: usize, frame: &mut RgbFrameBuffer) {
        self.iterate_over_screen(iter_nb, frame)
    }
}
