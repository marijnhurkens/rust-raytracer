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
