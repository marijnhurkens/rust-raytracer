use nalgebra::{Point3, Vector2, Vector3};

use bsdf::Bsdf;
use helpers::face_forward;

pub struct Interaction {
    pub point: Point3<f64>,
    pub normal: Vector3<f64>,
}

#[derive(Copy, Clone, Debug)]
pub struct SurfaceInteraction {
    pub bsdf: Option<Bsdf>,
    pub point: Point3<f64>,
    pub geometry_normal: Vector3<f64>,
    pub shading_normal: Vector3<f64>,
    pub wo: Vector3<f64>,
    pub uv: Vector2<f64>,
    pub ss: Vector3<f64>,
    pub ts: Vector3<f64>,
    pub delta_p_delta_u: Vector3<f64>,
    pub delta_p_delta_v: Vector3<f64>,
    pub p_error: Vector3<f64>,
}

impl SurfaceInteraction {
    pub fn new(
        point: Point3<f64>,
        geometry_normal: Vector3<f64>,
        wo: Vector3<f64>,
        uv: Vector2<f64>,
        ss: Vector3<f64>,
        ts: Vector3<f64>,
        delta_p_delta_u: Vector3<f64>,
        delta_p_delta_v: Vector3<f64>,
        p_error: Vector3<f64>,
    ) -> SurfaceInteraction {
        let shading_normal = ss.cross(&ts).normalize();
        let geometry_normal = face_forward(geometry_normal, shading_normal);

        SurfaceInteraction {
            bsdf: None,
            point,
            geometry_normal,
            shading_normal,
            wo,
            uv,
            ss,
            ts,
            delta_p_delta_u,
            delta_p_delta_v,
            p_error,
        }
    }

}
