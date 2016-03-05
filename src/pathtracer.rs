#![allow(dead_code)]
use brdf::{Brdf, pdf_a_to_w};
use camera::PerspectiveCamera;
use framebuffer::FrameBuffer;
use geometry::{Frame, Ray};
use light::{Light, Radiance};
use math::vector_traits::*;
use math::{Vec2u, Vec3f, Vec2f, Zero, One, EPS_RAY};
use rand::{StdRng, Rng, SeedableRng};
use render::Render;
use scene::{Scene, SurfaceProperties};

pub struct CpuPathTracer<S: Scene> {
    frame: FrameBuffer,
    scene: S,
    camera: PerspectiveCamera,
    rng: StdRng,
}

fn mis(a_pdf: f32) -> f32 {
    a_pdf
}

// Mis weight for 2 pdfs
fn mis2(sample_pdf: f32, other_pdf: f32) -> f32 {
    mis(sample_pdf) / (mis(sample_pdf) + mis(other_pdf))
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
        let light_count = self.scene.get_lights_nb();
        let light_pick_prob = 1.0 / light_count as f32;
        self.rng.reseed(&[iter_nb]);
        let res = self.camera.get_view_size();
        let (res_x, res_y) = (res.x as usize, res.y as usize);
        for pix_nb in 0..(res_x * res_y) {
            let (x, y) = (pix_nb % res_x, pix_nb / res_x);
            let sample = Vec2f::new(x as f32, y as f32)
                + Vec2f::new(self.rng.next_f32(), self.rng.next_f32());
            let mut ray = self.camera.ray_from_screen(&sample);

            let mut path_weight = Vec3f::one();
            let mut color = Vec3f::zero();
            let mut path_lenght = 1;
            let mut last_pdf_w = 1.0f32;

            'current_path: loop {
                let isect = self.scene.nearest_intersection(&ray);
                let mut isect = match isect {
                    None => {
                        let backlight = self.scene.get_background_light();
                        let rad = match backlight.get_radiance(&ray.dir, Zero::zero()) {
                            None => break 'current_path,
                            Some(rad) => rad
                        };

                        let mis_weight = if path_lenght >= 1 {
                            mis2(last_pdf_w, rad.dir_pdf_a * light_pick_prob)
                        } else {
                            1.0
                        };

                        color = color + path_weight * rad.intensity * mis_weight;
                        break 'current_path;
                    },
                    Some(isect) => isect
                };

                isect.dist += EPS_RAY;
                let hit_point = ray.orig + ray.dir * isect.dist;
                let norm_frame = Frame::from_z(isect.normal);
                let brdf_opt = match isect.surface {
                    SurfaceProperties::Material(mat_id) => {
                        Brdf::new(*self.scene.get_material(mat_id), norm_frame, &ray)
                    },
                    SurfaceProperties::Light(light_id) => { // some geometry light DEAD CODE!
                        let light = self.scene.get_light(light_id);
                        if let Some(rad) = light.get_radiance(&ray.dir, hit_point) {
                            let mis_weight = if path_lenght > 1 {
                                let cos_theta = norm_frame.to_local(&-ray.dir).z;
                                let dir_pdf_w = pdf_a_to_w(rad.dir_pdf_a, isect.dist, cos_theta);
                                mis2(last_pdf_w, dir_pdf_w * light_pick_prob)
                            } else {
                                1.0
                            };
                            color = color + path_weight * mis_weight * rad.intensity;
                        }
                        break 'current_path;
                    }
                };

                let brdf = match brdf_opt {
                    Some(brdf) => brdf,
                    None => break 'current_path
                };

                if brdf.continuation_prob() == 0.0 || path_lenght > MAX_PATH_LENGTH {
                    break 'current_path;
                }

                // next event estimation
                let light_id = (self.rng.next_f32() * light_count as f32).floor() as i32;
                let light = self.scene.get_light(light_id);
                let rands = (self.rng.next_f32(), self.rng.next_f32());
                if let Some(illum) = light.illuminate(hit_point, rands) {
                    if let Some((brdf_eval, cos_theta)) = brdf.evaluate(&illum.dir_to_light) {
                        let mut brdf_pdf_w = brdf_eval.dir_pdf_w;
                        let weight = if !light.is_delta() {
                            brdf_pdf_w *= brdf.continuation_prob();
                            mis2(illum.dir_pdf_w * light_pick_prob, brdf_pdf_w)
                        } else {
                            1.0
                        };
                        let conrib_radiance = (illum.intensity * brdf_eval.radiance)
                            * (weight * cos_theta / (light_pick_prob * illum.dir_pdf_w));
                        let ray_to_light = Ray { orig: hit_point, dir: illum.dir_to_light };
                        if !self.scene.was_occluded(&ray_to_light, -illum.dist_to_light) {
                            color = color + conrib_radiance * path_weight;
                        } else {
                            color = Vec3f::new(1000.0, 0.0, 1000.0);
                        }
                    }
                }

                // calc next step
                let rands = (self.rng.next_f32(), self.rng.next_f32(), self.rng.next_f32());
                if let Some((mut sample, cos_theta)) = brdf.sample(rands) {
                    let cont_prob = brdf.continuation_prob();
                    last_pdf_w = sample.pdf_w * cont_prob;
                    if cont_prob < 1.0 { // russian roulette
                        if cont_prob < self.rng.next_f32() {
                            break 'current_path;
                        }
                        sample.pdf_w *= cont_prob;
                    }
                    path_weight = path_weight * sample.factor * (cos_theta / sample.pdf_w);
                    ray.dir = -sample.dir;
                    ray.orig = hit_point + sample.dir * EPS_RAY;
                } else {
                    break 'current_path;
                }
                path_lenght += 1;
            }
            self.frame.add_color((x, y), color);
        }
    }

    fn get_framebuffer(&self) -> &FrameBuffer {
        &self.frame
    }
}

