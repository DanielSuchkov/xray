#![allow(dead_code)]
// use brdf::{Brdf};
use brdf;
use brdf::Material;
use camera::PerspectiveCamera;
use framebuffer::FrameBuffer;
use geometry::{Frame, Ray, GeometryList, GeometryManager, Sphere, Triangle, Surface};
use math::vector_traits::*;
use math::{Vec2u, Vec3f, Vec2f, Zero, One, EPS_RAY, EPS_COSINE, vec3_from_value};
use rand::{StdRng, Rng, SeedableRng};
use render::Render;
use scene::{/*Scene, */SurfaceProperties};
use nalgebra::ApproxEq;
use std::f32::consts::{PI, FRAC_1_PI};

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

fn get_material(id: i32) -> Material {
    if id == 0 {
        Material {
            diffuse: vec3_from_value(0.99),
            specular: Zero::zero(),
            phong_exp: 1.0
        }
    } else if id == 1 {
        Material {
            diffuse: Vec3f::new(0.156863, 0.803922, 0.172549),
            // diffuse: Vec3f::new(0.0, 1.0, 0.0),
            specular: Zero::zero(),
            phong_exp: 1.0
        }
    } else {
        Material {
            diffuse: Vec3f::new(0.803922, 0.152941, 0.172549),
            // diffuse: Vec3f::new(1.0, 0.0, 0.0),
            specular: Zero::zero(),
            phong_exp: 1.0
        }
    }
}

fn get_light_color(_id: i32) -> Vec3f {
    vec3_from_value(4.0)
}

fn light_pos() -> Vec3f {
    Vec3f::new(0.0, 1.7, 0.0)
}

fn radius() -> f32 {
    0.7
}

fn uniform_cone_sample(cos_a_max: f32, rnd: (f32, f32)) -> Vec3f {
    let cos_a = 1.0 - rnd.0 * (1.0 - cos_a_max);
    let sin_a = (1.0 - cos_a * cos_a).sqrt();
    let phi = 2.0 * PI * rnd.1;
    Vec3f {
        x: phi.cos() * sin_a, y: phi.sin() * sin_a, z: cos_a
    }
}

fn sample_brdf(norm: &Vec3f, rnd: (f32, f32)) -> Vec3f {
    let dir_local = brdf::cos_hemisphere_sample_w(rnd);
    let norm_basis = Frame::from_z(norm);
    norm_basis.to_world(&dir_local)
}

fn sample_light(rnd: (f32, f32)) -> Vec3f {
    let dir = brdf::uniform_sphere_sample_w(rnd);
    dir * radius()
}

