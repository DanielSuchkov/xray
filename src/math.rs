pub use nalgebra::BaseNum;
pub use num::traits::{Zero, One};

pub const EPS_COSINE: f32 = 1e-6;

pub mod vector_traits {
    pub use nalgebra::{Absolute, BaseFloat, Cross, Dot, FloatVec, Norm, Vec2, Vec3, Vec4};

    pub trait VectorExtra<T>: FloatVec<T> where T: BaseFloat {
        fn reflect_local(&self) -> Self;
        fn reflect_global(&self, normal: &Self) -> Self;
        fn fold<F>(&self, f: F) -> T where F: Fn(T, T) -> T;
        fn map<F>(&self, f: F) -> Self where F: Fn(T) -> T;
        fn zip<F>(&self, other: &Self, f: F) -> Self where F: Fn(T, T) -> T;
    }

    impl<T> VectorExtra<T> for Vec3<T>
        where T: From<f32> + BaseFloat,
              Vec3<T>: FloatVec<T> {
        fn reflect_local(&self) -> Vec3<T> {
            Vec3::new(-self.x, -self.y, self.z)
        }

        fn reflect_global(&self, normal: &Self) -> Self {
            let scale = <T as From<f32>>::from(2.0) * self.dot(normal).abs();
            (*self + normal.clone() * scale).normalize()
        }

        fn fold<F>(&self, f: F) -> T where F: Fn(T, T) -> T {
            f(f(self.x, self.y), self.z)
        }

        fn map<F>(&self, f: F) -> Self where F: Fn(T) -> T {
            Vec3::new(f(self.x), f(self.y), f(self.z))
        }

        fn zip<F>(&self, other: &Self, f: F) -> Self where F: Fn(T, T) -> T {
            Vec3::new(f(self.x, other.x), f(self.y, other.y), f(self.z, other.z))
        }
    }
}

pub mod matrix_traits {
    pub use nalgebra::{
        Col, Eye, Inv, Mat3, Mat4, PerspMat3, Rot3,
        Rotation, Row, Transpose, Diag, Mat
    };

    use num::One;
    use std::ops::Mul;

    pub trait MatrixExtra<N, R, C>: Mat<N, R, C>
        where C: Mul<Self, Output = R> {
        fn from_row(nrow: usize, row: &R) -> Self;
    }

    impl<T, R, C, N> MatrixExtra<N, R, C> for T
        where T: Mat<N, R, C> + One,
              R: Clone,
              C: Mul<T, Output=R> {
        fn from_row(nrow: usize, row: &R) -> T {
            let mut new = T::one();
            new.set_row(nrow, row.clone());
            new
        }
    }
}

use self::vector_traits::*;
use self::matrix_traits::*;

pub type Mat4f = Mat4<f32>;
pub type Mat3f = Mat3<f32>;
pub type Rot3f = Rot3<f32>;
pub type Vec4f = Vec4<f32>;
pub type Vec3f = Vec3<f32>;
pub type Vec3d = Vec3<f64>;
pub type Vec2f = Vec2<f32>;
pub type Vec2u = Vec2<usize>;

pub fn mix(a: f32, b: f32, mix: f32) -> f32 {
    b * mix + a * (1.0 - mix)
}

pub fn clamp(x: f32, min: f32, max: f32) -> f32 {
    x.max(min).min(max)
}

pub fn vec3_from_value<T: BaseFloat>(val: T) -> Vec3<T> {
    Vec3::new(val, val, val)
}

pub fn mat3_to_4<N: BaseNum>(m: &Mat3<N>) -> Mat4<N> {
    Mat4 {
        m11: m.m11,     m12: m.m12,     m13: m.m13,     m14: N::zero(),
        m21: m.m21,     m22: m.m22,     m23: m.m23,     m24: N::zero(),
        m31: m.m31,     m32: m.m32,     m33: m.m33,     m34: N::zero(),
        m41: N::zero(), m42: N::zero(), m43: N::zero(), m44: N::one(),
    }
}

pub fn vec3_to_4<N: BaseNum>(v: &Vec3<N>, aditional: N) -> Vec4<N> {
    Vec4::new(v.x, v.y, v.z, aditional)
}

pub fn vec4_to_3<N: BaseNum>(v: &Vec4<N>) -> Vec3<N> {
    Vec3::new(v.x, v.y, v.z)
}

pub fn ortho(v: &Vec3f) -> Vec3f {
    // http://lolengine.net/blog/2013/09/21/picking-orthogonal-vector-combing-coconuts
    if v.x.abs() > v.z.abs() {
        Vec3f::new(-v.y, v.x, 0.0)
    } else {
        Vec3f::new(0.0, -v.z, v.y)
    }
}

pub fn smin_exp(a: f32, b: f32, k: f32 ) -> f32 {
    let res = (-k*a).exp() + (-k*b).exp();
    -res.ln() / k
}

pub fn smin_pow(a: f32, b: f32, k: f32) -> f32 {
    let a = a.powf(k);
    let b = b.powf(k);
    ((a * b) / (a + b)).powf(1.0 / k)
}

pub fn smin_poly(a: f32, b: f32, k: f32) -> f32 {
    let h = clamp(0.5 + 0.5 * (b - a) / k, 0.0, 1.0);
    mix(b, a, h) - k * h * (1.0 - h)
}

pub fn triple_sin(p: &Vec3f) -> f32 {
    p.map(|x| (1.0 * x).sin()).fold(|x, y| x * y)
    // 1.00 * (1.00 * p.x).sin() * (1.00 * p.y).sin() * (1.00 * p.z).sin()
}
