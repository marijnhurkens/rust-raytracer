// Float FrDielectric(Float cosThetaI, Float etaI, Float etaT) {
//     cosThetaI = Clamp(cosThetaI, -1, 1);
//     <<Potentially swap indices of refraction>>
//        bool entering = cosThetaI > 0.f;
//        if (!entering) {
//            std::swap(etaI, etaT);
//            cosThetaI = std::abs(cosThetaI);
//        }
//
//     <<Compute cosThetaT using Snell’s law>>
//        Float sinThetaI = std::sqrt(std::max((Float)0,
//                                             1 - cosThetaI * cosThetaI));
//        Float sinThetaT = etaI / etaT * sinThetaI;
//        <<Handle total internal reflection>>
//        Float cosThetaT = std::sqrt(std::max((Float)0,
//                                             1 - sinThetaT * sinThetaT));
//
//     Float Rparl = ((etaT * cosThetaI) - (etaI * cosThetaT)) /
//                   ((etaT * cosThetaI) + (etaI * cosThetaT));
//     Float Rperp = ((etaI * cosThetaI) - (etaT * cosThetaT)) /
//                   ((etaI * cosThetaI) + (etaT * cosThetaT));
//     return (Rparl * Rparl + Rperp * Rperp) / 2;
// }

fn fresnel_dielectric(cos_theta_i: f64, eta_i: f64, eta_t: f64) -> f64 {
    let mut eta_i = eta_i;
    let mut eta_t = eta_t;
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
    fn evaluate(&self, cos_i: f64) -> f64 {
        fresnel_dielectric(cos_i, self.eta_i, self.eta_t)
    }
}
