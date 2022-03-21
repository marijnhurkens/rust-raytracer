use nalgebra::{Point3, Vector2, Vector3};

// new method for intersections
pub struct SurfaceInteraction {
    pub point: Point3<f64>,
    pub surface_normal: Vector3<f64>,
    pub wo: Vector3<f64>,
    pub uv: Vector2<f64>,
}

impl SurfaceInteraction {
    pub fn new(
        point: Point3<f64>,
        surface_normal: Vector3<f64>,
        wo: Vector3<f64>,
        uv: Vector2<f64>,
    ) -> SurfaceInteraction {
        SurfaceInteraction {
            point,
            surface_normal,
            wo,
            uv,
        }
    }
}
