use render::{Render, CpuRender};
use scene::{Scene, SurfaceProperties};
use camera::PerspectiveCamera;
use framebuffer::FrameBuffer;
use math::{Vec2f, Vec3f, Vec2u, vec3_from_value, Zero};
use math::vector_traits::*;

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
