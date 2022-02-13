use renderer::*;
use scene::camera::Camera;
use rand::*;
use sobol::Sobol;
use sobol::params::JoeKuoD6;
use std::f64::consts::PI;
use nalgebra::Point2;

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

#[derive(Debug, Copy, Clone)]
pub struct Sampler {
    method: SamplerMethod,
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
        method: SamplerMethod,
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
    ) -> Vec<Sample>
    {
        match &self.method {
            SamplerMethod::Random => self.get_random_samples(x, y, samples),
            SamplerMethod::Sobol => self.get_sobol_samples(x, y, samples),
        }
    }


    fn get_random_samples(
        &self,
        x: u32,
        y: u32,
        sample_num: u32,
    ) -> Vec<Sample>
    {
        let mut rng = thread_rng();
        let mut samples = Vec::new();

        for _w in 0..sample_num {
            let sub_pixel_horizontal_offset = rng.gen_range(0.0, 1.0);
            let sub_pixel_vertical_offset = rng.gen_range(0.0, 1.0);
            let sub_pixel_horizontal_offset_radians = sub_pixel_horizontal_offset - 0.5 * &self.pixel_width_radians;
            let sub_pixel_vertical_offset_radians = sub_pixel_vertical_offset - 0.5 * &self.pixel_height_radians;
            
            let horizontal_offset = &self.camera.right
                * ((x as f64 * &self.pixel_width_radians) + sub_pixel_horizontal_offset_radians - &self.half_width_radians);
            // pos_y needs to be negated because in the image the upper row is row 0,
            // in the 3d world the y axis descends when going down.
            let vertical_offset = &self.camera.up
                * ((-(y as f64) * &self.pixel_height_radians) + sub_pixel_vertical_offset_radians + &self.half_height_radians);

            let sample = Sample {
                ray: Ray {
                    point: self.camera.position,
                    direction: (&self.camera.forward + horizontal_offset + vertical_offset).normalize(),
                },
                pixel_position: Point2::new(x as f64 + sub_pixel_horizontal_offset - 0.5, y as f64 + sub_pixel_vertical_offset - 0.5),
            };

            dbg!(sample);

            samples.push(sample);
        }

        samples
    }

    fn get_sobol_samples(
        &self,
        x: u32,
        y: u32,
        sample_num: u32,
    ) -> Vec<Sample>
    {
        let mut samples = Vec::new();

        for point in SOBOL.iter().take(sample_num as usize) {
            let sub_pixel_horizontal_offset = (point[0] - 0.5) * &self.pixel_width_radians;
            let sub_pixel_vertical_offset = (point[1] - 0.5) * &self.pixel_height_radians;

            let horizontal_offset = &self.camera.right
                * ((x as f64 * &self.pixel_width_radians) + sub_pixel_horizontal_offset - &self.half_width_radians);
            // y needs to be negated because in the image the upper row is row 0,
            // in the 3d world the y axis descends when going down.
            let vertical_offset = &self.camera.up
                * ((-(y as f64) * &self.pixel_height_radians) + sub_pixel_vertical_offset + &self.half_height_radians);

            let sample = Sample {
                ray: Ray {
                    point: self.camera.position,
                    direction: (&self.camera.forward + horizontal_offset + vertical_offset).normalize(),
                },
                pixel_position: Point2::new(x as f64 + point[0] - 0.5, y as f64 + point[1] - 0.5),
            };

            samples.push(sample);
        }

        samples
    }
}