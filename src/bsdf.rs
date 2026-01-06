use bitflags::bitflags;
use nalgebra::{Point2, Point3, Vector3};
use num_traits::Float;
use rand::prelude::{IteratorRandom, SliceRandom};
use rand::{rng, Rng};

use crate::bsdf::helpers::{abs_cos_theta, get_cosine_weighted_in_hemisphere, same_hemisphere};
use crate::bsdf::lambertian::Lambertian;
use crate::bsdf::microfacet_reflection::MicrofacetReflection;
use crate::bsdf::microfacet_transmission::MicrofacetTransmission;
use crate::bsdf::oren_nayar::OrenNayar;
use crate::bsdf::specular_reflection::SpecularReflection;
use crate::bsdf::specular_transmission::SpecularTransmission;
use crate::bsdf::rough_dielectric::RoughDielectric;
use crate::renderer::{
    debug_write_pixel, debug_write_pixel_f64, debug_write_pixel_f64_on_bounce,
    debug_write_pixel_on_bounce,
};
use crate::surface_interaction::SurfaceInteraction;

pub mod helpers;
pub mod lambertian;
pub mod microfacet_reflection;
pub mod microfacet_transmission;
pub mod oren_nayar;
pub mod specular_reflection;
pub mod specular_transmission;
pub mod rough_dielectric;

const MAX_BXDF_COUNT: usize = 5;

#[derive(Copy, Clone, Debug)]
pub struct Bsdf {
    bxdfs: [Option<Bxdf>; MAX_BXDF_COUNT],
    ior: f64,
    geometry_normal: Vector3<f64>,
    shading_normal: Vector3<f64>,
    ss: Vector3<f64>,
    ts: Vector3<f64>,
}

#[derive(Debug)]
pub struct BsdfSampleResult {
    pub wi: Vector3<f64>,
    pub pdf: f64,
    pub f: Vector3<f64>,
    pub sampled_flags: BXDFTYPES,
}

impl Bsdf {
    pub fn new(surface_interaction: SurfaceInteraction, ior: Option<f64>) -> Bsdf {
        Bsdf {
            bxdfs: [None; MAX_BXDF_COUNT],
            ior: ior.unwrap_or(1.0),
            geometry_normal: surface_interaction.geometry_normal,
            shading_normal: surface_interaction.shading_normal,
            ss: surface_interaction.ss,
            ts: surface_interaction.ts,
        }
    }

    pub fn add(&mut self, bxdf: Bxdf) -> &mut Bsdf {
        let slot = self.bxdfs.iter_mut().find(|x| x.is_none()).unwrap();

        *slot = Some(bxdf);

        self
    }

    pub fn eta(&self) -> f64 {
        self.ior
    }

    pub fn has_bxdfs_with_flags(&self, bxdf_types_flags: BXDFTYPES) -> bool {
        self.bxdfs
            .iter()
            .any(|x| x.unwrap().get_type_flags().intersects(bxdf_types_flags))
    }

