use nalgebra::{Vector3, Point3, Scalar};
use rand::{thread_rng, Rng};
use yaml_rust::yaml::Array;
use yaml_rust::Yaml;

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
        return None;
    } else {
        return Some(eta * angle_of_incidence + (eta * cosi - k.sqrt()) * n);
    }
}


pub fn yaml_array_into_point3(array: &Yaml) -> Point3<f64>
{
    Point3::new(
        array[0].as_f64().unwrap(),
        array[1].as_f64().unwrap(),
        array[2].as_f64().unwrap(),
    )
}

pub fn yaml_array_into_vector3(array: &Yaml) -> Vector3<f64>
{
    Vector3::new(
        array[0].as_f64().unwrap(),
        array[1].as_f64().unwrap(),
        array[2].as_f64().unwrap(),
    )
}

pub fn yaml_into_u32(yaml: &Yaml) -> u32
{
    yaml.as_i64().unwrap() as u32
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

        println!("{}", ratio);

        assert!(ratio > 0.03999999);
        assert!(ratio < 0.04000001);
    }

    #[test]
    fn test_fresnel_ratio_one_at_perpendicular_of_normal() {
        let normal = Vector3::new(0.0, 0.0, -1.0); // normal is straight forward
        let angle_of_incidence = Vector3::new(1.0, 0.0, 0.0); // impact is 90 deg at normal
        let ior = 1.5; // Test ior

        let ratio = get_fresnel_ratio(normal, angle_of_incidence, ior);

        println!("{}", ratio);
        assert!(ratio > 0.99999999);
        assert!(ratio < 1.00000001);
    }

    #[test]
    fn test_fresnel_ratio_half_at_around_82_deg_on_normal() {
        let normal = Vector3::new(0.0, 0.0, -1.0); // normal is straight forward
        let angle_of_incidence = Vector3::new(1.0, 0.0, 0.1259879).normalize(); // impact is 82 at normal
        let ior = 1.5; // Test ior

        let ratio = get_fresnel_ratio(normal, angle_of_incidence, ior);

        println!("{}", ratio);
        assert!(ratio > 0.499999);
        assert!(ratio < 0.500001);
    }
}
