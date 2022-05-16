use bitflags::bitflags;
use nalgebra::{Point3, Vector3};
use rand::prelude::SliceRandom;
use rand::thread_rng;

use bsdf::lambertian::Lambertian;
use bsdf::specular_reflection::SpecularReflection;
use renderer::{debug_write_pixel, debug_write_pixel_f64};
use surface_interaction::SurfaceInteraction;

pub mod fresnel;
pub mod lambertian;
pub mod specular_reflection;

const MAX_BXDF_COUNT: usize = 5;

#[derive(Copy, Clone, Debug)]
pub struct BSDF {
    bxdfs: [Option<BXDF>; MAX_BXDF_COUNT],
    ior: f64,
    geometry_normal: Vector3<f64>,
    shading_normal: Vector3<f64>,
    ss: Vector3<f64>,
    ts: Vector3<f64>,
}

impl BSDF {
    pub fn new(surface_interaction: SurfaceInteraction, ior: Option<f64>) -> BSDF {
        BSDF {
            bxdfs: [None; MAX_BXDF_COUNT],
            ior: ior.unwrap_or(1.0),
            geometry_normal: surface_interaction.geometry_normal,
            shading_normal: surface_interaction.shading_normal,
            ss: surface_interaction.ss,
            ts: surface_interaction.ts,
        }
    }

    pub fn add(&mut self, bxdf: BXDF) -> &mut BSDF {
        let slot = self.bxdfs.iter_mut().find(|x| x.is_none()).unwrap();

        *slot = Some(bxdf);

        self
    }

    pub fn sample_f(
        &self,
        wo_world: Vector3<f64>,
        bxdf_types_flags: BXDFTYPES,
    ) -> (Vector3<f64>, f64, Vector3<f64>) {
        let mut rng = thread_rng();

        let bxdfs: Vec<&BXDF> = self
            .bxdfs
            .iter()
            .filter_map(|bxdf| {
                if let Some(bxdf) = bxdf {
                    if bxdf.get_type_flags().intersects(bxdf_types_flags) {
                        return Some(bxdf);
                    }
                }

                None
            })
            .collect();

        if bxdfs.len() == 0 {
            return (Vector3::zeros(), 0.0, Vector3::zeros());
        }

        let wo = self.world_to_local(wo_world);

        let (wi, pdf, f) = bxdfs
            .choose(&mut rng)
            .unwrap()
            .sample_f(Point3::new(1.0, 1.0, 1.0), wo);

        let wi_world = self.local_to_world(wi);
        //debug_write_pixel_f64(pdf);
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
        let reflect =
            wi_world.dot(&self.geometry_normal) * wo_world.dot(&self.geometry_normal) > 0.0;
        let must_match_type = match reflect {
            true => BXDFTYPES::REFLECTION,
            false => BXDFTYPES::TRANSMISSION,
        };

        let mut f = Vector3::zeros();
        for bxdf in &self.bxdfs.iter().filter_map(|x| *x).collect::<Vec<_>>() {
            if bxdf.get_type_flags().intersects(bxdf_types_flags)
                && bxdf.get_type_flags().contains(must_match_type)
            {
                f += bxdf.f(wo, wi);
            }
        }

        // shadow terminator offset
        f *= shift_cos_in(wi_world.dot(&self.shading_normal), 1.002);

        f
    }

    pub fn pdf(
        &self,
        wo_world: Vector3<f64>,
        wi_world: Vector3<f64>,
        bxdf_types_flags: BXDFTYPES,
    ) -> f64 {
        let wi = self.world_to_local(wi_world);
        let wo = self.world_to_local(wo_world);
        let reflect =
            wi_world.dot(&self.geometry_normal) * wo_world.dot(&self.geometry_normal) > 0.0;
        let must_match_type = match reflect {
            true => BXDFTYPES::REFLECTION,
            false => BXDFTYPES::TRANSMISSION,
        };

        let mut pdf = 0.0;
        for bxdf in &self.bxdfs.iter().filter_map(|x| *x).collect::<Vec<_>>() {
            if bxdf.get_type_flags().intersects(bxdf_types_flags)
                && bxdf.get_type_flags().contains(must_match_type)
            {
                pdf += bxdf.pdf(wo, wi);
            }
        }

        pdf
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

fn bump_shadowing_term(
    normal_geometry: Vector3<f64>,
    normal_shading: Vector3<f64>,
    wi: Vector3<f64>,
) -> f64 {
    let g =
        (normal_geometry.dot(&wi) / normal_shading.dot(&wi)) * normal_geometry.dot(&normal_shading);

    if g >= 1.0
    {
        return 1.0;
    }

    if g < 0.0
    {
        return 0.0;
    }

    let g2 = g.powf(2.0);
    -g2 * g + g2 + g
}

fn shift_cos_in(cos_in: f64, frequency_multiplier: f64) -> f64 {
    let cos_in = cos_in.min(1.0);
    let angle = cos_in.acos();
    (angle * frequency_multiplier).cos().max(0.0) / cos_in
}

bitflags! {
    pub struct BXDFTYPES: u32 {
        const REFLECTION = 0b00000001;
        const REFRACTION = 0b00000010;
        const DIFFUSE = 0b00000100;
        const SPECULAR = 0b00001000;
        const TRANSMISSION = 0b00010000;
        const ALL = Self::REFLECTION.bits | Self::REFRACTION.bits | Self::DIFFUSE.bits | Self::SPECULAR.bits;
    }
}

#[derive(Debug, Clone, Copy)]
pub enum BXDF {
    Lambertian(Lambertian),
    SpecularReflection(SpecularReflection),
}

pub trait BXDFtrait {
    fn get_type_flags(&self) -> BXDFTYPES;
    fn f(&self, wo: Vector3<f64>, wi: Vector3<f64>) -> Vector3<f64>;
    fn pdf(&self, wo: Vector3<f64>, wi: Vector3<f64>) -> f64;
    fn sample_f(&self, _point: Point3<f64>, wo: Vector3<f64>) -> (Vector3<f64>, f64, Vector3<f64>);
}

impl BXDFtrait for BXDF {
    fn get_type_flags(&self) -> BXDFTYPES {
        match self {
            BXDF::Lambertian(x) => x.get_type_flags(),
            BXDF::SpecularReflection(x) => x.get_type_flags(),
        }
    }

    fn f(&self, wo: Vector3<f64>, wi: Vector3<f64>) -> Vector3<f64> {
        match self {
            BXDF::Lambertian(x) => x.f(wo, wi),
            BXDF::SpecularReflection(x) => x.f(wo, wi),
        }
    }

    fn pdf(&self, wo: Vector3<f64>, wi: Vector3<f64>) -> f64 {
        match self {
            BXDF::Lambertian(x) => x.pdf(wo, wi),
            BXDF::SpecularReflection(x) => x.pdf(wo, wi),
        }
    }

    fn sample_f(&self, _point: Point3<f64>, wo: Vector3<f64>) -> (Vector3<f64>, f64, Vector3<f64>) {
        match self {
            BXDF::Lambertian(x) => x.sample_f(_point, wo),
            BXDF::SpecularReflection(x) => x.sample_f(_point, wo),
        }
    }
}
