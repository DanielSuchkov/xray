#![allow(dead_code)]
use math::{Vec3f};
use std::f32::consts::{PI};

pub fn luminance(a_rgb: &Vec3f) -> f32 {
    // a_rgb.x + a_rgb.y + a_rgb.z
    0.212671 * a_rgb.x + 0.715160 * a_rgb.y + 0.072169 * a_rgb.z
}

pub fn cos_hemisphere_sample_w(rnd: (f32, f32)) -> Vec3f { // -> (Vec3f, f32) {
    let phi = rnd.0 * 2.0 * PI;
    let cos_theta = rnd.1.sqrt();
    let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

    let ret = Vec3f::new(sin_theta * phi.cos(), sin_theta * phi.sin(), cos_theta);
    ret
    // (ret, ret.z * FRAC_1_PI)
}

pub fn uniform_hemisphere_sample(rnd: (f32, f32)) -> Vec3f {
    let phi = rnd.0 * 2.0 * PI;
    let cos_2theta = rnd.1;
    let sin_2theta = 2.0 * (rnd.1 - rnd.1 * rnd.1).sqrt();
    Vec3f {
        x: 2.0 * phi.cos() * sin_2theta, y: 2.0 * phi.sin() * sin_2theta, z: cos_2theta
    }
}

pub fn uniform_sphere_sample(rnd: (f32, f32)) -> Vec3f {
    // rnd.1 - sin^2(theta)
    let phi = rnd.0 * 2.0 * PI;
    let cos_2theta = 1.0 - 2.0 * rnd.1;
    let sin_2theta = 2.0 * (rnd.1 - rnd.1 * rnd.1).sqrt();
    Vec3f {
        x: 2.0 * phi.cos() * sin_2theta, y: 2.0 * phi.sin() * sin_2theta, z: cos_2theta
    }
}

pub fn uniform_cone_sample(cos_a_max: f32, rnd: (f32, f32)) -> Vec3f {
    let cos_a = 1.0 - rnd.0 * (1.0 - cos_a_max);
    let sin_a = (1.0 - cos_a * cos_a).sqrt();
    let phi = 2.0 * PI * rnd.1;
    Vec3f {
        x: phi.cos() * sin_a, y: phi.sin() * sin_a, z: cos_a
    }
}

pub fn pow_cos_hemisphere_sample_w(n: f32, rnd: (f32, f32)) -> Vec3f {
    let phi = rnd.0 * 2.0 * PI;
    let cos_theta = rnd.1.powf(1.0 / (n + 1.0));
    let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
    Vec3f {
        x: phi.cos() * sin_theta, y: phi.sin() * sin_theta, z: cos_theta
    }
}
