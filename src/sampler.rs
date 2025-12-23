use std::f64::consts::PI;

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
    sobol_1d: Sobol<f64>,
    sobol_2d: Sobol<f64>,
    sobol_3d: Sobol<f64>,
}

impl SobolSampler {
    pub fn new() -> Self {
        let sobol_params = JoeKuoD6::standard();
        let sobol_1d = Sobol::<f64>::new(1, &sobol_params);
        let sobol_2d = Sobol::<f64>::new(2, &sobol_params);
        let sobol_3d = Sobol::<f64>::new(3, &sobol_params);

        SobolSampler {
            sobol_1d,
            sobol_2d,
            sobol_3d,
        }
    }

    pub fn get_1d(&mut self) -> f64 {
        self.sobol_1d.next().unwrap().pop().unwrap()
    }

    pub fn get_2d(&mut self) -> Vec<f64> {
        self.sobol_2d.next().unwrap()
    }

    pub fn get_3d(&mut self) -> Vec<f64> {
        self.sobol_3d.next().unwrap()
    }
    
    pub fn get_2d_point(&mut self) -> Point2<f64> {
        Point2::from_slice(&self.sobol_2d.next().unwrap())
    }
    
    pub fn get_3d_point(&mut self) -> Point3<f64> {
        Point3::from_slice(&self.sobol_3d.next().unwrap())
    }

    pub fn get_camera_sample(&mut self, pixel_pos: Point2<f64>) -> CameraSample {
        let p_film = pixel_pos + Point2::from_slice(&self.sobol_2d.next().unwrap()).coords;

        CameraSample {
            p_lens: Point2::from_slice(&self.sobol_2d.next().unwrap()),
            p_film,
        }
    }
}