    pub fn sample_f(
        &self,
        wo_world: Vector3<f64>,
        bxdf_types_flags: BXDFTYPES,
        sample_u: Point2<f64>,
    ) -> BsdfSampleResult {
        let bxdfs_matching: Vec<usize> = self
            .bxdfs
            .iter()
            .enumerate()
            .filter_map(|(i, bxdf)| {
                if let Some(bxdf) = bxdf {
                    if bxdf.get_type_flags().intersects(bxdf_types_flags) {
                        return Some(i);
                    }
                }

                None
            })
            .collect();
        let matching_bxdf_count = bxdfs_matching.len();

        if bxdfs_matching.is_empty() {
            return BsdfSampleResult {
                wi: Vector3::zeros(),
                pdf: 0.0,
                f: Vector3::zeros(),
                sampled_flags: BXDFTYPES::NONE,
            };
        }

        let wo = self.world_to_local(wo_world);
        if wo.z == 0.0 {
            return BsdfSampleResult {
                wi: Vector3::zeros(),
                pdf: 0.0,
                f: Vector3::zeros(),
                sampled_flags: BXDFTYPES::NONE,
            };
        }

        let chosen_index = ((sample_u.x * matching_bxdf_count as f64).floor() as usize)
            .min(matching_bxdf_count - 1);

        let sample_2_remapped = Point2::new(
            ((sample_u.x * matching_bxdf_count as f64) - chosen_index as f64)
                .min(1.0 - f64::epsilon()),
            sample_u.y,
        );

        let bxdf = self.bxdfs[chosen_index].as_ref().unwrap();
        let (wi, mut pdf, mut f) = bxdf.sample_f(sample_2_remapped, wo);
        if pdf == 0.0 {
            return BsdfSampleResult {
                wi: Vector3::zeros(),
                pdf: 0.0,
                f: Vector3::zeros(),
                sampled_flags: bxdf.get_type_flags(),
            };
        }

       // debug_write_pixel_on_bounce(wi, 0);
        //debug_write_pixel_f64_on_bounce(pdf, 0);

        let wi_world = self.local_to_world(wi);

        if !bxdf.get_type_flags().contains(BXDFTYPES::SPECULAR) || matching_bxdf_count > 1 {
            for (i, bxdf_loop) in self.bxdfs.iter().enumerate() {
                if let Some(bxdf_loop) = bxdf_loop {
                    if i != chosen_index && bxdf_loop.get_type_flags().intersects(bxdf_types_flags)
                    {
                        pdf += bxdf_loop.pdf(wo, wi);
                    }
                }
            }
        }

        if matching_bxdf_count > 1 {
            pdf /= (matching_bxdf_count as f64);
        }

        if !bxdf.get_type_flags().contains(BXDFTYPES::SPECULAR) {
            let reflect =
                wi_world.dot(&self.geometry_normal) * wo_world.dot(&self.geometry_normal) > 0.0;

            f = Vector3::zeros();
            for bxdf in &self.bxdfs.iter().filter_map(|x| *x).collect::<Vec<_>>() {
                if bxdf.get_type_flags().intersects(bxdf_types_flags)
                    && ((reflect && bxdf.get_type_flags().contains(BXDFTYPES::REFLECTION))
                        || (!reflect && bxdf.get_type_flags().contains(BXDFTYPES::TRANSMISSION)))
                {
                    f += bxdf.f(wo, wi);
                }
            }
        }

        BsdfSampleResult {
            wi: wi_world,
            pdf,
            f,
            sampled_flags: bxdf.get_type_flags(),
        }
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
        if wo.z == 0.0 {
            return 0.0;
        }

        let mut pdf = 0.0;
        let mut matching_bxdf_count = 0;
        for bxdf in &self.bxdfs.iter().filter_map(|x| *x).collect::<Vec<_>>() {
            if bxdf.get_type_flags().intersects(bxdf_types_flags) {
                matching_bxdf_count += 1;
                pdf += bxdf.pdf(wo, wi);
            }
        }

        if matching_bxdf_count > 0 {
            return pdf / matching_bxdf_count as f64;
        }

        0.0
    }

    fn world_to_local(&self, v: Vector3<f64>) -> Vector3<f64> {
        // Vector3::new(
        //     v.dot(&self.ss),
        //     v.dot(&self.ts),
        //     v.dot(&self.shading_normal),
        // )

        // Rebuild an orthonormal basis from ss + shading_normal, without stored ts.
        let n = self.shading_normal.normalize();

        // Make ss orthogonal to n (Gram–Schmidt) and normalize.
        let mut s = self.ss - n * self.ss.dot(&n);
        if s.magnitude_squared() <= 1e-20 {
            // Degenerate: pick any vector not parallel to n as a seed.
            let seed = if n.x.abs() < 0.9 {
                Vector3::new(1.0, 0.0, 0.0)
            } else {
                Vector3::new(0.0, 1.0, 0.0)
            };
            s = (seed - n * seed.dot(&n)).normalize();
        } else {
            s = s.normalize();
        }

        // Derive t to complete a right-handed frame.
        let t = n.cross(&s);

        Vector3::new(v.dot(&s), v.dot(&t), v.dot(&n))
    }

    fn local_to_world(&self, v: Vector3<f64>) -> Vector3<f64> {
        // Vector3::new(
        //     self.ss.x * v.x + self.ts.x * v.y + self.shading_normal.x * v.z,
        //     self.ss.y * v.x + self.ts.y * v.y + self.shading_normal.y * v.z,
        //     self.ss.z * v.x + self.ts.z * v.y + self.shading_normal.z * v.z,
        // )

        // Rebuild the same orthonormal basis used by world_to_local().
        let n = self.shading_normal.normalize();

        let mut s = self.ss - n * self.ss.dot(&n);
        if s.magnitude_squared() <= 1e-20 {
            let seed = if n.x.abs() < 0.9 {
                Vector3::new(1.0, 0.0, 0.0)
            } else {
                Vector3::new(0.0, 1.0, 0.0)
            };
            s = (seed - n * seed.dot(&n)).normalize();
        } else {
            s = s.normalize();
        }

        let t = n.cross(&s);

        // World = s * x + t * y + n * z
        s * v.x + t * v.y + n * v.z
    }
}

