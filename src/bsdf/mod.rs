use bitflags::bitflags;
use nalgebra::{Point3, Vector3};

use bsdf::lambertian::Lambertian;
use helpers::{abs_cos_theta, get_cosine_weighted_in_hemisphere, same_hemisphere};
use surface_interaction::SurfaceInteraction;

pub mod lambertian;

#[derive(Clone, Debug)]
pub struct BSDF {
    bxdfs: Vec<BXDF>,
    ior: f64,
    geometry_normal: Vector3<f64>,
    shading_normal: Vector3<f64>,
    ss: Vector3<f64>,
    ts: Vector3<f64>,
}

impl BSDF {
    pub fn new(surface_interaction: SurfaceInteraction, ior: Option<f64>) -> BSDF {
        let mut ss = surface_interaction.delta_p_delta_u.normalize();
        let ts = surface_interaction.surface_normal.cross(&ss);

        ss = surface_interaction.surface_normal.cross(&ts);

        BSDF {
            bxdfs: vec![],
            ior: ior.unwrap_or(1.0),
            geometry_normal: surface_interaction.surface_normal,
            shading_normal: surface_interaction.surface_normal,
            ss,
            ts,
        }
    }

    pub fn add(&mut self, bxdf: BXDF) -> &mut BSDF {
        self.bxdfs.push(bxdf);

        self
    }

    pub fn sample_f(
        &self,
        wo_world: Vector3<f64>,
        bxdf_types_flags: BXDFTYPES,
    ) -> (Vector3<f64>, f64, Vector3<f64>) {
        let bxdfs: Vec<&BXDF> = self
            .bxdfs
            .iter()
            .filter(|&bxdf| bxdf.get_type_flags().intersects(bxdf_types_flags))
            .collect();

        if bxdfs.len() == 0 {
            return (Vector3::zeros(), 0.0, Vector3::zeros());
        }

        let wo = self.world_to_local(wo_world);

        let (wi, pdf, f) = bxdfs[0].sample_f(Point3::new(1.0, 1.0, 1.0), wo);

        let wi_world = self.local_to_world(wi);
        (wi_world, pdf, f)
    }

    pub fn f(
        &self,
        wo_world: Vector3<f64>,
        wi_world: Vector3<f64>,
        bxdf_types_flags: BXDFTYPES,
    ) -> Vector3<f64> {
        let wi = self.world_to_local(wi_world);
        let wo = self.world_to_local(wo_world);

        let mut f = Vector3::zeros();
        for bxdf in &self.bxdfs {
            if bxdf.get_type_flags().intersects(bxdf_types_flags) {
                f += bxdf.f(wo, wi);
            }
        }

        f
    }

    fn world_to_local(&self, v: Vector3<f64>) -> Vector3<f64> {
        Vector3::new(
            v.dot(&self.ss),
            v.dot(&self.ts),
            v.dot(&self.shading_normal),
        )
    }

    fn local_to_world(&self, v: Vector3<f64>) -> Vector3<f64> {
        Vector3::new(
            self.ss.x * v.x + self.ts.x * v.y + self.shading_normal.x * v.z,
            self.ss.y * v.x + self.ts.y * v.y + self.shading_normal.y * v.z,
            self.ss.z * v.x + self.ts.z * v.y + self.shading_normal.z * v.z,
        )
    }
}

bitflags! {
    pub struct BXDFTYPES: u32 {
        const REFLECTION = 0b00000001;
        const REFRACTION = 0b00000010;
        const DIFFUSE = 0b00000100;
        const ALL = Self::REFLECTION.bits | Self::REFRACTION.bits | Self::DIFFUSE.bits;
    }
}

#[derive(Debug, Clone)]
pub enum BXDF {
    Lambertian(Lambertian),
}

pub trait BXDFtrait {
    fn get_type_flags(&self) -> BXDFTYPES;
    fn f(&self, wo: Vector3<f64>, wi: Vector3<f64>) -> Vector3<f64>;
}

impl BXDFtrait for BXDF {
    fn get_type_flags(&self) -> BXDFTYPES {
        match self {
            BXDF::Lambertian(x) => x.get_type_flags(),
        }
    }

    fn f(&self, wo: Vector3<f64>, wi: Vector3<f64>) -> Vector3<f64> {
        match self {
            BXDF::Lambertian(x) => x.f(wo, wi),
        }
    }
}

impl BXDF {
    pub fn pdf(&self, wo: Vector3<f64>, wi: Vector3<f64>) -> f64 {
        if same_hemisphere(wo, wi) {
            abs_cos_theta(wi) * std::f64::consts::FRAC_1_PI
        } else {
            0.0
        }
    }

    pub fn sample_f(
        &self,
        _point: Point3<f64>,
        wo: Vector3<f64>,
    ) -> (Vector3<f64>, f64, Vector3<f64>) {
        let mut wi = get_cosine_weighted_in_hemisphere();
        if wo.z < 0.0 {
            wi.z = -wi.z;
        }

        (wi, self.pdf(wo, wi), self.f(wo, wi))
    }
}
