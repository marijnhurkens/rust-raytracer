use image::{ImageBuffer, Rgba};
use nalgebra::{Point2, Vector2, Vector3};
use rand::*;
use renderer::{SampleResult};
use std::cmp;
use std::sync::{Mutex, Arc, RwLock, RwLockReadGuard};

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
    pub fn add_samples(&mut self, samples: Vec<SampleResult>) {
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
    filter_method: FilterMethod,
    filter_map: Vec<Vec<f64>>,
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

        let filter_map = vec!(
          vec!(0.1,0.2,0.1),
          vec!(0.2,1.0,0.2),
          vec!(0.1,0.2,0.1),
        );

        Film {
            image_size,
            film_size,
            crop_start,
            crop_end,
            pixels,
            image_buffer: ImageBuffer::new(image_size.x, image_size.y),
            filter_method,
            filter_map,
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
            let x = (sample.pixel_location.x + 0.5).floor() as u32;
            let y = (sample.pixel_location.y + 0.5).floor() as u32;

            for (filter_y, filter_row) in self.filter_map.iter().enumerate()
            {
                for (filter_x, filter_weight) in filter_row.iter().enumerate()
                {
                    let diff_x = (filter_x as i32 - 1);
                    let diff_y = (filter_y as i32 - 1);

                    let new_x = x as i32 + diff_x;
                    let new_y = y as i32 + diff_y;

                    if new_x < 0 || new_y < 0 || new_x >= self.image_size.x as i32 || new_y >= self.image_size.y as i32 {
                        continue;
                    }

                    let pixel_index = self.get_pixel_index(new_x as u32, new_y as u32);


                    self.pixels[pixel_index].sum_radiance += sample.radiance * *filter_weight;
                    self.pixels[pixel_index].sum_weight += filter_weight;
                }
            }
        }

        for (index, pixel) in self.pixels.iter().enumerate() {
            let y = index as u32 / self.image_size.x;
            let x = index as u32 % self.image_size.x;

            let radiance = pixel.sum_radiance / pixel.sum_weight;

            let pixel_color_rgba = image::Rgba([
                ((radiance.x.powf(1.0 / 1.2)) * 255.0) as u8,
                ((radiance.y.powf(1.0 / 1.2)) * 255.0) as u8,
                ((radiance.z.powf(1.0 / 1.2)) * 255.0) as u8,
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

