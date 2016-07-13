#![allow(dead_code)]
use geometry::Ray;
use math::matrix_traits::*;
use math::vector_traits::*;
use math::{Mat4f, Rot3f, Vec2f, Vec2u, Vec3f, Vec4f};
use math;
use std::marker::PhantomData;
use framebuffer::{RgbFrameBuffer, YxyFrameBuffer};

#[derive(Clone, Debug)]
pub struct CameraBuilder<T: Camera> {
    pos: Vec3f,
    at: Vec3f,
    up: Vec3f,
    view_size: Vec2f,
    fov: f32,
    near: f32,
    far: f32,
    phantom: PhantomData<T>
}

#[derive(Clone, Copy)]
pub struct PerspectiveCamera {
    projection: PerspMat3<f32>,
    view_size: Vec2f,
    // translation: Mat4f,
    position: Vec3f,
    rotation: Rot3f,
    // world2raster: Mat4f,
    raster2world: Mat4f,
}

pub trait Camera {
    fn new(pos: Vec3f, at: Vec3f, up: Vec3f, view_size: Vec2f, fov: f32, near: f32, far: f32) -> Self;

    fn get_view_size(&self) -> Vec2f;

    fn build_rgb_framebuffer(&self) -> RgbFrameBuffer {
        let view_size = self.get_view_size();
        RgbFrameBuffer::new(Vec2u { x: view_size.x as usize, y: view_size.y as usize })
    }

    fn build_yxy_framebuffer(&self) -> YxyFrameBuffer {
        let view_size = self.get_view_size();
        YxyFrameBuffer::new(Vec2u { x: view_size.x as usize, y: view_size.y as usize })
    }
}

impl<T> CameraBuilder<T> where T: Camera {
    pub fn new() -> CameraBuilder<T> {
        CameraBuilder {
            pos: Vec3f::new(0.0, 0.0, 0.0),
            at: Vec3f::new(0.0, 0.0, -1.0),
            up: Vec3f::new(0.0, 1.0, 0.0),
            view_size: Vec2::new(800.0, 600.0),
            fov: 45.0,
            near: 0.1,
            far: 10000.0,
            phantom: PhantomData
        }
    }

    pub fn build(&self) -> T {
        Camera::new(self.pos, self.at, self.up, self.view_size, self.fov, self.near, self.far)
    }

    pub fn with_pos(&mut self, p: Vec3f) -> &mut CameraBuilder<T> {
        self.pos = p;
        self
    }

    pub fn with_look_at(&mut self, at: Vec3f) -> &mut CameraBuilder<T> {
        self.at = at;
        self
    }

    pub fn with_up(&mut self, up: Vec3f) -> &mut CameraBuilder<T> {
        self.up = up;
        self
    }

    pub fn with_view_size(&mut self, vs: Vec2u) -> &mut CameraBuilder<T> {
        self.view_size = Vec2::new(vs.x as f32, vs.y as f32);
        self
    }

    pub fn with_fov(&mut self, fov: f32) -> &mut CameraBuilder<T> {
        self.fov = fov;
        self
    }

    pub fn with_znear(&mut self, near: f32) -> &mut CameraBuilder<T> {
        self.near = near;
        self
    }

    pub fn with_zfar(&mut self, far: f32) -> &mut CameraBuilder<T> {
        self.far = far;
        self
    }
}

impl Camera for PerspectiveCamera {
    fn new(pos: Vec3f, at: Vec3f, up: Vec3f, view_size: Vec2f, fov: f32, near: f32, far: f32)
        -> PerspectiveCamera {
        let proj = PerspMat3::new(view_size.x / view_size.y, fov.to_radians(), near, far);
        let proj_mat = proj.to_mat().transpose();
        let transl: Mat4f = Mat4f::from_row(3, &math::vec3_to_4(&-pos, 1.0));
        let rot = Rot3::look_at_z(&at.normalize(), &-up.normalize());
        let world2cam = transl * math::mat3_to_4(&rot.submat());
        let world2screen = world2cam * proj_mat;
        let screen2world = world2screen.inv().expect("cant calc w2s inversion :(");
        let one_px_move = Mat4::from_row(3, &Vec4f::new(-1.0, -1.0, 0.0, 1.0));
        let raster2screen = Mat4f::from_diag(&Vec4f::new(2.0 / view_size.x, 2.0 / view_size.y, 0.0, 1.0))
            * one_px_move;
        let raster2world = raster2screen * screen2world;

        // let world2raster = world2screen * one_px_move
        //     * Mat4f::from_diag(&Vec4f::new(0.5 * view_size.x, 0.5 * view_size.y, 0.0, 1.0));

        PerspectiveCamera {
            projection: proj,
            position: pos,
            rotation: rot,
            raster2world: raster2world,
            // world2raster: world2raster,
            view_size: view_size,
        }
    }

