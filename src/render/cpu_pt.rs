use brdf::Brdf;
use camera::{Camera, PerspectiveCamera};
use framebuffer::FrameBuffer;
use math::vector_traits::*;
use math::{Vec3f, Vec2f, Zero, One};
use rand::{Rng, thread_rng};
use render::{Render, /*CpuStRender, */CpuMtRender};
use scene::{Scene, SurfaceProperties};
use std::f32::consts::PI;

const MAX_PATH_LENGTH: u32 = 100;

pub struct CpuPt<S: Scene> {
    scene: S,
    camera: PerspectiveCamera,
}

unsafe impl<S> Sync for CpuPt<S> where S: Scene {}

impl<S> CpuMtRender for CpuPt<S> where S: Scene {
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
                    if let Some(rad) = self.scene.get_light(light_id).radiate(&ray) {
                        if path_length == 0 { // caustic path
                            let max_component = rad.radiance.x.max(rad.radiance.y.max(rad.radiance.z));
                            color = rad.radiance / max_component * PI;
                        } else {
                            color = path_weight * rad.radiance;
                        }
                    }
                    break 'current_path;
                }
            };

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

impl<S> Render<S> for CpuPt<S> where S: Scene {
    fn new(cam: PerspectiveCamera, scene: S) -> CpuPt<S> {
        CpuPt {
            camera: cam,
            scene: scene,
        }
    }

    fn iterate(&self, iter_nb: usize, frame: &mut FrameBuffer) {
        self.iterate_over_screen(iter_nb, frame)
    }
}
