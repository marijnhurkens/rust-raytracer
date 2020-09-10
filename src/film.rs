use image::{ImageBuffer, Rgba};
use nalgebra::{Point2, Vector2, Vector3};
use rand::*;
use renderer::{SampleResult};
use std::cmp;
use std::sync::{Mutex, Arc};
use std::f64::EPSILON;

pub enum FilterMethod {
    Box,
}

pub struct Bucket {
    pub start: Point2<u32>,
    pub end: Point2<u32>,
    pub samples: Vec<SampleResult>,
    pub finished: bool,
}

impl Bucket {
    pub fn add_samples(&mut self, samples: &Vec<SampleResult>) {
        self.samples.extend(samples);
    }

    pub fn finish(&mut self) {
        self.finished = true;
    }
}


#[derive(Debug, Copy, Clone)]
struct Pixel {
    pub sum_weight: f64,
    pub sum_radiance: Vector3<f64>,
}

pub struct Film {
    pub image_size: Vector2<u32>,
    film_size: u32,
    crop_start: Option<Point2<u32>>,
    crop_end: Option<Point2<u32>>,
    pixels: Vec<Pixel>,
    pub image_buffer: ImageBuffer<Rgba<u8>, Vec<u8>>,
    filter_radius: f64,
    filter_method: FilterMethod,
    filter_table: Vec<f64>,
    filter_table_size: usize,
    current_bucket: u32,
    buckets: Vec<Arc<Mutex<Bucket>>>,
}


impl Film {
    pub fn new(image_size: Vector2<u32>, bucket_size: Vector2<u32>, film_size: u32,
               crop_start: Option<Point2<u32>>, crop_end: Option<Point2<u32>>,
               filter_method: FilterMethod,
    ) -> Film
    {
        let mut pixels = vec!();

        for i in 0..(image_size.x * image_size.y) {
            pixels.push(Pixel {
                sum_weight: 0.0,
                sum_radiance: Vector3::new(0.0, 0.0, 0.0),
            });
        }

        let mut filter_table = vec!();
        let filter_table_size: usize = 16;
        let filter_radius = 1.2;
        let alpha = 0.5;

        for y in 0..filter_table_size {
            for x in 0..filter_table_size {
                let x_pos = (x as f64 + 0.5) * filter_radius / filter_table_size as f64;
                let y_pos = (y as f64 + 0.5) * filter_radius / filter_table_size as f64;
                let evaluate_point = Point2::new(x_pos, y_pos);

                filter_table.push(evaluate_gaussian(evaluate_point, filter_radius, alpha));
            }
        }

        Film {
            image_size,
            film_size,
            crop_start,
            crop_end,
            pixels,
            image_buffer: ImageBuffer::new(image_size.x, image_size.y),
            filter_radius,
            filter_method,
            filter_table,
            filter_table_size,
            current_bucket: 0,
            buckets: init_buckets(image_size, bucket_size),
        }
    }

    pub fn get_bucket(&mut self) -> Option<Arc<Mutex<Bucket>>>
    {
        let len = self.buckets.len() as u32;

        if self.current_bucket >= len {
            return None;
        }

        let bucket = self.buckets[self.current_bucket as usize].clone();
        self.current_bucket += 1;

        Some(bucket)
    }

