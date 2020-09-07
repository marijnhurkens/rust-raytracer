use renderer::*;
use scene::camera::Camera;
use rand::*;
use sobol::Sobol;
use sobol::params::JoeKuoD6;
use std::f64::consts::PI;
use std::rc::Rc;
use std::iter::Take;
use std::borrow::Borrow;
use std::sync::Arc;

#[derive(Debug, Copy, Clone)]
pub enum Method {
    Random,
    Sobol,
}

#[derive(Debug, Copy, Clone)]
struct Sample {
    pub offset_x: f64,
    pub offset_y: f64,
}

#[derive(Debug, Copy, Clone)]
pub struct Sampler {
    method: Method,
    camera: Camera,
    image_width: u32,
    image_height: u32,
    half_width_radians: f64,
    half_height_radians: f64,
    pixel_width_radians: f64,
    pixel_height_radians: f64,
}

lazy_static! {
 pub static ref SOBOL: Vec<Vec<f64>> =  {
    let params = JoeKuoD6::standard();
    let seq = Sobol::<f64>::new(2, &params);

    seq.take(10000).collect()
 };
}

impl Sampler {
    pub fn new(
        method: Method,
        camera: Camera,
        image_width: u32,
        image_height: u32,
    ) -> Self {
        let aspect_ratio = image_width as f64 / image_height as f64;

        // fov = horizontal
        let fov_radians = PI * (camera.fov / 180.0); // for 60 degs: 60deg / 180deg = 0.33 * PI = 1.036 radians
        let half_width_radians = (fov_radians / 2.0).tan();
        let half_height_radians = half_width_radians * (1.0 / aspect_ratio);

        let plane_width = half_width_radians * 2.0;
        let plane_height = half_height_radians * 2.0;

        let pixel_width_radians = plane_width / (image_width - 1) as f64;
        let pixel_height_radians = plane_height / (image_height - 1) as f64;

        Sampler {
            method,
            camera,
            image_width,
            image_height,
            half_width_radians,
            half_height_radians,
            pixel_width_radians,
            pixel_height_radians,
        }
    }
    pub fn get_samples(
        &self,
        samples: u32,
        x: u32,
        y: u32,
    ) -> Vec<Ray>
    {
        match &self.method {
            Method::Random => self.get_random_samples(x, y, samples),
            Method::Sobol => self.get_sobol_samples(x, y, samples),
        }
    }


    fn get_random_samples(
        &self,
        x: u32,
        y: u32,
        samples: u32,
    ) -> Vec<Ray>
    {
        let mut rng = thread_rng();

        let mut rays = Vec::new();

        let half_pixel_width = &self.pixel_width_radians / 2.0;
        let half_pixel_height = &self.pixel_height_radians / 2.0;

        for _w in 0..samples {
            let sub_pixel_horizontal_offset = rng.gen_range(-half_pixel_width, half_pixel_width);
            let sub_pixel_vertical_offset = rng.gen_range(-half_pixel_height, half_pixel_height);

            let horizontal_offset = &self.camera.right
                * ((x as f64 * &self.pixel_width_radians) + sub_pixel_horizontal_offset - &self.half_width_radians);
            // pos_y needs to be negated because in the image the upper row is row 0,
            // in the 3d world the y axis descends when going down.
            let vertical_offset = &self.camera.up
                * ((-(y as f64) * &self.pixel_height_radians) + sub_pixel_vertical_offset + &self.half_height_radians);

            let ray = Ray {
                point: self.camera.position,
                direction: (&self.camera.forward + horizontal_offset + vertical_offset).normalize(),
            };

            rays.push(ray);
        }

        rays
    }

    fn get_sobol_samples(
        &self,
        x: u32,
        y: u32,
        samples: u32,
    ) -> Vec<Ray>
    {
        let mut rays = Vec::new();

        for point in SOBOL.iter().take(samples as usize) {
            let sub_pixel_horizontal_offset = (point[0] - 0.5) * &self.pixel_width_radians;
            let sub_pixel_vertical_offset = (point[1]- 0.5) * &self.pixel_height_radians;

            let horizontal_offset = &self.camera.right
                * ((x as f64 * &self.pixel_width_radians) + sub_pixel_horizontal_offset - &self.half_width_radians);
            // pos_y needs to be negated because in the image the upper row is row 0,
            // in the 3d world the y axis descends when going down.
            let vertical_offset = &self.camera.up
                * ((-(y as f64) * &self.pixel_height_radians) + sub_pixel_vertical_offset + &self.half_height_radians);

            let ray = Ray {
                point: self.camera.position,
                direction: (&self.camera.forward + horizontal_offset + vertical_offset).normalize(),
            };

            rays.push(ray);
        }

        rays
    }
}