use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, RwLock};
use std::thread;
use std::thread::JoinHandle;
use std::time::SystemTime;

use nalgebra::{Point2, Point3, Vector3};

use camera::Camera;
use film::{Bucket, Film};
use lights::LightIrradianceSample;
use objects::{ArcObject, Object};
use objects::ObjectTrait;
use sampler::Sampler;
use scene::Scene;
use surface_interaction::SurfaceInteraction;
use tracer::trace;
use SamplerMethod;

#[derive(Debug, Copy, Clone)]
pub struct Settings {
    pub denoise: bool,
    pub thread_count: u32,
    pub depth_limit: u32,
    pub max_samples: u32,
    pub min_samples: u32,
    pub gamma: f64,
    pub sampler: Sampler,
}

pub struct DebugBuffer {
    pub width: u32,
    pub height: u32,
    pub buffer: Vec<f64>,
}

lazy_static! {
    pub static ref SETTINGS: RwLock<Settings> = {
        RwLock::new(Settings {
            denoise: false,
            thread_count: 8,
            depth_limit: 4,
            max_samples: 32,
            min_samples: 16,
            gamma: 1.2,
            sampler: Sampler::new(
                SamplerMethod::Random,
                Camera::new(
                    Point3::new(0.0, 0.0, -1.0),
                    Point3::new(0.0, 0.0, 0.0),
                    80.0,
                ),
                80,
                80,
            ),
        })
    };
    pub static ref DEBUG_BUFFER: RwLock<DebugBuffer> = {
        RwLock::new(DebugBuffer {
            width: 0,
            height: 0,
            buffer: vec![],
        })
    };
}

thread_local! {
    static CURRENT_X: RefCell<u32> = RefCell::new(0);
    static CURRENT_Y: RefCell<u32> = RefCell::new(0);
}

pub struct ThreadMessage {
    pub exit: bool,
    pub finished: bool,
}

#[derive(Debug)]
pub struct Stats {
    pub rays_done: u32,
    pub threads: HashMap<u32, StatsThread>,
}

#[derive(Copy, Clone, Debug)]
pub struct StatsThread {
    pub start_time: SystemTime,
    pub ns_per_ray: f64,
    pub rays_done: u32,
}

#[derive(Debug, Copy, Clone)]
pub struct Ray {
    pub point: Point3<f64>,
    pub direction: Vector3<f64>,
}

#[derive(Debug, Copy, Clone)]
pub struct Intersection {
    pub distance: f64,
    pub normal: Vector3<f64>,
}

#[derive(Debug, Copy, Clone)]
pub struct SampleResult {
    pub radiance: Vector3<f64>,
    pub pixel_location: Point2<f64>,
    pub normal: Vector3<f64>,
}

lazy_static! {
    pub static ref STATS: RwLock<Stats> = {
        let stats = Stats {
            rays_done: 0,
            threads: HashMap::new(),
        };

        RwLock::new(stats)
    };
}

pub fn debug_write_pixel(val: Vector3<f64>) {
    let mut buffer = DEBUG_BUFFER.write().unwrap();
    let mut index = 0;
    CURRENT_Y.with(|y| {
        CURRENT_X.with(|x| {
            index = *y.borrow() * buffer.height + *x.borrow();
            index *= 3;
        });
    });
    buffer.buffer[index as usize] = val.x;
    buffer.buffer[(index + 1) as usize] = val.y;
    buffer.buffer[(index + 2) as usize] = val.z;
}

pub fn debug_write_pixel_f64(val: f64) {
    let mut buffer = DEBUG_BUFFER.write().unwrap();
    let mut index = 0;
    CURRENT_Y.with(|y| {
        CURRENT_X.with(|x| {
            index = *y.borrow() * buffer.height + *x.borrow();
            index *= 3;
        });
    });
    buffer.buffer[index as usize] = val;
    buffer.buffer[(index + 1) as usize] = val;
    buffer.buffer[(index + 2) as usize] = val;
}