    fn get_view_size(&self) -> Vec2f {
        self.view_size
    }
}

impl PerspectiveCamera {
    pub fn set_fov(&mut self, deg_angle: f32) -> &mut PerspectiveCamera {
        self.projection.set_fov(deg_angle.to_radians());
        self.recache_world_mat();
        self
    }

    pub fn set_aspect(&mut self, aspect: f32) -> &mut PerspectiveCamera {
        self.projection.set_aspect(aspect);
        self.recache_world_mat();
        self
    }

    pub fn set_view_dimensions(&mut self, width: u32, height: u32) -> &mut PerspectiveCamera {
        self.projection.set_aspect(width as f32 / height as f32);
        self.recache_world_mat();
        self
    }

    pub fn set_znear(&mut self, val: f32) -> &mut PerspectiveCamera {
        self.projection.set_znear(val);
        self.recache_world_mat();
        self
    }

    pub fn set_zfar(&mut self, val: f32) -> &mut PerspectiveCamera {
        self.projection.set_zfar(val);
        self.recache_world_mat();
        self
    }

    pub fn set_rotation(&mut self, rot: Vec3f) -> &mut PerspectiveCamera {
        self.rotation.set_rotation(rot);
        self.recache_world_mat();
        self
    }

    // pub fn set_look_at(&mut self, at: Vec3f, up: Vec3f) -> &mut PerspectiveCamera {
    //     self.rotation.look_at(at, up);
    //     self
    // }

    pub fn set_position(&mut self, pos: &Vec3f) -> &mut PerspectiveCamera {
        self.position = *pos;
        self.recache_world_mat();
        self
    }

    pub fn with_fov(mut self, deg_angle: i32) -> PerspectiveCamera {
        self.projection.set_fov((deg_angle as f64).to_radians() as f32);
        self
    }

    pub fn with_aspect(mut self, aspect: f32) -> PerspectiveCamera {
        self.projection.set_aspect(aspect);
        self
    }

    pub fn with_view_dimensions(mut self, width: u32, height: u32) -> PerspectiveCamera {
        self.projection.set_aspect(width as f32 / height as f32);
        self
    }

    pub fn with_znear(mut self, val: f32) -> PerspectiveCamera {
        self.projection.set_znear(val);
        self
    }

    pub fn with_zfar(mut self, val: f32) -> PerspectiveCamera {
        self.projection.set_zfar(val);
        self
    }

    pub fn with_rotation(mut self, rot: Vec3f) -> PerspectiveCamera {
        self.rotation.set_rotation(rot);
        self
    }

    // pub fn with_look_at(mut self, at: Vec3f, up: Vec3f) -> PerspectiveCamera {
    //     self.rotation.look_at(at, up);
    //     self
    // }

    pub fn with_position(mut self, pos: &Vec3f) -> PerspectiveCamera {
        self.set_position(pos);
        self
    }

    // pub fn get_world2raster_mat(&self) -> &Mat4f {
    //     &self.world2raster
    // }

    pub fn get_raster2world_mat(&self) -> &Mat4f {
        &self.raster2world
    }

    // pub fn apply_world2raster(&self, vec: &Vec3f) -> Vec3f {
    //     math::vec4_to_3(&(self.world2raster * math::vec3_to_4(vec, 1.0)))
    // }

    pub fn get_position(&self) -> Vec3f {
        self.position
    }

