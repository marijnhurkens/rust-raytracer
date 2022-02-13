use std::cmp;
use std::sync::{Arc, Mutex};

use image::{ImageBuffer, Rgba};
use nalgebra::{Point2, Vector2, Vector3};

use helpers::Bounds;
use renderer::SampleResult;

#[derive(PartialEq)]
pub enum FilterMethod {
    None,
    Gaussian,
    Mitchell,
}

const GAUSSIAN_ALPHA: f64 = 1.5;

impl FilterMethod {
    pub fn from_str(str: &str) -> Option<FilterMethod> {
        match str {
            "gaussian" => Some(FilterMethod::Gaussian),
            "mitchell" => Some(FilterMethod::Mitchell),
            _ => Some(FilterMethod::None),
        }
    }
}

#[derive(Debug)]
pub struct Bucket {
    pub sample_bounds: Bounds<u32>,
    pub pixel_bounds: Bounds<u32>,
    pub samples: Vec<SampleResult>,
    pixels: Vec<Pixel>,
}

impl Bucket {
    pub fn add_samples(&mut self, samples: &[SampleResult]) {
        self.samples.extend(samples);
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Pixel {
    pub sum_weight: f64,
    pub sum_radiance: Vector3<f64>,
}

pub struct Film {
    pub image_size: Vector2<u32>,
    crop_start: Option<Point2<u32>>,
    crop_end: Option<Point2<u32>>,
    pub pixels: Vec<Pixel>,
    pub image_buffer: ImageBuffer<Rgba<u8>, Vec<u8>>,
    filter_radius: f64,
    filter_method: FilterMethod,
    filter_table: Vec<f64>,
    filter_table_size: usize,
    bucket_size: Vector2<u32>,
    current_bucket: u32,
    buckets: Vec<Arc<Mutex<Bucket>>>,
}

impl Film {
    pub fn new(
        image_size: Vector2<u32>,
        bucket_size: Vector2<u32>,
        crop_start: Option<Point2<u32>>,
        crop_end: Option<Point2<u32>>,
        filter_method: FilterMethod,
        filter_radius: f64,
    ) -> Film {
        let mut pixels = vec![];

        for _ in 0..(image_size.x * image_size.y) {
            pixels.push(Pixel {
                sum_weight: 0.0,
                sum_radiance: Vector3::new(0.0, 0.0, 0.0),
            });
        }

        let mut filter_table = vec![];
        let filter_table_size: usize = 16;

        for y in 0..filter_table_size {
            for x in 0..filter_table_size {
                let x_pos = (x as f64 + 0.5) * filter_radius / filter_table_size as f64;
                let y_pos = (y as f64 + 0.5) * filter_radius / filter_table_size as f64;
                let evaluate_point = Point2::new(x_pos, y_pos);

                match filter_method {
                    FilterMethod::Gaussian => filter_table.push(evaluate_gaussian(
                        evaluate_point,
                        filter_radius,
                        GAUSSIAN_ALPHA,
                    )),
                    FilterMethod::Mitchell => {
                        filter_table.push(evaluate_mitchell(evaluate_point, filter_radius))
                    }
                    FilterMethod::None => {}
                }
            }
        }

        let mut film = Film {
            image_size,
            crop_start,
            crop_end,
            pixels,
            image_buffer: ImageBuffer::new(image_size.x, image_size.y),
            filter_radius,
            filter_method,
            filter_table,
            filter_table_size,
            current_bucket: 0,
            bucket_size,
            buckets: vec![],
        };

        film.init_buckets();

        film
    }

    pub fn get_bucket(&mut self) -> Option<Arc<Mutex<Bucket>>> {
        let len = self.buckets.len() as u32;

        if self.current_bucket >= len {
            println!("No buckets left.");
            return None;
        }

        let bucket = self.buckets[self.current_bucket as usize].clone();
        self.current_bucket += 1;

        println!("Handing out bucket {}", self.current_bucket);

        Some(bucket)
    }

    pub fn write_bucket_pixels(&self, bucket: &mut Bucket) {
        let samples = &bucket.samples;

        for sample in samples.iter() {
            // compute pixel influence raster
            let pixel_discrete = sample.pixel_location - Point2::new(0.5, 0.5);

            if self.filter_method == FilterMethod::None {
                let bucket_x = pixel_discrete.x as u32 - bucket.pixel_bounds.p_min.x;
                let bucket_y = pixel_discrete.y as u32 - bucket.pixel_bounds.p_min.y;
                let pixel_index =
                    (bucket_x as u32 + bucket.pixel_bounds.vector().x * bucket_y as u32) as usize;
                bucket.pixels[pixel_index].sum_radiance += sample.radiance;
                bucket.pixels[pixel_index].sum_weight += 1.0;
                continue;
            }

            let x_min = (pixel_discrete.x - self.filter_radius).ceil() as i32;
            let y_min = (pixel_discrete.y - self.filter_radius).ceil() as i32;
            let x_max = (pixel_discrete.x + self.filter_radius).floor() as i32;
            let y_max = (pixel_discrete.y + self.filter_radius).floor() as i32;

            for x in x_min..=x_max {
                for y in y_min..=y_max {
                    if x < 0
                        || y < 0
                        || x >= self.image_size.x as i32
                        || y >= self.image_size.y as i32
                    {
                        continue;
                    }

                    let filter_index_x = ((x as f64 - pixel_discrete.x)
                        * (1.0 / self.filter_radius)
                        * self.filter_table_size as f64)
                        .abs()
                        .floor()
                        .min(self.filter_table_size as f64 - 1.0)
                        as usize;
                    let filter_index_y = ((y as f64 - pixel_discrete.y)
                        * (1.0 / self.filter_radius)
                        * self.filter_table_size as f64)
                        .abs()
                        .floor()
                        .min(self.filter_table_size as f64 - 1.0)
                        as usize;

                    let filter_index = filter_index_y * self.filter_table_size + filter_index_x;

                    let filter_weight = self.filter_table[filter_index];

                    // convert to local bucket coordinates
                    let bucket_x = x as u32 - bucket.pixel_bounds.p_min.x;
                    let bucket_y = y as u32 - bucket.pixel_bounds.p_min.y;
                    let pixel_index = (bucket_x as u32
                        + bucket.pixel_bounds.vector().x * bucket_y as u32)
                        as usize;

                    bucket.pixels[pixel_index].sum_radiance += sample.radiance * filter_weight;
                    bucket.pixels[pixel_index].sum_weight += filter_weight;
                }
            }
        }

        bucket.samples = vec![];
    }

    pub fn merge_bucket_pixels_to_image_buffer(&mut self, bucket: &mut Bucket) {
        for (index, pixel) in bucket.pixels.iter().enumerate() {
            let x = (index as u32 % bucket.pixel_bounds.vector().x) + bucket.pixel_bounds.p_min.x;
            let y = (index as u32 / bucket.pixel_bounds.vector().x) + bucket.pixel_bounds.p_min.y;

            let film_pixel_index = self.get_pixel_index(x, y);

            self.pixels[film_pixel_index].sum_weight += pixel.sum_weight;
            self.pixels[film_pixel_index].sum_radiance += pixel.sum_radiance;

            if self.pixels[film_pixel_index].sum_weight < f64::EPSILON {
                self.image_buffer
                    .put_pixel(x as u32, y as u32, image::Rgba([0, 0, 0, 255]));
            }

            let radiance = self.pixels[film_pixel_index].sum_radiance
                / self.pixels[film_pixel_index].sum_weight;

            let pixel_color_rgba = image::Rgba([
                ((radiance.x) * 255.0) as u8,
                ((radiance.y) * 255.0) as u8,
                ((radiance.z) * 255.0) as u8,
                255,
            ]);

            self.image_buffer
                .put_pixel(x as u32, y as u32, pixel_color_rgba);
        }
    }

    fn get_pixel_index(&self, x: u32, y: u32) -> usize {
        (x + self.image_size.x * y) as usize
    }

    fn init_buckets(&mut self) {
        let mut buckets = Vec::new();
        let bucket_size = self.bucket_size;
        let image_size = self.image_size;
        let filter_radius = self.filter_radius;

        let (render_width, render_height) =
            if let (Some(crop_start), Some(crop_end)) = (self.crop_start, self.crop_end) {
                (crop_end.x - crop_start.x, crop_end.y - crop_start.y)
            } else {
                (image_size.x, image_size.y)
            };

        for x in 0..(render_width as f64 / bucket_size.x as f64).ceil() as u32 {
            for y in 0..(render_height as f64 / bucket_size.y as f64).ceil() as u32 {
                let start = if let Some(crop_start) = self.crop_start {
                    Point2::new(x * bucket_size.x, y * bucket_size.y) + crop_start.coords
                } else {
                    Point2::new(x * bucket_size.x, y * bucket_size.y)
                };

                // prevent rounding error, cap at image size
                let x_end = cmp::min(start.x + bucket_size.x, image_size.x);
                let y_end = cmp::min(start.y + bucket_size.y, image_size.y);

                let end = Point2::new(x_end, y_end);

                let sample_bounds = Bounds {
                    p_min: start,
                    p_max: end,
                };

                let pixel_bounds_start_x = (start.x as f64 - 0.5 - filter_radius).floor() as u32;
                let pixel_bounds_start_y = (start.y as f64 - 0.5 - filter_radius).floor() as u32;

                let pixel_bounds_end_x =
                    ((end.x as f64 + 0.5 + filter_radius).ceil() as u32).min(image_size.x);
                let pixel_bounds_end_y =
                    ((end.y as f64 + 0.5 + filter_radius).ceil() as u32).min(image_size.y);

                let pixel_bounds = Bounds {
                    p_min: Point2::new(pixel_bounds_start_x, pixel_bounds_start_y),
                    p_max: Point2::new(pixel_bounds_end_x, pixel_bounds_end_y),
                };

                let mut pixels = vec![];

                for _ in 0..pixel_bounds.area() {
                    pixels.push(Pixel {
                        sum_weight: 0.0,
                        sum_radiance: Vector3::new(0.0, 0.0, 0.0),
                    });
                }

                buckets.push(Arc::new(Mutex::new(Bucket {
                    sample_bounds,
                    pixel_bounds,
                    samples: vec![],
                    pixels,
                })));
            }
        }

        self.buckets = buckets;
    }
}

fn evaluate_gaussian(point: Point2<f64>, radius: f64, alpha: f64) -> f64 {
    let expv = (-alpha * radius * radius).exp();

    let x = ((-alpha * point.x * point.x).exp() - expv).max(0.0);
    let y = ((-alpha * point.y * point.y).exp() - expv).max(0.0);

    x * y
}

fn evaluate_mitchell(point: Point2<f64>, filter_radius: f64) -> f64 {
    let inv_radius = 1.0 / filter_radius;
    evaluate_mitchell_1d(point.x * inv_radius) * evaluate_mitchell_1d(point.y * inv_radius)
}

fn evaluate_mitchell_1d(input: f64) -> f64 {
    let _b = 0.33;
    let _c = 0.33;

    let x = (2.0 * input).abs();

    if x > 1.0 {
        return ((-_b - 6.0 * _c) * x * x * x
            + (6.0 * _b + 30.0 * _c) * x * x
            + (-12.0 * _b - 48.0 * _c) * x
            + (8.0 * _b + 24.0 * _c))
            * (1.0 / 6.0);
    }

    ((12.0 - 9.0 * _b - 6.0 * _c) * x * x * x
        + (-18.0 + 12.0 * _b + 6.0 * _c) * x * x
        + (6.0 - 2.0 * _b))
        * (1.0 / 6.0)
}
