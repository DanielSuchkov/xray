#![allow(dead_code)]

use math;
use math::{Vec3f, Mat4f};
use math::matrix_traits::*;
use math::vector_traits::*;
use nalgebra::Rot3;
use nalgebra::Rotation as RotationTrait;

#[derive(Copy, Clone)]
pub struct Transform {
    pub s: Scale,
    pub r: Rotation,
    pub t: Translation,
}

impl Transform {
    pub fn new() -> Transform {
        Transform {
            s: Scale::new_identity(),
            r: Rotation::new(),
            t: Translation::new_identity(),
        }
    }

    pub fn with_rotation(mut self, rot_axis_angle: Vec3f) -> Transform {
        self.r.set_rotation(rot_axis_angle);
        self
    }

    pub fn with_translation(mut self, t: Vec3f) -> Transform {
        self.t.set_translation(t);
        self
    }

    pub fn with_scale(mut self, scale_factor: Vec3f) -> Transform {
        self.s.set_scale(scale_factor);
        self
    }

    pub fn set_rotation(&mut self, rot_axis_angle: Vec3f) -> &mut Transform {
        self.r.set_rotation(rot_axis_angle);
        self
    }

    pub fn set_translation(&mut self, t: Vec3f) -> &mut Transform {
        self.t.set_translation(t);
        self
    }

    pub fn set_scale(&mut self, scale_factor: Vec3f) -> &mut Transform {
        self.s.set_scale(scale_factor);
        self
    }

    pub fn to_mat(&self) -> Mat4f {
        self.t.to_mat() * self.r.to_mat() * self.s.to_mat()
    }

    pub fn to_array(&self) -> [[f32; 4]; 4] {
        self.to_mat().as_ref().clone()
    }
}

#[derive(Copy, Clone)]
pub struct Scale {
    scale: Mat4f,
}

impl Scale {
    pub fn new_identity() -> Scale {
        Scale {
            scale: Mat4::new_identity(4)
        }
    }

    pub fn new(scale: Vec3f) -> Scale {
        let mut out = Scale::new_identity();
        out.set_scale(scale);
        out
    }

    pub fn set_scale(&mut self, s: Vec3f) -> &mut Scale {
        self.scale.m11 = s.x;
        self.scale.m22 = s.y;
        self.scale.m33 = s.z;
        self
    }

    pub fn get_scale(&self) -> Vec3f {
        Vec3::new(self.scale.m11, self.scale.m22, self.scale.m33)
    }

    pub fn as_mat(&self) -> &Mat4f {
        &self.scale
    }

    pub fn to_mat(&self) -> Mat4f {
        self.scale
    }
}

#[derive(Copy, Clone)]
pub struct Rotation {
    rot: Rot3<f32>
}

impl Rotation {
    pub fn new() -> Rotation {
        Rotation {
            rot: Rot3::from_diag(&Vec3::new(1.0, 1.0, 1.0))
        }
    }

    pub fn look_at_z(at: Vec3f, up: Vec3f) -> Rotation {
        Rotation {
            rot: Rot3::look_at_z(&at, &up)
        }
    }

    pub fn look_at(at: Vec3f, up: Vec3f) -> Rotation {
        Rotation {
            rot: Rot3::look_at(&at, &up)
        }
    }

    pub fn set_rotation(&mut self, rot_axis_angle: Vec3f) {
        self.rot.set_rotation(rot_axis_angle);
    }

    pub fn add_rotation(&mut self, rot_axis_angle: Vec3f) {
        self.rot.prepend_rotation_mut(&rot_axis_angle);
    }

    pub fn to_mat(&self) -> Mat4f {
        math::extend_mat3_to_4(&self.rot.submat())
    }

    // pub fn look_at(&mut self, at: Vec3f, up: Vec3f) {
    //     self.rot.look_at_z(&at, &up);
    // }
}

#[derive(Copy, Clone)]
pub struct Translation {
    mat: Mat4f
}

impl Translation {
    pub fn new_identity() -> Translation {
        Translation {
            mat: Mat4::new_identity(4)
        }
    }

    pub fn new(pos: Vec3f) -> Translation {
        Translation::new_identity().with_position(pos)
    }

    pub fn with_position(&self, pos: Vec3f) -> Translation {
        let mut out = *self;
        out.set_translation(pos);
        out
    }

    pub fn set_translation(&mut self, pos: Vec3f) {
        self.mat.set_row(3, math::extend_vec3_to_4(&pos, 1.0));
    }

    pub fn add_translation(&mut self, pos: Vec3f) {
        let cur_transl = self.get_translation();
        self.mat.set_row(3, math::extend_vec3_to_4(&(pos + cur_transl), 1.0));
    }

    pub fn get_translation(&self) -> Vec3f {
        math::shrink_vec4_to_3(&self.mat.row(3))
    }

    pub fn to_mat(&self) -> Mat4f {
        self.mat
    }
}
