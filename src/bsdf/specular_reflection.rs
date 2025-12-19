use nalgebra::{Point2, Point3, Vector3};

use crate::bsdf::helpers::fresnel::{Fresnel, FresnelDielectric, FresnelTrait};
use crate::bsdf::helpers::{abs_cos_theta, cos_theta};
use crate::bsdf::{BXDFtrait, BXDFTYPES};

#[derive(Debug, Clone, Copy)]
pub struct SpecularReflection {
    reflectance_color: Vector3<f64>,
    fresnel: Fresnel,
}

impl SpecularReflection {
    pub fn new(reflectance_color: Vector3<f64>, fresnel: Fresnel) -> Self {
        SpecularReflection {
            reflectance_color,
            fresnel,
        }
    }
}

impl BXDFtrait for SpecularReflection {
    fn get_type_flags(&self) -> BXDFTYPES {
        BXDFTYPES::REFLECTION | BXDFTYPES::SPECULAR
    }

    fn f(&self, _wo: Vector3<f64>, _wi: Vector3<f64>) -> Vector3<f64> {
        Vector3::zeros()
    }

    fn pdf(&self, _wo: Vector3<f64>, _wi: Vector3<f64>) -> f64 {
        1.0
    }

    fn sample_f(&self, _point: Point2<f64>, wo: Vector3<f64>) -> (Vector3<f64>, f64, Vector3<f64>) {
        let wi = Vector3::new(-wo.x, -wo.y, wo.z);
        let pdf = self.pdf(wo, wi);
        let f = self.fresnel.evaluate(cos_theta(wi)) * self.reflectance_color / abs_cos_theta(wi);

        (wi, pdf, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bsdf::helpers::fresnel::FresnelNoop;
    use approx::assert_relative_eq;

    #[test]
    fn test_specular_reflection_sample_f() {
        let reflectance = Vector3::new(1.0, 1.0, 1.0);
        let fresnel = Fresnel::Noop(FresnelNoop::new());
        let bxdf = SpecularReflection::new(reflectance, fresnel);

        let wo = Vector3::new(1.0, 1.0, 1.0).normalize();
        let sample = Point2::new(0.5, 0.5);

        let (wi, pdf, f) = bxdf.sample_f(sample, wo);

        // Check reflection direction
        // For normal (0,0,1), reflection of (x,y,z) is (-x,-y,z)
        assert_relative_eq!(wi.x, -wo.x);
        assert_relative_eq!(wi.y, -wo.y);
        assert_relative_eq!(wi.z, wo.z);

        // Check PDF
        assert_relative_eq!(pdf, 1.0);

        // Check f
        // f = fresnel * reflectance / abs_cos_theta(wi)
        // fresnel is 1.0
        // reflectance is 1.0
        // abs_cos_theta(wi) is wi.z (since wi is normalized and in local coords)
        let expected_f = reflectance / wi.z.abs();
        assert_relative_eq!(f, expected_f);
    }

    #[test]
    fn test_specular_reflection_f() {
        let reflectance = Vector3::new(1.0, 1.0, 1.0);
        let fresnel = Fresnel::Noop(FresnelNoop::new());
        let bxdf = SpecularReflection::new(reflectance, fresnel);

        let wo = Vector3::new(1.0, 1.0, 1.0).normalize();
        let wi = Vector3::new(-wo.x, -wo.y, wo.z);

        let f = bxdf.f(wo, wi);
        assert_relative_eq!(f, Vector3::zeros());
    }

    #[test]
    fn test_specular_reflection_pdf() {
        let reflectance = Vector3::new(1.0, 1.0, 1.0);
        let fresnel = Fresnel::Noop(FresnelNoop::new());
        let bxdf = SpecularReflection::new(reflectance, fresnel);

        let wo = Vector3::new(1.0, 1.0, 1.0).normalize();
        let wi = Vector3::new(-wo.x, -wo.y, wo.z);

        let pdf = bxdf.pdf(wo, wi);
        assert_relative_eq!(pdf, 1.0);
    }

    #[test]
    fn test_specular_reflection_with_dielectric() {
        let reflectance = Vector3::new(1.0, 1.0, 1.0);
        let fresnel = Fresnel::Dielectric(FresnelDielectric::new(1.0, 1.5));
        let bxdf = SpecularReflection::new(reflectance, fresnel);

        let wo = Vector3::new(0.0, 0.0, 1.0); // Normal incidence
        let sample = Point2::new(0.5, 0.5);

        let (wi, _pdf, f) = bxdf.sample_f(sample, wo);

        // Reflection direction should be same as wo for normal incidence (but flipped x, y which are 0)
        assert_relative_eq!(wi.x, -wo.x);
        assert_relative_eq!(wi.y, -wo.y);
        assert_relative_eq!(wi.z, wo.z);

        // Fresnel term for normal incidence: ((n1-n2)/(n1+n2))^2
        // ((1.0-1.5)/(1.0+1.5))^2 = (-0.5/2.5)^2 = (-0.2)^2 = 0.04
        let expected_fresnel = 0.04;
        let expected_f = reflectance * expected_fresnel / wi.z.abs();

        assert_relative_eq!(f, expected_f);
    }

    #[test]
    fn test_specular_reflection_with_dielectric_low_angle() {
        let reflectance = Vector3::new(1.0, 1.0, 1.0);
        let fresnel = Fresnel::Dielectric(FresnelDielectric::new(1.0, 1.5));
        let bxdf = SpecularReflection::new(reflectance, fresnel);

        let wo = Vector3::new(1.0, 0.0, 0.01).normalize(); // Normal incidence
        let sample = Point2::new(0.5, 0.5);

        let (wi, _pdf, f) = bxdf.sample_f(sample, wo);

        // Reflection direction should be same as wo for normal incidence (but flipped x, y)
        assert_relative_eq!(wi.x, -wo.x);
        assert_relative_eq!(wi.y, -wo.y);
        assert_relative_eq!(wi.z, wo.z);

        // Fresnel term for normal incidence: approaches 1.0 at grazing angles
        let expected_fresnel = fresnel.evaluate(cos_theta(wi));
        let expected_f = reflectance * expected_fresnel / wi.z.abs();

        assert_relative_eq!(f, expected_f);
    }
}
