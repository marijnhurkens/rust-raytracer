#[derive(Debug, Clone, Copy)]
pub enum Fresnel {
    Noop(FresnelNoop),
    Dielectric(FresnelDielectric),
}

pub trait FresnelTrait {
    fn evaluate(&self, cos_i: f64) -> f64;
}

impl FresnelTrait for Fresnel {
    fn evaluate(&self, cos_i: f64) -> f64 {
        match self {
            Fresnel::Noop(x) => x.evaluate(cos_i),
            Fresnel::Dielectric(x) => x.evaluate(cos_i),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct FresnelNoop {}

impl FresnelNoop {
    pub fn new() -> Self {
        FresnelNoop {}
    }
}

impl FresnelTrait for FresnelNoop {
    fn evaluate(&self, cos_theta_i: f64) -> f64 {
        1.0
    }
}

#[derive(Copy, Clone, Debug)]
pub struct FresnelDielectric {
    eta_i: f64,
    eta_t: f64,
}

impl FresnelDielectric {
    pub fn new(eta_i: f64, eta_t: f64) -> Self {
        FresnelDielectric { eta_i, eta_t }
    }
}

impl FresnelTrait for FresnelDielectric {
    fn evaluate(&self, cos_theta_i: f64) -> f64 {
        let mut eta_i = self.eta_i;
        let mut eta_t = self.eta_t;
        let mut cos_theta_i = cos_theta_i.clamp(-1.0, 1.0);

        if cos_theta_i <= 0.0 {
            std::mem::swap(&mut eta_i, &mut eta_t);
            cos_theta_i = cos_theta_i.abs();
        }

        let sin_theta_i = (1.0 - cos_theta_i * cos_theta_i).max(0.0).sqrt();
        let sin_theta_t = eta_i / eta_t * sin_theta_i;

        if sin_theta_t >= 1.0 {
            return 1.0;
        }

        let cos_theta_t = (1.0 - sin_theta_t * sin_theta_t).max(0.0).sqrt();

        let rpar_l = ((eta_t * cos_theta_i) - (eta_i * cos_theta_t))
            / ((eta_t * cos_theta_i) + (eta_i * cos_theta_t));
        let rper_n = ((eta_i * cos_theta_i) - (eta_t * cos_theta_t))
            / ((eta_i * cos_theta_i) + (eta_t * cos_theta_t));

        (rpar_l * rpar_l + rper_n * rper_n) / 2.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_fresnel_dielectric_normal_incidence() {
        // Air to Glass
        let fresnel = FresnelDielectric::new(1.0, 1.5);
        // R = ((1 - 1.5) / (1 + 1.5))^2 = (-0.5 / 2.5)^2 = 0.04
        assert_relative_eq!(fresnel.evaluate(1.0), 0.04, epsilon = 1e-4);
    }

    #[test]
    fn test_fresnel_dielectric_no_reflection() {
        // Matched indices
        let fresnel = FresnelDielectric::new(1.5, 1.5);
        assert_relative_eq!(fresnel.evaluate(1.0), 0.0, epsilon = 1e-4);
        assert_relative_eq!(fresnel.evaluate(0.5), 0.0, epsilon = 1e-4);
    }

    #[test]
    fn test_fresnel_dielectric_tir() {
        // Glass to Air
        // n1 = 1.5, n2 = 1.0
        // Critical angle sin(theta_c) = 1/1.5 = 0.666...
        // theta_c approx 41.8 degrees.
        // Test at 60 degrees. cos(60) = 0.5.
        // We need to simulate coming from the denser medium.
        // If we construct with (1.5, 1.0), then cos_theta_i > 0 means we are in 1.5 going to 1.0.
        let fresnel = FresnelDielectric::new(1.5, 1.0);
        assert_relative_eq!(fresnel.evaluate(0.5), 1.0, epsilon = 1e-4);
    }

    #[test]
    fn test_fresnel_dielectric_swap() {
        // Air to Glass, but ray coming from Glass side (backface).
        // n_i = 1.0, n_t = 1.5.
        // cos_theta_i = -1.0 (normal incidence from back).
        // Should behave like Glass to Air at normal incidence.
        // R = ((1.5 - 1) / (1.5 + 1))^2 = 0.04.
        let fresnel = FresnelDielectric::new(1.0, 1.5);
        assert_relative_eq!(fresnel.evaluate(-1.0), 0.04, epsilon = 1e-4);
    }

    #[test]
    fn test_fresnel_dielectric_parallel_perpendicular() {
        // Check a specific angle where we know the result or can calculate it manually.
        // n1=1.0, n2=1.5. theta_i = 60 deg. cos_i = 0.5.
        // sin_i = sqrt(1 - 0.5^2) = sqrt(0.75) = 0.866025
        // sin_t = (1/1.5) * sin_i = 0.666 * 0.866 = 0.57735
        // cos_t = sqrt(1 - sin_t^2) = sqrt(1 - 0.3333) = sqrt(0.6666) = 0.81649

        // r_par = (1.5 * 0.5 - 1.0 * 0.81649) / (1.5 * 0.5 + 1.0 * 0.81649)
        //       = (0.75 - 0.81649) / (0.75 + 0.81649)
        //       = -0.06649 / 1.56649 = -0.042445

        // r_per = (1.0 * 0.5 - 1.5 * 0.81649) / (1.0 * 0.5 + 1.5 * 0.81649)
        //       = (0.5 - 1.224735) / (0.5 + 1.224735)
        //       = -0.724735 / 1.724735 = -0.42020

        // R = (r_par^2 + r_per^2) / 2
        //   = (0.001801 + 0.176568) / 2
        //   = 0.178369 / 2 = 0.08918

        let fresnel = FresnelDielectric::new(1.0, 1.5);
        let result = fresnel.evaluate(0.5);
        // Let's be a bit generous with epsilon as my manual calc was rough
        assert_relative_eq!(result, 0.08918, epsilon = 1e-3);
    }

    #[test]
    fn test_fresnel_noop() {
        let fresnel = FresnelNoop::new();
        assert_relative_eq!(fresnel.evaluate(1.0), 1.0);
        assert_relative_eq!(fresnel.evaluate(0.5), 1.0);
        assert_relative_eq!(fresnel.evaluate(-0.5), 1.0);
    }
}
