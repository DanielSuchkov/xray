use render::{Render, CpuStRender};
use scene::{Scene, SurfaceProperties};
use camera::{Camera, PerspectiveCamera};
use framebuffer::FrameBuffer;
use math::{Vec2f, Vec3f, vec3_from_value, Zero};
use math::vector_traits::*;

pub struct EyeLight<S: Scene> {
    camera: PerspectiveCamera,
    scene: S,
}

impl<S> CpuStRender for EyeLight<S> where S: Scene {
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

    fn get_view_size(&self) -> Vec2f {
        self.camera.get_view_size()
    }
}

impl<S> Render<S> for EyeLight<S> where S: Scene {
    fn new(cam: PerspectiveCamera, scene: S) -> EyeLight<S> {
        EyeLight {
            camera: cam,
            scene: scene,
        }
    }

    fn iterate(&self, iter_nb: usize, frame: &mut FrameBuffer) {
        self.iterate_over_screen(iter_nb, frame)
    }
}