    pub fn update_image_buffer(&mut self, bucket: &Bucket)
    {
        let samples = &bucket.samples;

        for sample in samples.iter()
        {
            // compute pixel influence raster
            let pixel_discrete = sample.pixel_location - Point2::new(0.5, 0.5);
            let x_min = (pixel_discrete.x - self.filter_radius).ceil() as i32;
            let y_min = (pixel_discrete.y - self.filter_radius).ceil() as i32;
            let x_max = (pixel_discrete.x + self.filter_radius).floor() as i32;
            let y_max = (pixel_discrete.y + self.filter_radius).floor() as i32;

            for x in x_min..=x_max
            {
                for y in y_min..=y_max
                {
                    if x < 0 || y < 0 || x >= self.image_size.x as i32 || y >= self.image_size.y as i32 {
                        continue;
                    }


                    // Float fx = std::abs((x - pFilmDiscrete.x) *
                    // invFilterRadius.x * filterTableSize);
                    // ifx[x - p0.x] = std::min((int)std::floor(fx), filterTableSize - 1);

                    let filter_index_x = ((x as f64 - pixel_discrete.x) * (1.0 / self.filter_radius) * self.filter_table_size as f64).abs().floor().min(self.filter_table_size as f64 - 1.0) as usize;
                    let filter_index_y = ((y as f64 - pixel_discrete.y) * (1.0 / self.filter_radius) * self.filter_table_size as f64).abs().floor().min(self.filter_table_size as f64 - 1.0) as usize;


                    let index = filter_index_y * self.filter_table_size + filter_index_x;

                    let filter_weight = self.filter_table[index];


                    let pixel_index = self.get_pixel_index(x as u32, y as u32);

                    self.pixels[pixel_index].sum_radiance += sample.radiance * filter_weight;
                    self.pixels[pixel_index].sum_weight += filter_weight;

                }
            }
        }

        for (index, pixel) in self.pixels.iter().enumerate() {
            let y = index as u32 / self.image_size.x;
            let x = index as u32 % self.image_size.x;

            if pixel.sum_weight < EPSILON {
                self.image_buffer.put_pixel(x as u32, y as u32, image::Rgba([
                    0,
                    0,
                    0,
                    255,
                ]));
            }

            let radiance = pixel.sum_radiance / pixel.sum_weight;
            //
            // let pixel_color_rgba = image::Rgba([
            //     ((radiance.x.powf(1.0 / 1.2)) * 255.0) as u8,
            //     ((radiance.y.powf(1.0 / 1.2)) * 255.0) as u8,
            //     ((radiance.z.powf(1.0 / 1.2)) * 255.0) as u8,
            //     255,
            // ]);

            let pixel_color_rgba = image::Rgba([
                (radiance.x * 255.0) as u8,
                (radiance.y * 255.0) as u8,
                (radiance.z * 255.0) as u8,
                255,
            ]);

            self.image_buffer.put_pixel(x as u32, y as u32, pixel_color_rgba);
        }

        // let mut pixel_color = Vector3::new(0.0, 0.0, 0.0);
        //
        // for sample in &sample_results {
        //     pixel_color += sample.radiance;
        // }
        //
        // pixel_color /= sample_results.len() as f64;

        // // If the new sample is almost exactly the same as the current average we stop
        // // sampling.
        // if adaptive_sampling && samples > settings.min_samples && (pixel_color - new_pixel_color).magnitude().abs() < (1.0 / 2000.0) {
        //     pixel_color = pixel_color + ((new_pixel_color - pixel_color) / samples as f64);
        //     break;
        // } else {
        //     pixel_color = pixel_color + ((new_pixel_color - pixel_color) / samples as f64);
        // }


        // let pixel_color_rgba = image::Rgba([
        //     ((pixel_color.x.powf(1.0 / 1.2)) * 255.0) as u8,
        //     ((pixel_color.y.powf(1.0 / 1.2)) * 255.0) as u8,
        //     ((pixel_color.z.powf(1.0 / 1.2)) * 255.0) as u8,
        //     255,
        // ]);
        //
        // self.image_buffer.put_pixel(sample_results[0].pixel_location.x as u32, sample_results[0].pixel_location.y as u32, pixel_color_rgba);
    }

    fn get_pixel_index(&self, x: u32, y: u32) -> usize
    {
        (x + self.image_size.x * y) as usize
    }
}

fn init_buckets(image_size: Vector2<u32>, bucket_size: Vector2<u32>) -> Vec<Arc<Mutex<Bucket>>>
{
    let mut buckets = Vec::new();

    for x in 0..(image_size.x as f32 / bucket_size.x as f32).ceil() as u32 {
        for y in 0..(image_size.y as f32 / bucket_size.y as f32).ceil() as u32 {
            let start = Point2::new(x * bucket_size.x, y * bucket_size.y);

            // prevent rounding error, cap at image size
            let x_end = cmp::min(start.x + bucket_size.x, image_size.x);
            let y_end = cmp::min(start.y + bucket_size.y, image_size.y);

            let end = Point2::new(x_end, y_end);
            buckets.push(Arc::new(Mutex::new(Bucket {
                start,
                end,
                samples: vec!(),
                finished: false,
            })));
        }
    }

    buckets
}

fn evaluate_gaussian(point: Point2<f64>, radius: f64, alpha: f64) -> f64
{
    let expv = (-alpha * radius * radius).exp();

    let x = ((-alpha * point.x * point.x).exp() - expv).max(0.0);
    let y = ((-alpha * point.y * point.y).exp() - expv).max(0.0);

    x * y
}