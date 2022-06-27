use std::f64::consts::{FRAC_PI_2, FRAC_PI_4};
use std::ops::Mul;

use nalgebra::{ArrayStorage, ClosedSub, Point2, Point3, Scalar, U1, U3, Vector2, Vector3};
use nalgebra::indexing::MatrixIndex;
use rand::{thread_rng, Rng};
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
    vec - 2.0 * vec.dot(&normal) * normal
}

pub fn get_random_in_unit_sphere() -> Vector3<f64> {
    let mut rng = thread_rng();

    let mut vec: Vector3<f64>;

    while {
        vec = 2.0 * Vector3::new(rng.gen::<f64>(), rng.gen::<f64>(), rng.gen::<f64>())
            - Vector3::new(1.0, 1.0, 1.0);

        vec.dot(&vec) >= 1.0
    } {}

    vec
}

// todo: add sampler input
pub fn uniform_sample_triangle() -> Point2<f64> {
    let mut rng = thread_rng();

    let point = Point2::new(rng.gen::<f64>(), rng.gen::<f64>());
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

pub fn get_refract_ray(
    normal: Vector3<f64>,
    angle_of_incidence: Vector3<f64>,
    ior: f64,
) -> Option<Vector3<f64>> {
    let mut cosi = angle_of_incidence.dot(&normal);

    // clamp
    if cosi > 1.0 {
        cosi = 1.0
    }

    if cosi < -1.0 {
        cosi = -1.0;
    }

    let mut etai = 1.0;
    let mut etat = ior;
    let mut n = normal;

    if cosi < 0.0 {
        cosi = -cosi;
    } else {
        etai = ior;
        etat = 1.0;
        n = -normal;
    }

    let eta = etai / etat;
    let k = 1.0 - eta * eta * (1.0 - cosi * cosi);
    if k < 0.0 {
        // no refraction
        None
    } else {
        Some(eta * angle_of_incidence + (eta * cosi - k.sqrt()) * n)
    }
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

pub fn permute< T: Copy>(v: Vector3<T>, x: usize, y: usize, z: usize) -> Vector3<T>
{
    Vector3::new(v[x], v[y], v[z])
}

pub fn concentric_sample_disk() -> Point2<f64> {
    let mut rng = thread_rng();

    let u_offset = Point2::new(rng.gen::<f64>(), rng.gen::<f64>()) * 2.0 - Vector2::new(1.0, 1.0);

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
        let vec = Vector3::new(1,3,2);
        let res = max_dimension_vec_3(vec);
        assert_eq!(1, res);
    }

    #[test]
    fn test_permute() {
        let input = Vector3::new(1, 2, 3);

        let res = permute(input, 2, 1, 0);

        assert_eq!(Vector3::new(3, 2, 1), res);
    }
}
