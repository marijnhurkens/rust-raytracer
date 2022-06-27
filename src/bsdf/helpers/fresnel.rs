// todo: create enum 

pub trait Fresnel {
    fn evaluate(&self, cos_i: f64) -> f64;
}

#[derive(Copy, Clone, Debug)]
pub struct DielectricFresnel {
    eta_i: f64,
    eta_t: f64,
}

impl DielectricFresnel {
    pub fn new(eta_i: f64, eta_t: f64) -> Self {
        DielectricFresnel { eta_i, eta_t }
    }
}

impl Fresnel for DielectricFresnel {
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
