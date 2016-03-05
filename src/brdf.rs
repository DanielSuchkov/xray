#![allow(dead_code)]
use math::{Vec3f, Vec2f, Zero, EPS_COSINE, EPS_PHONG};
use math::vector_traits::*;
use std::f32::consts::{FRAC_1_PI, PI};
use geometry::{Frame, Ray};
use std::ops::Add;

#[derive(Debug, Clone, PartialEq, Copy)]
pub struct Material {
    pub diffuse: Vec3f,
    pub specular: Vec3f,
    pub phong_exp: f32
}

#[derive(Debug, Clone)]
pub struct Brdf {
    material: Material,
    frame: Frame,
    local_dir_fix: Vec3f,
    prob: Probabilities,
}

#[derive(Debug, Clone)]
pub struct BrdfEval {
    pub radiance: Vec3f,
    pub dir_pdf_w: f32,
}

#[derive(Debug, Clone)]
pub struct Sample {
    pub factor: Vec3f,
    pub dir: Vec3f,
    pub pdf_w: f32
}

#[derive(Debug, Clone)]
struct Probabilities {
    diffuse: f32,
    phong: f32,
    continuation: f32,
}

impl Brdf {
    pub fn new(mat: Material, frame: Frame, ray: &Ray) -> Option<Brdf> {
        let local_dir = frame.to_local(&-ray.dir);
        let prob = Probabilities::new(&mat);
        if local_dir.z.abs() < EPS_COSINE || prob.continuation == 0.0 {
            None
        } else {
            Some(Brdf { material: mat, local_dir_fix: local_dir, frame: frame, prob: prob })
        }
    }

    pub fn evaluate(&self, world_dir_gen: &Vec3f) -> Option<(BrdfEval, f32)> {
        let local_dir_gen = self.frame.to_local(world_dir_gen);
        if local_dir_gen.z * self.local_dir_fix.z < 0.0 {
            None
        } else {
            let diffuse = self.evaluate_diffuse(&local_dir_gen);
            let phong = self.evaluate_phong(&local_dir_gen);
            Some((diffuse + phong, local_dir_gen.z.abs()))
        }
    }

    pub fn sample(&self, rands: (f32, f32, f32)) -> Option<(Sample, f32)> {
        let mut sample = if rands.2 < self.prob.diffuse {
            self.sample_diffuse((rands.0, rands.1)).map(|mut diff_sample| {
                let phong_eval = self.evaluate_phong(&diff_sample.dir);
                diff_sample.factor = diff_sample.factor + phong_eval.radiance;
                diff_sample.pdf_w += phong_eval.dir_pdf_w;
                diff_sample
            })
        } else {
            self.sample_phong((rands.0, rands.1)).map(|mut phong_sample| {
                let diff_eval = self.evaluate_diffuse(&phong_sample.dir);
                phong_sample.factor = phong_sample.factor + diff_eval.radiance;
                phong_sample.pdf_w += diff_eval.dir_pdf_w;
                phong_sample
            })
        };

        match sample {
            Some(ref mut sample) if sample.dir.z.abs() >= EPS_COSINE => {
                let cos_theta = sample.dir.z.abs();
                sample.dir = self.frame.to_world(&sample.dir);
                Some((sample.clone(), cos_theta))
            },
            _ => None
        }
    }

    pub fn continuation_prob(&self) -> f32 {
        self.prob.continuation
    }

    fn evaluate_diffuse(&self, local_dir_gen: &Vec3f) -> BrdfEval {
        if self.prob.diffuse == 0.0 {
            Zero::zero()
        } else {
            BrdfEval {
                radiance: self.material.diffuse * FRAC_1_PI,
                dir_pdf_w: self.prob.diffuse * (local_dir_gen.z * FRAC_1_PI).max(0.0)
            }
        }
    }

    fn evaluate_phong(&self, local_dir_gen: &Vec3f) -> BrdfEval {
        if self.prob.phong == 0.0 || self.local_dir_fix.z < EPS_COSINE || local_dir_gen.z < EPS_COSINE {
            Zero::zero()
        } else {
            let local_refl_fix = self.local_dir_fix.reflect_local();
            let dot_refl_wi = local_refl_fix.dot(local_dir_gen);
            if dot_refl_wi <= EPS_PHONG {
                Zero::zero()
            } else {
                let pdf_w = cos_hemisphere_pow_pdf_w(&local_refl_fix, local_dir_gen, self.material.phong_exp);
                let rho = self.material.specular * (self.material.phong_exp + 2.0) * 0.5 * FRAC_1_PI;
                BrdfEval {
                    radiance: rho * dot_refl_wi.powf(self.material.phong_exp),
                    dir_pdf_w: pdf_w * self.prob.phong
                }
            }
        }
    }

    fn sample_diffuse(&self, rands: (f32, f32)) -> Option<Sample> {
        if self.local_dir_fix.z < EPS_COSINE {
            None
        } else {
            let (local_dir, pdf_w) = cos_hemisphere_sample_w(rands);
            Some(Sample {
                factor: self.material.diffuse * FRAC_1_PI,
                dir: local_dir,
                pdf_w: pdf_w * self.prob.diffuse
            })
        }
    }

