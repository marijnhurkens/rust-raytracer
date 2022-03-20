use bitflags::bitflags;
use nalgebra::Vector3;
use surface_interaction::SurfaceInteraction;

pub struct BSDF {
    bxdfs: Vec<BXDF>,
    ior: f64,
    geometry_normal: Vector3<f64>,
    shading_normal: Vector3<f64>,
}

impl BSDF {
    pub fn new(
        surface_interaction: SurfaceInteraction,
        ior: Option<f64>
    ) -> BSDF {
        BSDF {
            bxdfs: vec![],
            ior: ior.unwrap_or(1.0),
            geometry_normal: surface_interaction.surface_normal,
            shading_normal: surface_interaction.surface_normal
        }
    }

    pub fn add(&mut self, bxdf: BXDF) -> &mut BSDF {
        self.bxdfs.push(bxdf);

        self
    }

  //  pub fn f()

    //fn world_to_local()
}

bitflags! {
    struct BXDFTYPES: u32 {
        const REFLECTION = 0b00000001;
        const REFRACTION = 0b00000010;
        const DIFFUSE = 0b00000100;
        const ALL = Self::REFLECTION.bits | Self::REFRACTION.bits | Self::DIFFUSE.bits;
    }
}

pub struct BXDF {
    type_flags: BXDFTYPES
}

