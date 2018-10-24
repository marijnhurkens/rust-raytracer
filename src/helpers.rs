use cgmath::*;
use rand::{thread_rng, Rng};

pub fn vector_reflect(vec: Vector3<f64>, normal: Vector3<f64>) -> Vector3<f64> {
    vec - 2.0 * vec.dot(normal) * normal
}

pub fn get_random_in_unit_sphere() -> Vector3<f64> {
    let mut rng = thread_rng();

    let mut vec: Vector3<f64>;

    while {
        vec = 2.0 * Vector3::new(rng.gen::<f64>(), rng.gen::<f64>(), rng.gen::<f64>())
            - Vector3::new(1.0, 1.0, 1.0);

        vec.dot(vec) >= 1.0
    } {}

    vec
}

pub fn get_fresnel_ratio(normal: Vector3<f64>, angle_of_incidence: Vector3<f64>, ior: f64) -> f64 {
    let fresnel_ratio: f64;

    let mut i_dot_n = angle_of_incidence.dot(normal);

    // clamp
    if i_dot_n > 1.0 {
        i_dot_n = 1.0;
    }
    if i_dot_n < -1.0 {
        i_dot_n = -1.0;
    }

    let mut eta_i = 1.0;
    let mut eta_t = ior;

    // If dot product of angle and normal is < 0 we are outside of the
    // object, otherwise inside
    if i_dot_n > 0.0 {
        eta_i = eta_t;
        eta_t = 1.0;
    }

    let sin_t = eta_i / eta_t * (1.0 - i_dot_n * i_dot_n).max(0.0).sqrt();

    if sin_t > 1.0 {
        //Total internal reflection
        fresnel_ratio = 1.0;
    } else {
        let cos_t = (1.0 - sin_t * sin_t).max(0.0).sqrt();
        let cos_i = cos_t.abs();
        let r_s = ((eta_t * cos_i) - (eta_i * cos_t)) / ((eta_t * cos_i) + (eta_i * cos_t));
        let r_p = ((eta_i * cos_i) - (eta_t * cos_t)) / ((eta_i * cos_i) + (eta_t * cos_t));
        fresnel_ratio = (r_s * r_s + r_p * r_p) / 2.0;
    }

    // let mut  r0 = (1.0 - ior) / (1.0 + ior);
    // r0 = r0 * r0;

    // let mut cosX = -normal.dot(angle_of_incidence);
    // if(1.0 > ior) {
    //     let n = 1.0/ior;
    //     let sinT2 = n * n * (1.0 - cosX * cosX);

    //     if (sinT2 > 1.0) {
    //         return 1.0;
    //     }

    //     cosX = (1.0 - sinT2).sqrt();
    // }
    // let x = 1.0 - cosX;
    // fresnel_ratio = r0 + (1.0 - r0) * x * x * x * x * x;

    fresnel_ratio
}

pub fn get_refract_ray(
    normal: Vector3<f64>,
    angle_of_incidence: Vector3<f64>,
    ior: f64,
) -> Option<Vector3<f64>> {
    // float cosi = clamp(-1, 1, dotProduct(I, N));
    // float etai = 1, etat = ior;
    // Vec3f n = N;
    // if (cosi < 0) { cosi = -cosi; } else { std::swap(etai, etat); n= -N; }
    // float eta = etai / etat;
    // float k = 1 - eta * eta * (1 - cosi * cosi);
    // return k < 0 ? 0 : eta * I + (eta * cosi - sqrtf(k)) * n;

    let mut cosi = angle_of_incidence.dot(normal);

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
        n = -n;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fresnel_ratio_zero_at_impact_on_normal() {
        let normal = Vector3::new(0.0, 0.0, -1.0); // normal is straight forward
        let angle_of_incidence = Vector3::new(0.0, 0.0, 1.0); // impact is reverse of normal
        let ior = 0.66666666666; // Test ior

        let ratio = get_fresnel_ratio(normal, angle_of_incidence, ior);

        println!("{}", ratio);

        assert!(ratio > 0.03999999);
        assert!(ratio < 0.04000001);
    }

    #[test]
    fn test_fresnel_ratio_one_at_perpendicular_of_normal() {
        let normal = Vector3::new(0.0, 0.0, -1.0); // normal is straight forward
        let angle_of_incidence = Vector3::new(1.0, 0.0, 0.0); // impact is 90 deg at normal
        let ior = 0.6666666; // Test ior

        let ratio = get_fresnel_ratio(normal, angle_of_incidence, ior);

        println!("{}", ratio);
        assert!(ratio > 0.99999999);
        assert!(ratio < 1.00000001);
    }

    #[test]
    fn test_fresnel_ratio_half_at_around_80_deg_on_normal() {
        let normal = Vector3::new(0.0, 0.0, -1.0); // normal is straight forward
        let angle_of_incidence = Vector3::new(1.00, 0.0, 0.38); // impact is 90 deg at normal
        let ior = 1.5; // Test ior

        let ratio = get_fresnel_ratio(normal, angle_of_incidence, ior);

        println!("{}", ratio);
        assert!(ratio > 0.499999);
        assert!(ratio < 0.500001);
    }
}
