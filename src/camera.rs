use std::f64::consts::PI;
use std::sync::{Arc, RwLock};

use ggez::graphics::Transform;
use nalgebra::{
    Affine2, Affine3, Isometry3, Matrix4, Perspective3, Point2, Point3, Projective3, Quaternion,
    Rotation3, Scale3, SimdValue, Similarity3, Translation3, UnitQuaternion, Vector3,
};

use crate::helpers::Bounds;
use crate::renderer::Ray;
use crate::Film;

#[derive(Clone)]
pub struct Camera {
    pub position: Point3<f64>,
    pub target: Point3<f64>,
    pub fov: f64,
    pub aperture: f64,
    pub focal_distance: f64,
    pub film: Arc<RwLock<Film>>,
    camera_to_world: Matrix4<f64>,
    camera_to_screen: Matrix4<f64>,
    screen_to_raster: Matrix4<f64>,
    raster_to_screen: Matrix4<f64>,
    raster_to_camera: Matrix4<f64>,
}

/// World space -> scene
/// Camera space -> local camera coordinate system
/// Screen space -> camera space projected onto the film screen
/// Raster space -> x and y range from (0,0) to (resolution.x, resolution.y)
///
/// x +strafe right - strafe left
//  y +up -down
//  z +backward -forward
///
impl Camera {
    pub fn new(
        position: Point3<f64>,
        target: Point3<f64>,
        aspect_ratio: f64,
        fov: f64,
        aperture: f64,
        screen_window: Bounds<f64>,
        film: Arc<RwLock<Film>>,
    ) -> Camera {
        let image_size = {
            let film_lock = &film.read().unwrap();
            film_lock.image_size
        };

        let direction = target - position;
        let focal_distance = direction.magnitude();

        let world_up = Vector3::y();

        // Create a rotation and translation matrix from camera space to world space with the Y axis as up direction.
        let camera_to_world = Rotation3::face_towards(&direction, &world_up)
            .to_homogeneous()
            .append_translation(&position.coords);

        let camera_to_screen = perspective(fov, 0.01, 1000.0);

        /// To translate from screen space (x -1.0 to 1.0 and y -1.0 to 1.0) to raster space (based on the film resolution)
        /// we apply the following steps (bottom to top):
        /// - translate so the upper left corner is at the origin
        /// - scale by the screen window width and height, flip y coords in the process
        /// - scale by image resolution
        let screen_to_raster = Matrix4::new_nonuniform_scaling(&Vector3::new(
            image_size.x as f64,
            image_size.y as f64,
            1.0,
        )) * Matrix4::new_nonuniform_scaling(&Vector3::new(
            1.0 / (screen_window.p_max.x - screen_window.p_min.x), // 1 / (1 - (-1)) = 0.5
            1.0 / (screen_window.p_min.y - screen_window.p_max.y), // 1 / (-1 - 1) = -0.5, flip
            1.0,
        )) * Matrix4::new_translation(&Vector3::new(
            -screen_window.p_min.x, // -(-1) = 1
            -screen_window.p_max.y, // - (1) = -1
            0.0,
        ));

        let raster_to_screen = screen_to_raster.try_inverse().unwrap();
        let raster_to_camera = camera_to_screen.try_inverse().unwrap() * raster_to_screen;

        Camera {
            position,
            target,
            fov,
            aperture,
            focal_distance,
            film,
            camera_to_world,
            camera_to_screen,
            screen_to_raster,
            raster_to_screen,
            raster_to_camera,
        }
    }

    pub fn generate_ray(&self, sample: CameraSample) -> Ray {
        let mut origin = Point3::origin();

        let p_film = Point3::new(sample.p_film.x, sample.p_film.y, 0.0);
        let mut direction = self.raster_to_camera.transform_point(&p_film).coords;

        if self.aperture > 0.0 {
            let p_lens = self.aperture * crate::helpers::concentric_sample_disk();
            let ft = self.focal_distance / direction.z;

            let p_focus = ft * direction;
            origin = Point3::new(p_lens.x, p_lens.y, 0.0);
            direction = (p_focus - origin.coords).normalize()
        }

        let origin = self.camera_to_world.transform_point(&origin);
        let direction = self.camera_to_world.transform_vector(&direction);

        Ray {
            point: origin,
            direction: direction.normalize(),
        }
    }
}