pub fn render(
    scene: Scene,
    film: Arc<RwLock<Film>>,
) -> (Vec<JoinHandle<()>>, Receiver<ThreadMessage>) {
    let scene = Arc::new(scene);
    let mut threads: Vec<JoinHandle<()>> = vec![];
    let settings = SETTINGS.read().unwrap();

    let (sender, receiver): (Sender<ThreadMessage>, Receiver<ThreadMessage>) = mpsc::channel();

    // thread id is used to divide the work
    for thread_id in 0..settings.thread_count {
        let thread_scene = scene.clone();
        let thread_film = film.clone();

        let thread_sender = sender.clone();

        let thread = thread::spawn(move || {
            STATS.write().unwrap().threads.insert(
                thread_id,
                StatsThread {
                    start_time: SystemTime::now(),
                    rays_done: 0,
                    ns_per_ray: 0.0,
                },
            );

            let start_time = SystemTime::now();
            let mut samples_done = 0;

            loop {
                let bucket = thread_film.write().unwrap().get_bucket();

                match bucket {
                    Some(bucket) => {
                        // this lock should always work so do try_lock
                        let mut bucket_lock = bucket.try_lock().unwrap();

                        // returns false if thread was requested to stop
                        if !render_work(&mut bucket_lock, &thread_scene) {
                            return;
                        }

                        samples_done += bucket_lock.samples.len();
                        thread_film
                            .read()
                            .unwrap()
                            .write_bucket_pixels(&mut bucket_lock);
                        // keep the write lock as short as possible
                        thread_film
                            .write()
                            .unwrap()
                            .merge_bucket_pixels_to_image_buffer(&mut bucket_lock);
                    }
                    None => {
                        break;
                    }
                }
            } // end of loop

            let duration = start_time.elapsed().expect("Duration failed!");
            let secs = duration.as_secs();
            let sub_nanos = duration.subsec_nanos();
            let nano_seconds = secs * 1_000_000_000 + sub_nanos as u64;
            let nano_seconds_per_sample = (nano_seconds as f64 / samples_done as f64).round();

            println!("Thread {thread_id} done, {samples_done} rendered, {nano_seconds_per_sample} ns per sample");

            thread_sender
                .send(ThreadMessage {
                    exit: false,
                    finished: true,
                })
                .unwrap();
        }); // end of thread

        threads.push(thread);
    }

    (threads, receiver)
}

fn render_work(bucket: &mut Bucket, scene: &Scene) -> bool {
    let settings = SETTINGS.read().unwrap();

    for y in bucket.sample_bounds.p_min.y..bucket.sample_bounds.p_max.y {
        for x in bucket.sample_bounds.p_min.x..bucket.sample_bounds.p_max.x {
            CURRENT_X.with(|current_x| *current_x.borrow_mut() = x);
            CURRENT_Y.with(|current_y| *current_y.borrow_mut() = y);

            let samples = settings.sampler.get_samples(settings.max_samples, x, y);

            let mut sample_results: Vec<SampleResult> = Vec::with_capacity(samples.len());

            for sample in samples {
                let (mut new_pixel_color, normal) = trace(&settings, sample.ray, scene).unwrap();

                // new_pixel_color = new_pixel_color
                //     .simd_clamp(Vector3::new(0.0, 0.0, 0.0), Vector3::new(1.0, 1.0, 1.0));

                sample_results.push(SampleResult {
                    radiance: new_pixel_color,
                    pixel_location: sample.pixel_position,
                    normal,
                });
            }

            bucket.add_samples(&sample_results);
        }
    }

    true
}

pub fn check_intersect_scene(ray: Ray, scene: &Scene) -> Option<(SurfaceInteraction, &ArcObject)> {
    let mut closest_hit: Option<(SurfaceInteraction, &ArcObject)> = None;
    let mut closest_distance = f64::MAX;

    let bvh_ray = bvh::ray::Ray::new(
        bvh::Point3::new(ray.point.x as f32, ray.point.y as f32, ray.point.z as f32),
        bvh::Vector3::new(
            ray.direction.x as f32,
            ray.direction.y as f32,
            ray.direction.z as f32,
        ),
    );

    let hit_sphere_aabbs = scene.bvh.traverse_iterator(&bvh_ray, &scene.objects);
    for object in hit_sphere_aabbs {
        if let Some((distance, intersection)) = object.test_intersect(ray) {
            // If we found an intersection we check if the current
            // closest intersection is farther than the intersection
            // we found.

            match closest_hit {
                None => {
                    closest_hit = Some((intersection, object));
                    closest_distance = distance;
                }
                Some((_, _)) => {
                    if distance < closest_distance {
                        closest_hit = Some((intersection, object));
                        closest_distance = distance;
                    }
                }
            }
        }
    }

    closest_hit
}

pub fn check_intersect_scene_simple(ray: Ray, scene: &Scene, max_dist: f64) -> bool {
    let bvh_ray = bvh::ray::Ray::new(
        bvh::Point3::new(ray.point.x as f32, ray.point.y as f32, ray.point.z as f32),
        bvh::Vector3::new(
            ray.direction.x as f32,
            ray.direction.y as f32,
            ray.direction.z as f32,
        ),
    );

    scene
        .bvh
        .traverse_iterator(&bvh_ray, &scene.objects)
        .any(|object| {
            if let Some((distance, _)) = object.test_intersect(ray) {
                // If we found an intersection we check if distance is less
                // than the max distance we want to check. If so -> exit with true
                if distance < max_dist {
                    return true;
                }
            }

            false
        })
}

pub fn check_light_visible(
    interaction: &SurfaceInteraction,
    scene: &Scene,
    light_sample: &LightIrradianceSample,
) -> bool {
    let ray = Ray {
        point: interaction.point,
        direction: (light_sample.point - interaction.point).normalize(),
    };

    let distance = nalgebra::distance(&interaction.point, &light_sample.point);

    if check_intersect_scene_simple(ray, scene, distance) {
        return false;
    }

    true
}