fn bump_shadowing_term(
    normal_geometry: Vector3<f64>,
    normal_shading: Vector3<f64>,
    wi: Vector3<f64>,
) -> f64 {
    let g =
        (normal_geometry.dot(&wi) / normal_shading.dot(&wi)) * normal_geometry.dot(&normal_shading);

    if g >= 1.0 {
        return 1.0;
    }

    if g < 0.0 {
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
        const GLOSSY= 0b00100000;
        const ALL = Self::REFLECTION.bits | Self::REFRACTION.bits | Self::DIFFUSE.bits |
        Self::SPECULAR.bits | Self::GLOSSY.bits;
        const NONE = 0b00000000;
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Bxdf {
    Lambertian(Lambertian),
    SpecularReflection(SpecularReflection),
    SpecularTransmission(SpecularTransmission),
    OrenNayar(OrenNayar),
    MicrofacetReflection(MicrofacetReflection),
    MicrofacetTransmission(MicrofacetTransmission),
    RoughDielectric(RoughDielectric),
}

pub trait BXDFtrait {
    fn get_type_flags(&self) -> BXDFTYPES;
    fn f(&self, wo: Vector3<f64>, wi: Vector3<f64>) -> Vector3<f64>;
    fn pdf(&self, wo: Vector3<f64>, wi: Vector3<f64>) -> f64 {
        if same_hemisphere(wo, wi) {
            abs_cos_theta(wi) * std::f64::consts::FRAC_1_PI
        } else {
            0.0
        }
    }
    fn sample_f(&self, _point: Point2<f64>, wo: Vector3<f64>) -> (Vector3<f64>, f64, Vector3<f64>) {
        let mut wi = get_cosine_weighted_in_hemisphere();
        if wo.z < 0.0 {
            wi.z = -wi.z;
        }

        (wi, self.pdf(wo, wi), self.f(wo, wi))
    }
}

impl BXDFtrait for Bxdf {
    fn get_type_flags(&self) -> BXDFTYPES {
        match self {
            Bxdf::Lambertian(x) => x.get_type_flags(),
            Bxdf::SpecularReflection(x) => x.get_type_flags(),
            Bxdf::SpecularTransmission(x) => x.get_type_flags(),
            Bxdf::OrenNayar(x) => x.get_type_flags(),
            Bxdf::MicrofacetReflection(x) => x.get_type_flags(),
            Bxdf::MicrofacetTransmission(x) => x.get_type_flags(),
            Bxdf::RoughDielectric(x) => x.get_type_flags(),
        }
    }

    fn f(&self, wo: Vector3<f64>, wi: Vector3<f64>) -> Vector3<f64> {
        match self {
            Bxdf::Lambertian(x) => x.f(wo, wi),
            Bxdf::SpecularReflection(x) => x.f(wo, wi),
            Bxdf::SpecularTransmission(x) => x.f(wo, wi),
            Bxdf::OrenNayar(x) => x.f(wo, wi),
            Bxdf::MicrofacetReflection(x) => x.f(wo, wi),
            Bxdf::MicrofacetTransmission(x) => x.f(wo, wi),
            Bxdf::RoughDielectric(x) => x.f(wo, wi),
        }
    }

    fn pdf(&self, wo: Vector3<f64>, wi: Vector3<f64>) -> f64 {
        match self {
            Bxdf::Lambertian(x) => x.pdf(wo, wi),
            Bxdf::SpecularReflection(x) => x.pdf(wo, wi),
            Bxdf::SpecularTransmission(x) => x.pdf(wo, wi),
            Bxdf::OrenNayar(x) => x.pdf(wo, wi),
            Bxdf::MicrofacetReflection(x) => x.pdf(wo, wi),
            Bxdf::MicrofacetTransmission(x) => x.pdf(wo, wi),
            Bxdf::RoughDielectric(x) => x.pdf(wo, wi),
        }
    }

    fn sample_f(&self, point: Point2<f64>, wo: Vector3<f64>) -> (Vector3<f64>, f64, Vector3<f64>) {
        match self {
            Bxdf::Lambertian(x) => x.sample_f(point, wo),
            Bxdf::SpecularReflection(x) => x.sample_f(point, wo),
            Bxdf::SpecularTransmission(x) => x.sample_f(point, wo),
            Bxdf::OrenNayar(x) => x.sample_f(point, wo),
            Bxdf::MicrofacetReflection(x) => x.sample_f(point, wo),
            Bxdf::MicrofacetTransmission(x) => x.sample_f(point, wo),
            Bxdf::RoughDielectric(x) => x.sample_f(point, wo),
        }
    }
}
