use nalgebra::{Point3, Vector2, Vector3};

use bsdf::BSDF;

#[derive(Copy, Clone, Debug)]
pub struct SurfaceInteraction {
    pub bsdf: Option<BSDF>,
    pub point: Point3<f64>,
    pub surface_normal: Vector3<f64>,
    pub wo: Vector3<f64>,
    pub uv: Vector2<f64>,
    pub delta_p_delta_u: Vector3<f64>,
    pub delta_p_delta_v: Vector3<f64>,
    pub p_error: Vector3<f64>,
}

impl SurfaceInteraction {
    pub fn new(
        point: Point3<f64>,
        surface_normal: Vector3<f64>,
        wo: Vector3<f64>,
        uv: Vector2<f64>,
        delta_p_delta_u: Vector3<f64>,
        delta_p_delta_v: Vector3<f64>,
        p_error: Vector3<f64>,
    ) -> SurfaceInteraction {
        SurfaceInteraction {
            bsdf: None,
            point,
            surface_normal,
            wo,
            uv,
            delta_p_delta_u,
            delta_p_delta_v,
            p_error,
        }
    }
}
