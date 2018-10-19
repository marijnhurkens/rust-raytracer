use cgmath::*;
use rand::{thread_rng, Rng};

pub fn vector_reflect(vec: Vector3<f64>, normal: Vector3<f64>) -> Vector3<f64> {
    vec - 2.0 * vec.dot(normal) * normal
}

pub fn get_random_in_unit_sphere() -> Vector3<f64> {
    let mut rng = thread_rng();

    let mut vec: Vector3<f64>;

    while {
        vec = 2.0 * Vector3::new(rng.gen::<f64>(), rng.gen::<f64>(), rng.gen::<f64>())
            - Vector3::new(1.0, 1.0, 1.0);

        vec.dot(vec) >= 1.0
    } {}

    vec
}