    pub fn apply_raster2world(&self, vec: &Vec3f) -> Vec3f {
        let v = math::vec3_to_4(&vec, 1.0) * self.raster2world;
        math::vec4_to_3(&v) / v.w
        // math::vec4_to_3(&v) * (1.0 / v.w)
    }

    pub fn ray_from_screen(&self, coord: &Vec2f) -> Ray {
        let pos = self.get_position();
        let world_raster = self.apply_raster2world(&Vec3f::new(coord.x, coord.y, 0.0));
        let dir = (world_raster - pos).normalize();
        Ray { orig: pos, dir: dir }
    }

    pub fn add_position(&mut self, pos: &Vec3f) {
        let new_pos = self.get_position() + *pos;
        self.set_position(&new_pos);
        self.recache_world_mat();
    }

    pub fn add_rotation(&mut self, rot: Vec3f) {
        self.rotation.prepend_rotation_mut(&rot);
        self.recache_world_mat();
    }

    fn recache_world_mat(&mut self) {
//        self.world2raster = self.compute_world_mat();
//        self.raster2world = self.world2raster.inv().expect("WTF?!");
    }

    // fn compute_world_mat(&self) -> Mat4f {
    //     self.projection.to_mat()
    //     * self.rotation.to_mat()
    //     * self.translation.to_mat()
    //     * self.scale.to_mat()
    // }
}

mod tests {
    #![cfg_attr(not(test), allow(unused_imports))]
    use super::{PerspectiveCamera, CameraBuilder};
    use math::{Vec2u, Vec3f, Vec2f};
    use geometry::Ray;
    use nalgebra::ApproxEq;

    fn test_camera() -> PerspectiveCamera {
        let res = Vec2u::new(800, 600);
        CameraBuilder::new()
            .with_view_size(res.clone())
            .with_pos(Vec3f::new(-0.0439815, -4.12529, 0.222539))
            .with_look_at(Vec3f::new(0.00688625, 0.998505, -0.0542161))
            .with_up(Vec3f::new(3.73896e-4, 0.0542148, 0.998529))
            .with_fov(45.0)
            .with_znear(0.1)
            .with_zfar(10000.0)
            .build()
    }

    #[test]
    fn ray_to_world_0_0() {
        let cam = test_camera();
        let Ray {orig, dir} = cam.ray_from_screen(&Vec2f::new(0 as f32, 0 as f32));
        println!("{:?} | {:?}", orig, dir);
        assert!(orig.approx_eq(&Vec3f::new(-0.0439815, -4.12529, 0.222539)));
        assert!(dir.approx_eq(&Vec3f { x: 0.4602826, y: 0.8370593, z: 0.29575595 }));
    }

    #[test]
    fn ray_to_world_15_19() {
        let cam = test_camera();
        let Ray {orig, dir} = cam.ray_from_screen(&Vec2f::new(15 as f32, 19 as f32));
        println!("{:?} | {:?}", orig, dir);
        assert!(orig.approx_eq(&Vec3f::new(-0.0439815, -4.12529, 0.222539)));
        assert!(dir.approx_eq(&Vec3f { x: 0.44990647, y: 0.8485973, z: 0.27832857 }));
    }

    #[test]
    fn ray_to_world_490_580() {
        let cam = test_camera();
        let Ray {orig, dir} = cam.ray_from_screen(&Vec2f::new(490 as f32, 580 as f32));
        println!("{:?} | {:?}", orig, dir);
        assert!(orig.approx_eq(&Vec3f::new(-0.0439815, -4.12529, 0.222539)));
        assert!(dir.approx_eq(&Vec3f { x: -0.108884126, y: 0.90651166, z: -0.407898 }));
    }

    #[test]
    fn ray_to_world_800_600() {
        let cam = test_camera();
        let Ray {orig, dir} = cam.ray_from_screen(&Vec2f::new(800 as f32, 600 as f32));
        println!("{:?} | {:?}", orig, dir);
        assert!(orig.approx_eq(&Vec3f::new(-0.0439815, -4.12529, 0.222539)));
        assert!(dir.approx_eq(&Vec3f { x: -0.44894803, y: 0.8063677, z: -0.3849893 }));
    }
}
