#![allow(dead_code)]
use camera::PerspectiveCamera;
use framebuffer::FrameBuffer;
use geometry::GeometryManager;
use math::vector_traits::*;
use math::{Vec2u, Vec2f, Vec3f, Zero, vec3_from_value};
use rand::{Rng, thread_rng};
use scene::{Scene, SurfaceProperties};

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

pub struct EyeLight<S: Scene> {
    camera: PerspectiveCamera,
    scene: S,
    frame: FrameBuffer,
}

impl<S> CpuRender for EyeLight<S> where S: Scene {
    fn trace_from_screen(&self, sample: Vec2f) -> Vec3f {
        let ray = self.camera.ray_from_screen(&sample);

        if let Some(ref isect) = self.scene.nearest_intersection(&ray) {
            let l_dot_n = isect.normal.dot(&-ray.dir);
            if let SurfaceProperties::Material(mat_id) = isect.surface {
                use geometry::Ray;
                use math::Vec3f;
                let hit_point = ray.orig + ray.dir * isect.dist;
                if !self.scene.was_occluded(&Ray{orig: hit_point, dir: -ray.dir}, isect.dist) {
                    self.scene.get_material(mat_id).diffuse * l_dot_n.abs()
                } else {
                    Vec3f::zero()
                }
            } else {
                Vec3f::zero()
            }
        } else {
            vec3_from_value(0.5)
        }
    }

    fn get_mut_framebuffer(&mut self) -> &mut FrameBuffer {
        &mut self.frame
    }

    fn get_view_size(&self) -> Vec2f {
        self.camera.get_view_size()
    }
}

impl<S> Render<S> for EyeLight<S> where S: Scene {
    fn new(cam: PerspectiveCamera, scene: S) -> EyeLight<S> {
        let resolution = cam.get_view_size();
        let resolution = Vec2u::new(resolution.x as usize, resolution.y as usize);
        EyeLight {
            camera: cam,
            scene: scene,
            frame: FrameBuffer::new(resolution),
        }
    }

    fn iterate(&mut self, iter_nb: usize) {
        self.iterate_over_screen(iter_nb)
    }

    fn get_framebuffer(&self) -> &FrameBuffer {
        &self.frame
    }
}