impl/*<S>*/ Render/*<S>*/ for CpuPathTracer/*<S> where S: Scene*/ {
    fn new(cam: PerspectiveCamera/*, scene: S*/) -> CpuPathTracer/*<S>*/ {
        let resolution = cam.get_view_size();
        let resolution = Vec2u::new(resolution.x as usize, resolution.y as usize);
        let mut geo = GeometryList::new();
        {
            let cb = [
                Vec3f::new(-2.5,  2.5, -2.5), // 0
                Vec3f::new( 2.5,  2.5, -2.5), // 1
                Vec3f::new( 2.5,  2.5,  2.5), // 2
                Vec3f::new(-2.5,  2.5,  2.5), // 3
                Vec3f::new(-2.5, -2.5, -2.5), // 4
                Vec3f::new( 2.5, -2.5, -2.5), // 5
                Vec3f::new( 2.5, -2.5,  2.5), // 6
                Vec3f::new(-2.5, -2.5,  2.5)  // 7
            ];
            // floor
            geo.add_geometry(Surface {
                geometry: Triangle::new(cb[5], cb[4], cb[7]),
                properties: SurfaceProperties::Material(0)
            });
            geo.add_geometry(Surface {
                geometry: Triangle::new(cb[7], cb[6], cb[5]),
                properties: SurfaceProperties::Material(0)
            });

            // ceiling
            geo.add_geometry(Surface {
                geometry: Triangle::new(cb[2], cb[3], cb[0]),
                properties: SurfaceProperties::Material(0)
            });
            geo.add_geometry(Surface {
                geometry: Triangle::new(cb[0], cb[1], cb[2]),
                properties: SurfaceProperties::Material(0)
            });

            // back wall
            geo.add_geometry(Surface {
                geometry: Triangle::new(cb[2], cb[6], cb[7]),
                properties: SurfaceProperties::Material(0)
            });
            geo.add_geometry(Surface {
                geometry: Triangle::new(cb[7], cb[3], cb[2]),
                properties: SurfaceProperties::Material(0)
            });

            // left wall
            geo.add_geometry(Surface {
                geometry: Triangle::new(cb[3], cb[7], cb[4]),
                properties: SurfaceProperties::Material(1)
            });

            geo.add_geometry(Surface {
                geometry: Triangle::new(cb[4], cb[0], cb[3]),
                properties: SurfaceProperties::Material(1)
            });

            // right wall
            geo.add_geometry(Surface {
                geometry: Triangle::new(cb[1], cb[5], cb[6]),
                properties: SurfaceProperties::Material(2)
            });
            geo.add_geometry(Surface {
                geometry: Triangle::new(cb[6], cb[2], cb[1]),
                properties: SurfaceProperties::Material(2)
            });
        }
        geo.add_geometry(Surface {
            geometry: Sphere { center: Vec3f::new(0.3, -1.1, 0.45), radius: 0.7 },
            properties: SurfaceProperties::Material(0)
        });
        geo.add_geometry(Surface {
            geometry: Sphere { center: light_pos(), radius: radius() },
            properties: SurfaceProperties::Light(0)
        });
        CpuPathTracer {
            rng: StdRng::new().expect("cant create random generator"),
            camera: cam,
            geo: geo,
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
            let use_dl = true;
            let mut ray = self.camera.ray_from_screen(&sample);
            let mut path_length = 0;
            let mut path_weight = Vec3f::one();
            let mut color = Vec3f::zero();
            'current_path: loop {
                let isect = if let Some(isect) = self.geo.nearest_intersection(&ray) {
                    isect
                } else {
                    break 'current_path;
                };

                let mcol = match isect.surface {
                    SurfaceProperties::Material(id) => {
                        get_material(id).diffuse
                    },
                    SurfaceProperties::Light(id) => {
                        if !use_dl {
                            let lcol = get_light_color(id);
                            color = path_weight * lcol;
                        } else if path_length == 0 {
                            color = get_light_color(id);
                        }
                        break 'current_path;
                    }
                };

                if path_length > MAX_PATH_LENGTH || path_weight.sqnorm() < self.rng.next_f32() {
                    break 'current_path;
                }

                let hit_point = ray.dir * isect.dist + ray.orig;
                ray.dir = sample_brdf(&isect.normal, (self.rng.next_f32(), self.rng.next_f32()));
                ray.orig = hit_point + ray.dir * EPS_RAY;
                path_weight = path_weight * mcol;

                if use_dl {
                    let rnds = (self.rng.next_f32(), self.rng.next_f32());

                    let w = light_pos() - ray.orig;
                    let w2 = w.sqnorm();
                    let r2 = radius() * radius();
                    let cos_a_max = (1.0 - (r2 / w2).min(1.0)).sqrt();
                    let frac = 1.0 - cos_a_max;
                    let omega = 2.0 * PI * frac;
                    let le = get_light_color(0)/* / (4.0 * PI * r2) * FRAC_1_PI*/;
                    let ld = uniform_cone_sample(cos_a_max, rnds);
                    let w_basis = Frame::from_z(&w);
                    let ld = w_basis.to_world(&ld).normalize();
                    let cos_theta = isect.normal.dot(&ld).abs().max(0.0);
                    // let ldist = (/*ld * radius() + */light_pos() - ray.orig).norm();
                    let shadow_ray = Ray { orig: ray.orig, dir: ld };
                    // let lpnt = sample_light(rnds) + light_pos();
                    // let ld = lpnt - ray.orig;
                    // let nld = ld.normalize();
                    // let shadow_ray = Ray { orig: hit_point + ld * EPS_RAY, dir: ld };
                    let was_occluded = if let Some(is) = self.geo.nearest_intersection(&shadow_ray) {
                        match is.surface {
                            SurfaceProperties::Light(_id) => false,
                            _ => true
                        }
                    } else {
                        false
                    };
                    if !was_occluded {
                    // if !self.geo.was_occluded(&shadow_ray, ldist) {
                        color = color + le * path_weight * FRAC_1_PI * cos_theta * omega;
/*                        let l_to_pnt = light_pos() - ray.orig;
                        let term = (1.0 - radius() * radius() / ld.sqnorm()).sqrt().max(0.0).min(1.0);
                        let weight = 2.0 * (1.0 - term);
                        let cos_theta = nld.dot(&isect.normal);
                        if cos_theta > 0.0 {
                            color = color + path_weight * get_light_color(0) * di(&hit_point, &isect.normal, &lpnt, radius());
                        }*/
                    }
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
