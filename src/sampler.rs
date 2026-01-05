use std::f64::consts::PI;
use std::sync::Arc;
use lazy_static::lazy_static;
use nalgebra::{Point2, Point3};
use rand::*;
use sobol::params::JoeKuoD6;
use sobol::Sobol;

use crate::camera::{Camera, CameraSample};
use crate::renderer::Ray;
use crate::surface_interaction::SurfaceInteraction;

#[derive(Debug, Copy, Clone)]
pub enum SamplerMethod {
    Random,
    Sobol,
}

impl SamplerMethod {
    pub fn from_str(str: &str) -> Option<SamplerMethod> {
        match str {
            "random" => Some(SamplerMethod::Random),
            "sobol" => Some(SamplerMethod::Sobol),
            _ => Some(SamplerMethod::Random),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Sample {
    pub pixel_position: Point2<f64>,
    pub ray: Ray,
}

#[derive(Clone)]
pub struct SobolSampler {
    params: Arc<JoeKuoD6>,
    generator: Sobol<f64>,
    current_sample_vec: Vec<f64>,
    scramble_vec: Vec<f64>,
    current_dim: usize,
}

impl SobolSampler {
    pub fn new() -> Self {
        let sobol_params = JoeKuoD6::standard();
        let generator = Sobol::<f64>::new_with_resolution(1000, &sobol_params, Some(10));

        let mut rng = rng();
        let scramble_vec: Vec<f64> = (0..1000).map(|_| rng.random()).collect();

        SobolSampler {
            params: Arc::new(sobol_params),
            generator,
            current_sample_vec: Vec::new(),
            scramble_vec,
            current_dim: 0,
        }
    }

    /// Resets the sampler sequence. Call this at the start of each pixel.
    pub fn reset(&mut self) {
        self.generator = Sobol::<f64>::new_with_resolution(1000, &*self.params, Some(10));
        self.current_sample_vec.clear();
        self.current_dim = 0;

        let mut rng = rng();
        self.scramble_vec = (0..1000).map(|_| rng.random()).collect();
    }

    /// Advances to the next sample in the sequence. Call this at the start of each sample loop.
    pub fn start_sample(&mut self) {
        // Fetch the next high-dimensional vector for this path
        if let Some(mut vec) = self.generator.next() {
            for (i, v) in vec.iter_mut().enumerate() {
                if i < self.scramble_vec.len() {
                    *v = (*v + self.scramble_vec[i]) % 1.0;
                }
            }
            self.current_sample_vec = vec;
        } else {
            panic!("Sample generator returned None");
        }
        self.current_dim = 0;
    }

    fn next_dim(&mut self) -> f64 {
        if self.current_dim < self.current_sample_vec.len() {
            let v = self.current_sample_vec[self.current_dim];
            self.current_dim += 1;
            v
        } else {
            panic!("Sample generator ran out of dimensions");
        }
    }

    pub fn get_1d(&mut self) -> f64 {
        self.next_dim()
    }

    pub fn get_2d(&mut self) -> Vec<f64> {
        vec![self.next_dim(), self.next_dim()]
    }

    pub fn get_3d(&mut self) -> Vec<f64> {
        vec![self.next_dim(), self.next_dim(), self.next_dim()]
    }

    pub fn get_2d_point(&mut self) -> Point2<f64> {
        Point2::new(self.next_dim(), self.next_dim())
    }

    pub fn get_3d_point(&mut self) -> Point3<f64> {
        Point3::new(self.next_dim(), self.next_dim(), self.next_dim())
    }

    pub fn get_camera_sample(&mut self, pixel_pos: Point2<f64>) -> CameraSample {
        // Use the first 2 dimensions for film position
        let u1 = self.next_dim();
        let u2 = self.next_dim();
        let p_film = pixel_pos + Point2::new(u1, u2).coords;

        // Use the next 2 dimensions for lens position
        let u3 = self.next_dim();
        let u4 = self.next_dim();

        CameraSample {
            p_lens: Point2::new(u3, u4),
            p_film,
        }
    }
}
