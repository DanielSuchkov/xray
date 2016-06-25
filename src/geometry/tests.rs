use super::*;
use math::Vec3f;
use scene::SurfaceProperties;

#[test]
fn occlusion_sphere() {
    let mut geos = GeometryList::new();
    let sphere = Sphere { center: Vec3f::new(0.0, 0.0, 0.0), radius: 2.0 };
    geos.add_geometry(Surface { geometry: sphere, properties: SurfaceProperties::Material(0) });
    let ray = Ray { orig: Vec3f::new(0.0, 0.0, -5.0), dir: Vec3f::new(0.0, 0.0, 1.0) };
    assert!(geos.was_occluded(&ray, 4.0));
    assert!(!geos.was_occluded(&ray, 3.0 - EPS_RAY_DF));
    assert!(geos.was_occluded(&ray, 3.0 + EPS_RAY_DF));
}

#[test]
fn occlusion_sphere_on_surface() {
    let mut geos = GeometryList::new();
    let sphere = Sphere { center: Vec3f::new(0.0, 0.0, 0.0), radius: 2.0 };
    geos.add_geometry(Surface { geometry: sphere, properties: SurfaceProperties::Material(0) });
    let ray = Ray { orig: Vec3f::new(0.0, 0.0, -6.0), dir: Vec3f::new(0.0, 0.0, 1.0) };
    assert!(!geos.was_occluded(&ray, EPS_RAY_DF));
}

#[test]
fn occlusion_sphere_near_surface() {
    let mut geos = GeometryList::new();
    let sphere = Sphere { center: Vec3f::new(0.0, 0.0, 0.0), radius: 2.0 };
    geos.add_geometry(Surface { geometry: sphere, properties: SurfaceProperties::Material(0) });
    let ray = Ray { orig: Vec3f::new(0.0, 0.0, -2.0), dir: Vec3f::new(0.0, 0.0, 1.0) };
    assert!(!geos.was_occluded(&ray.advance(-EPS_RAY_GEO * 2.0), EPS_RAY_GEO));
}

#[test]
fn occlusion_tri_sphere() {
    let mut geos = GeometryList::new();
    let ident_mat = SurfaceProperties::Material(0);
    let sphere = Sphere { center: Vec3f::new(0.0, 0.0, 0.0), radius: 2.0 };
    geos.add_geometry(Surface { geometry: sphere, properties: ident_mat });

    let tri = Triangle::new(Vec3f::new(1.0, -1.0, -3.0) , Vec3f::new(-1.0, -1.0, -3.0), Vec3f::new(-1.0, 1.0, -3.0));
    geos.add_geometry(Surface { geometry: tri, properties: ident_mat });

    let ray = Ray { orig: Vec3f::new(0.0, 0.0, 2.1), dir: Vec3f::new(0.0, 0.0, 1.0) };
    assert!(!geos.was_occluded(&ray, 0.01));

    let ray_from_tri = Ray { orig: Vec3f::new(0.0, 0.0, -3.5), dir: Vec3f::new(0.0, 0.0, 1.0) };
    assert!(geos.was_occluded(&ray_from_tri, 2.0));
}

 #[test]
fn occlusion_tri() {
    let mut geos = GeometryList::new();
    let ident_mat = SurfaceProperties::Material(0);

    let tri = Triangle::new(Vec3f::new(1.0, -1.0, -3.0) , Vec3f::new(-1.0, -1.0, -3.0), Vec3f::new(-1.0, 1.0, -3.0));
    geos.add_geometry(Surface { geometry: tri, properties: ident_mat });

    let ray = Ray { orig: Vec3f::new(0.0, 0.0, 2.1), dir: Vec3f::new(0.0, 0.0, 1.0) };
    let was_occluded = if let Some(isect) = geos.nearest_intersection(&ray) {
        println!("{:?}", isect.dist);
        if isect.dist < 1.0 {
            true
        } else {
            false
        }
    } else {
        false
    };

    assert!(!was_occluded);

    let ray_from_tri = Ray { orig: Vec3f::new(0.0, 0.0, -3.5), dir: Vec3f::new(0.0, 0.0, 1.0) };
    assert!(geos.was_occluded(&ray_from_tri, 2.0));
}
