use nalgebra::{Point3, Vector3};

use bsdf::fresnel::DielectricFresnel;
use bsdf::{BXDFtrait, BXDFTYPES};
use bsdf::fresnel::Fresnel;
use bsdf::helpers::{abs_cos_theta, cos_theta};

#[derive(Debug, Clone, Copy)]
pub struct SpecularReflection {
    reflectance_color: Vector3<f64>,
    fresnel: DielectricFresnel,
}

impl SpecularReflection {
    pub fn new(reflectance_color: Vector3<f64>, fresnel: DielectricFresnel) -> Self {
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

    fn sample_f(&self, _point: Point3<f64>, wo: Vector3<f64>) -> (Vector3<f64>, f64, Vector3<f64>) {
        let wi = Vector3::new(-wo.x, -wo.y, wo.z);
        let pdf = self.pdf(wo, wi);
        let f = self.fresnel.evaluate(cos_theta(wi)) * self.reflectance_color / abs_cos_theta(wi);

        (wi, pdf, f)
    }
}
