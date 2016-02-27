use std;
use std::ops::{Div, Mul};
use cgmath::Vector3;

pub use cgmath::{EuclideanVector, Vector};

pub type Vec3f = Vector3<f32>;

pub trait FloatExtra {
    fn to_radian(self) -> Self;
}

impl<T> FloatExtra for T where T: Div<Output=T> + Mul<Output=T> + From<f32> {
    fn to_radian(self) -> T {
        self * From::from(std::f32::consts::PI) / From::from(180.0)
    }
}

pub fn mix(a: f32, b: f32, mix: f32) -> f32 {
    b * mix + a * (1.0 - mix)
}
