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
    let mut fresnel_ratio: f64;
    let normal_normalized = normal.normalize();
    let angle_of_incidence_normalized = angle_of_incidence.normalize();

    // FROM POVRAY, n = ior
    // double sqrg = Sqr(n) + Sqr(cosTi) - 1.0;

    // if(sqrg <= 0.0)
    //     // Total reflection.
    //     return 1.0;

    // double g = sqrt(sqrg);

    // double quot1 = (g - cosTi) / (g + cosTi);
    // double quot2 = (cosTi * (g + cosTi) - 1.0) / (cosTi * (g - cosTi) + 1.0);

    // double f = 0.5 * Sqr(quot1) * (1.0 + Sqr(quot2));

    // return clip(f, 0.0, 1.0);

     let cos_theta_incidence = normal.dot(angle_of_incidence).abs();
     let sqrg = ior * ior + cos_theta_incidence * cos_theta_incidence - 1.0;

     if sqrg <= 0.0 {
         fresnel_ratio = 1.0;
     } else {
         let g = sqrg.sqrt();
         let quot1 = (g - cos_theta_incidence) / (g + cos_theta_incidence);
         let quot2 = (cos_theta_incidence * (g + cos_theta_incidence) - 1.0) / (cos_theta_incidence * (g - cos_theta_incidence) + 1.0);

         let mut f = 0.5 * (quot1 * quot1) * (1.0 + (quot2 * quot2));

         if f > 1.0 {
             f = 1.0;
         }

         if f < 0.0 {
             f = 0.0;
         }

         fresnel_ratio = f;

     }

     //println!("Ratio {}", fresnel_ratio);



    // let mut n1 = 1.0;
    // let mut n2 = ior;

    // let cos_theta_incidence = normal_normalized.dot(angle_of_incidence_normalized);
    // let sin_theta_incidence = (1.0 - cos_theta_incidence * cos_theta_incidence).sqrt();

    // if cos_theta_incidence > 0.0 {
    //     n1 = ior;
    //     n2 = 1.0;
    // }

    // let sin_theta_transmittance = n1 * (sin_theta_incidence / n2);
    // let cos_theta_transmittance = (1.0 - sin_theta_transmittance * sin_theta_transmittance).sqrt();

    // let n1_cos_theta_transmittance = n1 * cos_theta_transmittance;
    // let n2_cos_theta_transmittance = n2 * cos_theta_transmittance;
    // let n1_cos_theta_incidence = n1 * cos_theta_incidence;
    // let n2_cos_theta_incidence = n2 * cos_theta_incidence;

    // let s_polarized_sqrt = (n1_cos_theta_incidence - n2_cos_theta_transmittance)
    //     / (n1_cos_theta_incidence + n2_cos_theta_transmittance);
    // let s_polarized = s_polarized_sqrt * s_polarized_sqrt;

    // let p_polarized_sqrt = (n2_cos_theta_incidence - n1_cos_theta_transmittance)
    //     / (n2_cos_theta_incidence + n1_cos_theta_transmittance);
    // let p_polarized = p_polarized_sqrt * p_polarized_sqrt;

    // fresnel_ratio = (s_polarized + p_polarized) / 2.0;

    /// version 1
    // let i_dot_n = angle_of_incidence.dot(normal.clone());
    // let mut eta_i = 1.0;
    // let mut eta_t = ior;
    // if i_dot_n > 0.0 {
    //     eta_i = eta_t;
    //     eta_t = 1.0;
    // }

    // let sin_t = eta_i / eta_t * (1.0 - i_dot_n * i_dot_n).max(0.0).sqrt();
    // if sin_t > 1.0 {
    //     //Total internal reflection
    //     fresnel_ratio = 1.0;
    // } else {
    //     let cos_t = (1.0 - sin_t * sin_t).max(0.0).sqrt();
    //     let cos_i = cos_t.abs();
    //     let r_s = ((eta_t * cos_i) - (eta_i * cos_t)) / ((eta_t * cos_i) + (eta_i * cos_t));
    //     let r_p = ((eta_i * cos_i) - (eta_t * cos_t)) / ((eta_i * cos_i) + (eta_t * cos_t));
    //     fresnel_ratio =  (r_s * r_s + r_p * r_p) / 2.0;
    //              println!("var: {}, ratio: {}", r_s, fresnel_ratio);

    // }

    // let max = 100.0;

    // if fresnel_ratio > max {
    //     fresnel_ratio = max;
    // }

    // fresnel_ratio /= max;

    /// version 2
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
        n = -normal;
    }

    let eta = etai / etat;
    let k = 1.0 - eta * eta * (1.0 - cosi * cosi);
    if k < 0.0 {
        // no refraction
        //println!("No refrac");
        return None;
    } else {
        return Some(eta * angle_of_incidence + (eta * cosi - k.sqrt()) * n);
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
    fn test_fresnel_ratio_half_at_around_80_deg_on_normal() {
        let normal = Vector3::new(0.0, 0.0, -1.0); // normal is straight forward
        let angle_of_incidence = Vector3::new(1.0, 0.0, 0.2); // impact is 80 at normal
        let ior = 1.5; // Test ior

        let ratio = get_fresnel_ratio(normal, angle_of_incidence, ior);

        println!("{}", ratio);
        assert!(ratio > 0.499999);
        assert!(ratio < 0.500001);
    }
}
