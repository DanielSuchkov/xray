use std;
use std::ops::{Div, Mul};

pub use nalgebra::{BaseNum};

pub type Mat4f = Mat4<f32>;
pub type Vec3f = Vec3<f32>;
pub type Vec2f = Vec2<f32>;
pub type Vec2u = Vec2<usize>;

pub mod vector_traits {
    pub use nalgebra::{Vec2, Vec3, Vec4, FloatVec, Absolute, BaseFloat, Dot, Norm, Cross};

    pub trait VectorExtra<T>: FloatVec<T> where T: BaseFloat {
        fn reflect(self, normal: Self) -> Self;
    }

    impl<T, S> VectorExtra<S> for T
        where T: FloatVec<S>,
              S: From<f32> + BaseFloat {
        fn reflect(self, normal: T) -> T {
            let scale: S = <S as From<f32>>::from(2.0) * self.dot(&normal).abs();
            (self + normal * scale).normalize()
        }
    }
}

pub mod matrix_traits {
    pub use nalgebra::{Mat3, Mat4, Inv, Eye, PerspMat3, Transpose, Row, Col, Diag};
}

use self::vector_traits::*;
use self::matrix_traits::*;

pub trait FloatExt {
    fn to_radian(self) -> Self;
}

impl<T> FloatExt for T where T: Div<Output=T> + Mul<Output=T> + From<f32> {
    fn to_radian(self) -> T {
        self * From::from(std::f32::consts::PI / 180.0)
    }
}

pub fn mix(a: f32, b: f32, mix: f32) -> f32 {
    b * mix + a * (1.0 - mix)
}

pub fn vec3_from_value<T: BaseFloat>(val: T) -> Vec3<T> {
    Vec3::new(val, val, val)
}

pub fn extend_mat3_to_4<N: BaseNum>(m: &Mat3<N>) -> Mat4<N> {
    Mat4 {
        m11: m.m11,     m12: m.m12,     m13: m.m13,     m14: N::zero(),
        m21: m.m21,     m22: m.m22,     m23: m.m23,     m24: N::zero(),
        m31: m.m31,     m32: m.m32,     m33: m.m33,     m34: N::zero(),
        m41: N::zero(), m42: N::zero(), m43: N::zero(), m44: N::one(),
    }
}

pub fn extend_vec3_to_4<N: BaseNum>(v: &Vec3<N>, aditional: N) -> Vec4<N> {
    Vec4::new(v.x, v.y, v.z, aditional)
}

pub fn shrink_vec4_to_3<N: BaseNum>(v: &Vec4<N>) -> Vec3<N> {
    Vec3::new(v.x, v.y, v.z)
}
