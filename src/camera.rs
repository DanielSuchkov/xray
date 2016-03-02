#![allow(dead_code)]

use math;
use math::{Vec3f, Mat4f, Vec2f, Vec2u, FloatExt};
use math::matrix_traits::*;
use math::vector_traits::*;
use transform::{Translation, Rotation, Scale};
use std::marker::PhantomData;

#[derive(Clone, Debug)]
pub struct CameraBuilder<T: Camera> {
    pos: Vec3f,
    at: Vec3f,
    up: Vec3f,
    view_size: Vec2u,
    fov: f32,
    near: f32,
    far: f32,
    phantom: PhantomData<T>
}

#[derive(Clone, Copy)]
pub struct PerspectiveCamera {
    projection: PerspMat3<f32>,
    translation: Translation,
    rotation: Rotation,
    scale: Scale,
    world2screen: Mat4f,
    screen2world: Mat4f,
}

pub trait Camera {
    fn new(pos: Vec3f, at: Vec3f, up: Vec3f, view_size: Vec2u, fov: f32, near: f32, far: f32) -> Self;
}

impl<T> CameraBuilder<T> where T: Camera {
    pub fn new() -> CameraBuilder<T> {
        CameraBuilder {
            pos: Vec3f::new(0.0, 0.0, 0.0),
            at: Vec3f::new(0.0, 0.0, -1.0),
            up: Vec3f::new(0.0, 1.0, 0.0),
            view_size: Vec2u::new(800, 600),
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
        self.view_size = vs;
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
    fn new(pos: Vec3f, at: Vec3f, up: Vec3f, view_size: Vec2u, fov: f32, near: f32, far: f32)
        -> PerspectiveCamera {
        let proj = PerspMat3::new(1.0, fov.to_radian(), -near, -far);
        let proj_m = proj.to_mat().transpose() * -1.0;
        let transl = Translation::new(pos);
        let mut transl_m = Mat4f::new_identity(4);
        transl_m.set_row(3, math::extend_vec3_to_4(&-pos, 1.0));
        let rot = Rotation::look_at_z(-at.normalize(), up.normalize());
        let world2cam = transl_m * rot.to_mat();
        let world2screen = world2cam * proj_m;
        let screen2world = world2screen.inv().expect("i cant do inversion");
        let raster_mul = Scale::new(Vec3f::new(2.0 / view_size.x as f32, 2.0 / view_size.y as f32, 0.0)).to_mat()
            * Translation::new(Vec3f::new(-1.0, -1.0, 0.0)).to_mat();
        let raster2world = raster_mul * screen2world;

        let world2raster = world2screen
            * Translation::new(Vec3f::new(-1.0, -1.0, 0.0)).to_mat()
            * Scale::new(Vec3f::new(0.5 * view_size.x as f32, 0.5 * view_size.y as f32, 0.0)).to_mat();

        PerspectiveCamera {
            projection: proj,
            translation: transl,
            rotation: rot,
            screen2world: raster2world,
            world2screen: world2raster,
            scale: Scale::new_identity(),
        }
    }
}

impl PerspectiveCamera {
    pub fn set_fov(&mut self, deg_angle: i32) -> &mut PerspectiveCamera {
        self.projection.set_fov((deg_angle as f64).to_radians() as f32);
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

    pub fn set_position(&mut self, pos: Vec3f) -> &mut PerspectiveCamera {
        self.translation.set_translation(pos);
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

    pub fn with_position(mut self, pos: Vec3f) -> PerspectiveCamera {
        self.translation.set_translation(pos);
        self
    }

    pub fn get_world2screen_mat(&self) -> &Mat4f {
        &self.world2screen
    }

    pub fn get_screen2world_mat(&self) -> &Mat4f {
        &self.screen2world
    }

    pub fn apply_world2screen(&self, vec: &Vec3f) -> Vec3f {
        math::shrink_vec4_to_3(&(self.world2screen * math::extend_vec3_to_4(vec, 1.0)))
    }

    pub fn apply_screen2world(&self, vec: &Vec3f) -> Vec3f {
        // math::shrink_vec4_to_3(&(math::extend_vec3_to_4(vec, 1.0)))
        let mut w = self.screen2world[(3,3)];
        for c in 0..3 {
            w += self.screen2world[(c, 3)] * vec[c];
        }
        let inv_w = 1.0 / w;
        let mut res = Vec3f::new(0.0, 0.0, 0.0);
        for r in 0..3 {
            res[r] = self.screen2world[(3, r)];
            for c in 0..3 {
                res[r] += vec[c] * self.screen2world[(c, r)];
            }
            res[r] *= inv_w;
        }
        res
    }

    pub fn ray_from_screen(&self, coord: &Vec2f) -> (Vec3f, Vec3f) {
        let pos = self.translation.get_translation();
        let world_raster = self.apply_screen2world(&Vec3f::new(coord.x, coord.y, 0.0));
        let dir = (world_raster - pos).normalize();
        (pos, dir)
    }

    pub fn add_position(&mut self, pos: Vec3f) {
        self.translation.add_translation(pos);
        self.recache_world_mat();
    }

    pub fn add_rotation(&mut self, rot: Vec3f) {
        self.rotation.add_rotation(rot);
        self.recache_world_mat();
    }

    fn recache_world_mat(&mut self) {
//        self.world2screen = self.compute_world_mat();
//        self.screen2world = self.world2screen.inv().expect("WTF?!");
    }

//     fn compute_world_mat(&self) -> Mat4f {
// //        self.projection.to_mat() * self.rotation.to_mat() * self.translation.to_mat() * self.scale.to_mat()
//     }
}