    fn sample_phong(&self, rands: (f32, f32)) -> Option<Sample> {
        let (local_dir_gen, _) = cos_hemisphere_pow_sample_w(self.material.phong_exp, rands);
        let local_refl_fix = self.local_dir_fix.reflect_local();
        let local_dir_gen = {
            let frame = Frame::from_z(local_refl_fix);
            frame.to_world(&local_dir_gen)
        };
        let dot_refl_wi = local_refl_fix.dot(&local_dir_gen);
        if dot_refl_wi <= EPS_PHONG {
            None
        } else {
            Some(Sample {
                dir: local_dir_gen,
                pdf_w: self.pdf_w_phong(&local_dir_gen),
                factor: self.material.specular * (self.material.phong_exp + 2.0) * 0.5 * FRAC_1_PI
            })
        }
    }

    fn pdf_w_phong(&self, local_dir: &Vec3f) -> f32 {
        if self.prob.phong == 0.0 {
            0.0
        } else {
            let local_refl_fix = self.local_dir_fix.reflect_local();
            let dot_refl_wi = local_refl_fix.dot(local_dir);
            if dot_refl_wi <= EPS_COSINE {
                0.0
            } else {
                let pdf_w = cos_hemisphere_pow_pdf_w(&local_refl_fix, local_dir, self.material.phong_exp);
                pdf_w * self.prob.phong
            }
        }
    }
}

impl Material {
    pub fn new_identity() -> Material {
        Material {
            diffuse: Zero::zero(),
            specular: Zero::zero(),
            phong_exp: 0.0
        }
    }

    fn albedo_diffuse(&self) -> f32 {
        luminance(&self.diffuse)
    }

    fn albedo_specular(&self) -> f32 {
        luminance(&self.specular)
    }

    fn total_albedo(&self) -> f32 {
        self.albedo_specular() + self.albedo_diffuse()
    }
}

impl Add for BrdfEval {
    type Output = BrdfEval;
    fn add(self, rhs: BrdfEval) -> BrdfEval {
        BrdfEval {
            radiance: self.radiance + rhs.radiance,
            dir_pdf_w: self.dir_pdf_w + rhs.dir_pdf_w
        }
    }
}

impl Zero for BrdfEval {
    fn zero() -> BrdfEval {
        BrdfEval {
            radiance: Zero::zero(),
            dir_pdf_w: Zero::zero(),
        }
    }

    fn is_zero(&self) -> bool {
        self.radiance.is_zero()
    }
}

impl Probabilities {
    fn new(mat: &Material) -> Probabilities {
        let albedo_diffuse = mat.albedo_diffuse();
        let albedo_specular = mat.albedo_specular();
        let total_albedo = mat.total_albedo();
        if total_albedo < 1.0e-9 {
            Probabilities {
                diffuse: 0.0,
                phong: 0.0,
                continuation: 0.0
            }
        } else {
            Probabilities {
                diffuse: albedo_diffuse / total_albedo,
                phong: albedo_specular / total_albedo,
                continuation: total_albedo.min(1.0)
            }
        }
    }
}

fn luminance(a_rgb: &Vec3f) -> f32 {
    0.212671 * a_rgb.x + 0.715160 * a_rgb.y + 0.072169 * a_rgb.z
}

// returns vector and pdf
pub fn cos_hemisphere_sample_w(rnd: (f32, f32)) -> (Vec3f, f32) {
    let phi = rnd.0 * 2.0 * PI;
    let costheta = rnd.1.sqrt();
    let sintheta = (1.0 - costheta * costheta).sqrt();

    let ret = Vec3f::new(sintheta * phi.cos(), sintheta * phi.sin(), costheta);
    (ret, ret.z * FRAC_1_PI)
}

// returns vector and pdf
pub fn cos_hemisphere_pow_sample_w(phong_exp: f32, rnd: (f32, f32)) -> (Vec3f, f32) {
    let phi = rnd.0 * 2.0 * PI;
    let costheta = rnd.1.powf(1.0 / (phong_exp + 1.0));
    let sintheta = (1.0 - costheta * costheta).sqrt();

    let ret = Vec3f::new(sintheta * phi.cos(), sintheta * phi.sin(), costheta);
    (ret, (phong_exp + 1.0) * costheta.powf(phong_exp) * (0.5 * FRAC_1_PI))
}

pub fn uniform_triangle_sample(rnd: (f32, f32)) -> Vec2f {
    let term = rnd.0.sqrt();
    Vec2f::new(1.0 - term, rnd.1 * term)
}

pub fn uniform_sphere_sample_w(rnd: (f32, f32)) -> (Vec3f, f32) {
    let phi = rnd.0 * 2.0 * PI;
    let term2 = 2.0 * (rnd.1 - rnd.1 * rnd.1).sqrt();

    (Vec3f::new(phi.cos() * term2, phi.sin() * term2, 1.0 - 2.0 * rnd.1), uniform_sphere_pdf_w())
}

pub fn cos_hemisphere_pow_pdf_w(normal: &Vec3f, dir: &Vec3f, phong_exp: f32) -> f32 {
    let cos_theta = normal.dot(dir).max(0.0);
    (phong_exp + 1.0) * cos_theta.powf(phong_exp) * (0.5 * FRAC_1_PI)
}

pub fn uniform_sphere_pdf_w() -> f32 {
    0.25 * FRAC_1_PI
}

pub fn pdf_w_to_a(pdf_w: f32, dist: f32, cos_there: f32) -> f32 {
    pdf_w * cos_there.abs() / (dist * dist)
}

pub fn pdf_a_to_w(pdf_a: f32, dist: f32, cos_there: f32) -> f32 {
    pdf_a * (dist * dist) / cos_there.abs()
}