pub fn perspective(fov_deg: f64, n: f64, f: f64) -> Matrix4<f64> {
    // Matrix4x4 persp(1, 0,           0,              0,
    //                 0, 1,           0,              0,
    //                 0, 0, f / (f - n), -f*n / (f - n),
    //                 0, 0,           1,              0);

    let perspective = Matrix4::new(
        1.0,
        0.0,
        0.0,
        0.0,
        0.0,
        1.0,
        0.0,
        0.0,
        0.0,
        0.0,
        f / (f - n),
        -f * n / (f - n),
        0.0,
        0.0,
        1.0,
        0.0,
    );

    let fov_rad = fov_deg * (PI / 180.0);

    let inv_tan_ang = 1.0 / (fov_rad / 2.0).tan();

    perspective * Matrix4::new_nonuniform_scaling(&Vector3::new(inv_tan_ang, inv_tan_ang, 1.0))
}

#[derive(Debug, Copy, Clone)]
pub struct CameraSample {
    pub p_lens: Point2<f64>,
    pub p_film: Point2<f64>,
}

#[cfg(test)]
mod tests {
    use std::f64::consts::PI;
    use std::sync::{Arc, RwLock};

    use approx::{assert_relative_eq, relative_eq};
    use nalgebra::{point, Perspective3, Point2, Point3, Vector2, Vector3};

    use crate::camera::{perspective, CameraSample};
    use crate::{Bounds, Camera, Film, FilterMethod};

    #[test]
    fn test() {
        let film = Arc::new(RwLock::new(Film::new(
            Vector2::new(100, 100),
            Vector2::new(100, 100),
            None,
            None,
            FilterMethod::None,
            1.0,
        )));

        let camera = Camera::new(
            Point3::new(0.0, 0.0, 1.0),
            Point3::new(0.0, 0.0, 0.0),
            1.0,
            90.0,
            0.0,
            Bounds {
                p_min: Point2::new(-1.0, -1.0),
                p_max: Point2::new(1.0, 1.0),
            },
            film.clone(),
        );

        let ray = camera.generate_ray(CameraSample {
            p_film: Point2::new(50.0, 50.0),
            p_lens: Point2::origin(),
        });

        assert_relative_eq!(0.0, ray.direction.x);
        assert_relative_eq!(0.0, ray.direction.y);
        assert_relative_eq!(-1.0, ray.direction.z);

        let ray_left = camera.generate_ray(CameraSample {
            p_film: Point2::new(0.0, 50.0),
            p_lens: Point2::origin(),
        });

        let ray_right = camera.generate_ray(CameraSample {
            p_film: Point2::new(100.0, 50.0),
            p_lens: Point2::origin(),
        });

        let angle = ray_left.direction.angle(&ray_right.direction);
        assert_relative_eq!(90.0, angle * 180.0 / PI, max_relative = 0.00001);

        let camera = Camera::new(
            Point3::new(0.0, 0.0, 1.0),
            Point3::new(0.0, 1.0, 0.0),
            1.0,
            90.0,
            0.0,
            Bounds {
                p_min: Point2::new(-1.0, -1.0),
                p_max: Point2::new(1.0, 1.0),
            },
            film,
        );

        let ray = camera.generate_ray(CameraSample {
            p_film: Point2::new(50.0, 50.0),
            p_lens: Point2::origin(),
        });

        let expected_direction = Vector3::new(0.0, 1.0, -1.0).normalize();
        assert_relative_eq!(expected_direction, ray.direction);

        let ray_left = camera.generate_ray(CameraSample {
            p_film: Point2::new(0.0, 50.0),
            p_lens: Point2::origin(),
        });

        let ray_right = camera.generate_ray(CameraSample {
            p_film: Point2::new(100.0, 50.0),
            p_lens: Point2::origin(),
        });

        let angle = ray_left.direction.angle(&ray_right.direction);
        assert_relative_eq!(90.0, angle * 180.0 / PI, max_relative = 0.00001);
    }
}
