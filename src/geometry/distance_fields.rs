#[allow(unused_imports)]
use math::{smin_exp, smin_poly, smin_pow};
use math::Vec3f;
use scene::SurfaceProperties;

pub trait DField {
    fn dist(&self, point: &Vec3f) -> f32;

    fn grad(&self, p: &Vec3f, delta: f32) -> Vec3f {
        let p = *p;
        let dx = Vec3f::new(delta, 0.0, 0.0);
        let dy = Vec3f::new(0.0, delta, 0.0);
        let dz = Vec3f::new(0.0, 0.0, delta);
        let dfdx = self.dist(&(p + dx)) - self.dist(&(p - dx));
        let dfdy = self.dist(&(p + dy)) - self.dist(&(p - dy));
        let dfdz = self.dist(&(p + dz)) - self.dist(&(p - dz));
        Vec3f { x: dfdx, y: dfdy, z: dfdz } / (2.0 * delta)
    }
}

pub trait Isosurface {
    fn dist(&self, point: &Vec3f) -> f32;
    fn grad(&self, p: &Vec3f, delta: f32) -> Vec3f;
    fn surface_properties(&self) -> SurfaceProperties;
}

pub struct DFieldsSubstr<A, B>
    where A: DField, B: DField {
    pub a: A,
    pub b: B,
    pub pos: Vec3f,
}

pub struct DFieldsUnion<A, B>
    where A: DField, B: DField {
    pub a: A,
    pub b: B,
    pub pos: Vec3f,
}

pub struct DFieldsBlend<A, B>
    where A: DField, B: DField {
    pub a: A,
    pub b: B,
    pub k: f32,
    pub pos: Vec3f,
}

#[derive(Debug, Clone)]
pub struct DFieldIsosurface<D: DField> {
    pub dfield: D,
    pub properties: SurfaceProperties
}

pub struct DFieldDisplace<D, F>
    where D: DField,
          F: Fn(&Vec3f) -> f32 {
    pub a: D,
    pub disp: F
}


impl<A, B> DField for DFieldsSubstr<A, B>
    where A: DField, B: DField {
    fn dist(&self, point: &Vec3f) -> f32 {
        let point = *point - self.pos;
        self.a.dist(&point).max(-self.b.dist(&point))
    }
}

impl<A, B> DField for DFieldsUnion<A, B>
    where A: DField, B: DField {
    fn dist(&self, point: &Vec3f) -> f32 {
        let point = *point - self.pos;
        self.a.dist(&point).min(self.b.dist(&point))
    }
}

impl<A, B> DField for DFieldsBlend<A, B>
    where A: DField, B: DField {
    fn dist(&self, point: &Vec3f) -> f32 {
        let point = *point - self.pos;
        smin_poly(self.a.dist(&point), self.b.dist(&point), self.k)
    }
}

impl<D, F> DField for DFieldDisplace<D, F>
    where D: DField,
          F: Fn(&Vec3f) -> f32 {
    fn dist(&self, point: &Vec3f) -> f32 {
        let d1 = self.a.dist(point);
        let d2 = (self.disp)(point);
        d1 + d2
    }
}

impl<D> Isosurface for DFieldIsosurface<D> where D: DField {
    fn dist(&self, point: &Vec3f) -> f32 {
        self.dfield.dist(point)
    }

    fn grad(&self, p: &Vec3f, delta: f32) -> Vec3f {
        self.dfield.grad(p, delta)
    }

    fn surface_properties(&self) -> SurfaceProperties {
        self.properties
    }
}

