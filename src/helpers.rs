use std::f64::consts::{FRAC_PI_2, FRAC_PI_4, PI};
use std::ops::Mul;

use nalgebra::indexing::MatrixIndex;
use nalgebra::{ArrayStorage, ClosedSub, Point2, Point3, Scalar, Vector2, Vector3, U1, U3};
use rand::{rng, Rng};
use yaml_rust::Yaml;

#[derive(Debug)]
pub struct Bounds<T: Copy + Scalar + ClosedSub + Mul> {
    pub p_min: Point2<T>,
    pub p_max: Point2<T>,
}

impl<T: Copy + Scalar + ClosedSub + Mul<Output = T>> Bounds<T> {
    pub fn area(&self) -> T {
        let area_vector = &self.vector();

        area_vector.x * area_vector.y
    }

    pub fn vector(&self) -> Vector2<T> {
        self.p_max - self.p_min
    }
}

pub fn vector_reflect(vec: Vector3<f64>, normal: Vector3<f64>) -> Vector3<f64> {
    -vec + 2.0 * vec.dot(&normal) * normal
}

pub fn get_random_in_unit_sphere() -> Vector3<f64> {
    let mut rng = rng();

    let mut vec: Vector3<f64>;

    while {
        vec =
            2.0 * Vector3::new(
                rng.random::<f64>(),
                rng.random::<f64>(),
                rng.random::<f64>(),
            ) - Vector3::new(1.0, 1.0, 1.0);

        vec.dot(&vec) >= 1.0
    } {}

    vec
}

pub fn offset_ray_origin(p: Point3<f64>, normal: Vector3<f64>, w: Vector3<f64>) -> Point3<f64> {
    let mut origin_offset = (normal.abs().dot(&Vector3::repeat(1e-4))) * normal;

    if w.dot(&normal) < 0.0 {
        origin_offset = -origin_offset
    }

    p + origin_offset
}

pub fn uniform_sample_triangle(sample: Vec<f64>) -> Point2<f64> {
    let point = Point2::from_slice(&sample);
    let su0 = point.x.sqrt();

    Point2::new(1.0 - su0, point.y * su0)
}

pub fn power_heuristic(nf: i32, f_pdf: f64, ng: i32, g_pdf: f64) -> f64 {
    let f = nf as f64 * f_pdf;
    let g = ng as f64 * g_pdf;
    (f * f) / (f * f + g * g)
}

pub fn coordinate_system(v1: Vector3<f64>) -> (Vector3<f64>, Vector3<f64>, Vector3<f64>) {
    let v2 = if v1.x.abs() > v1.y.abs() {
        Vector3::new(-v1.z, 0.0, v1.x) / (v1.x * v1.x + v1.z * v1.z).sqrt()
    } else {
        Vector3::new(0.0, v1.z, -v1.y) / (v1.y * v1.y + v1.z * v1.z).sqrt()
    };

    let v3 = v1.cross(&v2);

    (v1, v2, v3)
}

pub fn gamma(n: f64) -> f64 {
    (n * f64::EPSILON) / (1.0 - n * f64::EPSILON)
}

pub fn face_forward(n: Vector3<f64>, v: Vector3<f64>) -> Vector3<f64> {
    if n.dot(&v) < 0.0 {
        return -n;
    }

    n
}

pub fn get_fresnel_ratio(normal: Vector3<f64>, angle_of_incidence: Vector3<f64>, ior: f64) -> f64 {
    let fresnel_ratio: f64;

    // FROM POVRAY
    // https://github.com/POV-Ray/povray/blob/master/source/core/render/trace.cpp#L2668

    let cos_theta_incidence = normal.dot(&angle_of_incidence).abs();
    let sqrg = ior * ior + cos_theta_incidence * cos_theta_incidence - 1.0;

    if sqrg <= 0.0 {
        fresnel_ratio = 1.0;
    } else {
        let g = sqrg.sqrt();
        let quot1 = (g - cos_theta_incidence) / (g + cos_theta_incidence);
        let quot2 = (cos_theta_incidence * (g + cos_theta_incidence) - 1.0)
            / (cos_theta_incidence * (g - cos_theta_incidence) + 1.0);

        let mut f = 0.5 * (quot1 * quot1) * (1.0 + (quot2 * quot2));

        if f > 1.0 {
            f = 1.0;
        }

        if f < 0.0 {
            f = 0.0;
        }

        fresnel_ratio = f;
    }

    fresnel_ratio
}

pub fn refract(
    angle_of_incidence: Vector3<f64>,
    normal: Vector3<f64>,
    ior: f64,
) -> Option<Vector3<f64>> {
    let mut cos_theta_i = normal.dot(&angle_of_incidence);

    let sin_2_theta_i = 0.0f64.max(1.0 - cos_theta_i * cos_theta_i);
    let sin_2_theta_t = ior * ior * sin_2_theta_i;

    if sin_2_theta_t > 1.0 {
        return None;
    }

    let cos_theta_t = (1.0 - sin_2_theta_t).sqrt();

    Some(ior * -angle_of_incidence + (ior * cos_theta_i - cos_theta_t) * normal)
    // Float sin2ThetaI = std::max(0.f, 1.f - cosThetaI * cosThetaI);
    // Float sin2ThetaT = eta * eta * sin2ThetaI;
    // <<Handle total internal reflection for transmission>>
    // Float cosThetaT = std::sqrt(1 - sin2ThetaT);
    //
    // *wt = eta * -wi + (eta * cosThetaI - cosThetaT) * Vector3f(n);
}

pub fn yaml_array_into_point2(array: &Yaml) -> Point2<u32> {
    Point2::new(
        array[0].as_i64().unwrap() as u32,
        array[1].as_i64().unwrap() as u32,
    )
}

