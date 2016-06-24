#![allow(dead_code)]
use math::Vec3f;
use brdf::Material;

pub const DAYLIGHT_COLOR: Vec3f = Vec3f { x: 0.65, y: 0.6, z: 0.45 };
pub const EVENING_COLOR: Vec3f = Vec3f { x: 0.65, y: 0.55, z: 0.35 };
pub const GREEN_COLOR: Vec3f = Vec3f { x: 0.156863, y: 0.803922, z: 0.172549 };
pub const RED_COLOR: Vec3f = Vec3f { x: 0.803922, y: 0.152941, z: 0.172549 };
pub const MARGENTA_COLOR: Vec3f = Vec3f { x: 0.8, y: 0.2, z: 0.6 };
pub const GOLDEN_COLOR: Vec3f = Vec3f { x: 1.0, y: 0.7, z: 0.0 };
pub const SKY_BLUE_COLOR: Vec3f = Vec3f { x: 0.1, y: 0.9, z: 0.9 };

pub const WHITE_DIFFUSE: Material = Material {
    diffuse: Vec3f { x: 0.99, y: 0.99, z: 0.99 },
    specular: Vec3f { x: 0.0, y: 0.0, z: 0.0 },
    phong_exp: 1.0
};

pub const GREEN_DIFFUSE: Material = Material {
    diffuse: GREEN_COLOR,
    specular: Vec3f { x: 0.0, y: 0.0, z: 0.0 },
    phong_exp: 1.0
};

pub const RED_DIFFUSE: Material = Material {
    diffuse: RED_COLOR,
    specular: Vec3f { x: 0.0, y: 0.0, z: 0.0 },
    phong_exp: 1.0
};

pub const SKY_BLUE_DIFFUSE: Material = Material {
    diffuse: SKY_BLUE_COLOR,
    specular: Vec3f { x: 0.0, y: 0.0, z: 0.0 },
    phong_exp: 1.0
};

pub const BLUE_DIFFUSE: Material = Material {
    diffuse: Vec3f { x: 0.2, y: 0.2, z: 0.8 },
    specular: Vec3f { x: 0.0, y: 0.0, z: 0.0 },
    phong_exp: 1.0
};

pub const MARGENTA_DIFFUSE: Material = Material {
    diffuse: MARGENTA_COLOR,
    specular: Vec3f { x: 0.0, y: 0.0, z: 0.0 },
    phong_exp: 1.0
};

pub const DARK_MIRROR: Material = Material {
    diffuse: Vec3f { x: 0.0, y: 0.0, z: 0.0 }, // Vec3f::new(0.5, 0.5, 0.2) * 0.7,
    specular: Vec3f { x: 0.50, y: 0.50, z: 0.50 }, // Vec3f::new(0.5, 0.5, 0.2) * 0.3,
    phong_exp: 1000.0
};

pub const GOLDEN_SPEC: Material = Material {
    diffuse: Vec3f { x: 0.0, y: 0.0, z: 0.0 },
    specular: GOLDEN_COLOR,
    phong_exp: 10.0
};

pub const GOLDEN_MIRROR: Material = Material {
    diffuse: Vec3f { x: 0.5, y: 0.35, z: 0.15 },
    specular: GOLDEN_COLOR,
    phong_exp: 1000.0
};

pub const WHITE_CERAMICS: Material = Material {
    diffuse: Vec3f { x: 0.99, y: 0.99, z: 0.99 },
    specular: Vec3f { x: 0.5, y: 0.5, z: 0.5 },
    phong_exp: 1000.0
};

pub const MIRROR: Material = Material {
    diffuse: Vec3f { x: 0.0, y: 0.0, z: 0.0 },
    specular: Vec3f { x: 0.99, y: 0.99, z: 0.99 },
    phong_exp: 10000.0
};

pub const SKY_BLUE_MIRROR: Material = Material {
    diffuse: Vec3f { x: 0.05, y: 0.45, z: 0.45 },
    specular: SKY_BLUE_COLOR,
    phong_exp: 10000.0
};