pub fn yaml_array_into_point3(array: &Yaml) -> Point3<f64> {
    Point3::new(
        array[0].as_f64().unwrap(),
        array[1].as_f64().unwrap(),
        array[2].as_f64().unwrap(),
    )
}

pub fn yaml_array_into_vector3(array: &Yaml) -> Vector3<f64> {
    Vector3::new(
        array[0].as_f64().unwrap(),
        array[1].as_f64().unwrap(),
        array[2].as_f64().unwrap(),
    )
}

pub fn yaml_into_u32(yaml: &Yaml) -> u32 {
    yaml.as_i64().unwrap() as u32
}

pub fn max_dimension_vec_3<T: Scalar + PartialOrd>(v: Vector3<T>) -> usize {
    if v.x > v.y {
        if v.x > v.z {
            0
        } else {
            2
        }
    } else if v.y > v.z {
        1
    } else {
        2
    }
}

pub fn permute<T: Copy>(v: Vector3<T>, x: usize, y: usize, z: usize) -> Vector3<T> {
    Vector3::new(v[x], v[y], v[z])
}

pub fn concentric_sample_disk() -> Point2<f64> {
    let mut rng = rng();

    let u_offset =
        Point2::new(rng.random::<f64>(), rng.random::<f64>()) * 2.0 - Vector2::new(1.0, 1.0);

    if u_offset.x == 0.0 && u_offset.y == 0.0 {
        return Point2::new(0.0, 0.0);
    }

    let (theta, r);
    if u_offset.x.abs() > u_offset.y.abs() {
        r = u_offset.x;
        theta = FRAC_PI_4 * (u_offset.y / u_offset.x);
    } else {
        r = u_offset.y;
        theta = FRAC_PI_2 - FRAC_PI_4 * (u_offset.x / u_offset.y);
    }

    r * Point2::new(theta.cos(), theta.sin())
}

pub fn spherical_direction(sin_theta: f64, cos_theta: f64, phi: f64) -> Vector3<f64> {
    Vector3::new(sin_theta * phi.cos(), sin_theta * phi.sin(), cos_theta)
}

pub fn spherical_theta(v: Vector3<f64>) -> f64 {
    v.y.clamp(-1.0, 1.0).acos()
}

pub fn spherical_phi(v: Vector3<f64>) -> f64 {
    let p = v.x.atan2(v.z);

    if p < 0.0 {
        p + 2.0 * PI
    } else {
        p
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fresnel_ratio_low_at_impact_on_normal() {
        let normal = Vector3::new(0.0, 0.0, -1.0); // normal is straight forward
        let angle_of_incidence = Vector3::new(0.0, 0.0, 1.0); // impact is reverse of normal
        let ior = 1.5; // Test ior

        let ratio = get_fresnel_ratio(normal, angle_of_incidence, ior);

        assert!(ratio > 0.03999999);
        assert!(ratio < 0.04000001);
    }

    #[test]
    fn test_fresnel_ratio_one_at_perpendicular_of_normal() {
        let normal = Vector3::new(0.0, 0.0, -1.0); // normal is straight forward
        let angle_of_incidence = Vector3::new(1.0, 0.0, 0.0); // impact is 90 deg at normal
        let ior = 1.5; // Test ior

        let ratio = get_fresnel_ratio(normal, angle_of_incidence, ior);

        assert!(ratio > 0.99999999);
        assert!(ratio < 1.00000001);
    }

    #[test]
    fn test_fresnel_ratio_half_at_around_82_deg_on_normal() {
        let normal = Vector3::new(0.0, 0.0, -1.0); // normal is straight forward
        let angle_of_incidence = Vector3::new(1.0, 0.0, 0.1259879).normalize(); // impact is 82 at normal
        let ior = 1.5; // Test ior

        let ratio = get_fresnel_ratio(normal, angle_of_incidence, ior);

        assert!(ratio > 0.499999);
        assert!(ratio < 0.500001);
    }

    #[test]
    fn test_max_dimension_vec_3() {
        let vec = Vector3::new(1, 3, 2);
        let res = max_dimension_vec_3(vec);
        assert_eq!(1, res);
    }

    #[test]
    fn test_permute() {
        let input = Vector3::new(1, 2, 3);

        let res = permute(input, 2, 1, 0);

        assert_eq!(Vector3::new(3, 2, 1), res);
    }

    #[test]
    fn test_spherical_phi() {
        let v1 = Vector3::new(0.0, 0.0, 1.0);
        assert!((spherical_phi(v1) - 0.0).abs() < 1e-6);

        let v2 = Vector3::new(1.0, 0.0, 0.0);
        assert!((spherical_phi(v2) - std::f64::consts::FRAC_PI_2).abs() < 1e-6);

        let v3 = Vector3::new(0.0, 0.0, -1.0);
        assert!((spherical_phi(v3) - std::f64::consts::PI).abs() < 1e-6);

        let v4 = Vector3::new(-1.0, 0.0, 0.0);
        assert!((spherical_phi(v4) - 3.0 * std::f64::consts::FRAC_PI_2).abs() < 1e-6);
    }

    #[test]
    fn test_spherical_theta() {
        let v1 = Vector3::new(0.0, 1.0, 0.0); // Up (+Y) -> theta = 0
        assert!((spherical_theta(v1) - 0.0).abs() < 1e-6);

        let v2 = Vector3::new(1.0, 0.0, 0.0); // Horizon -> theta = PI/2
        assert!((spherical_theta(v2) - std::f64::consts::FRAC_PI_2).abs() < 1e-6);

        let v3 = Vector3::new(0.0, -1.0, 0.0); // Down (-Y) -> theta = PI
        assert!((spherical_theta(v3) - std::f64::consts::PI).abs() < 1e-6);
    }
}
