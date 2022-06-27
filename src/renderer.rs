use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, RwLock};
use std::thread;
use std::thread::JoinHandle;
use std::time::SystemTime;

use lazy_static::lazy_static;
use nalgebra::{Point2, Point3, Vector3};

use crate::camera::Camera;
use crate::film::{Bucket, Film};
use crate::lights::LightIrradianceSample;
use crate::objects::ObjectTrait;
use crate::objects::{ArcObject, Object};
use crate::sampler::SobolSampler;
use crate::scene::Scene;
use crate::surface_interaction::SurfaceInteraction;
use crate::tracer::trace;

#[derive(Debug, Copy, Clone)]
pub struct Settings {
    pub thread_count: u32,
    pub depth_limit: u32,
    pub max_samples: u32,
}

pub struct DebugBuffer {
    pub width: u32,
    pub height: u32,
    pub buffer: Vec<f64>,
}

lazy_static! {
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
    pub static CURRENT_BOUNCE: RefCell<u32> = RefCell::new(0);
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
    pub p_film: Point2<f64>,
    pub normal: Vector3<f64>,
}

pub fn render(
    scene: Scene,
    settings: Settings,
    sampler: SobolSampler,
    camera: Arc<Camera>,
) -> (Vec<JoinHandle<()>>, Receiver<ThreadMessage>) {
    let scene = Arc::new(scene);
    let mut threads: Vec<JoinHandle<()>> = vec![];

    let (sender, receiver): (Sender<ThreadMessage>, Receiver<ThreadMessage>) = mpsc::channel();

    // thread id is used to divide the work
    for thread_id in 0..settings.thread_count {
        let thread_scene = scene.clone();
        let thread_camera = camera.clone();
        let mut thread_sampler = sampler.clone();

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
                let bucket = thread_camera.film.write().unwrap().get_bucket();

                match bucket {
                    Some(bucket) => {
                        // this lock should always work so do try_lock
                        let mut bucket_lock = bucket.try_lock().unwrap();

                        // returns false if thread was requested to stop
                        if !render_work(
                            &mut bucket_lock,
                            &thread_scene,
                            &settings,
                            &mut thread_sampler,
                            &thread_camera,
                        ) {
                            return;
                        }

                        samples_done += bucket_lock.samples.len();
                        thread_camera
                            .film
                            .read()
                            .unwrap()
                            .write_bucket_pixels(&mut bucket_lock);
                        // keep the write lock as short as possible
                        thread_camera
                            .film
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

fn render_work(
    bucket: &mut Bucket,
    scene: &Scene,
    settings: &Settings,
    sampler: &mut SobolSampler,
    camera: &Arc<Camera>,
) -> bool {
    for y in bucket.sample_bounds.p_min.y..bucket.sample_bounds.p_max.y {
        for x in bucket.sample_bounds.p_min.x..bucket.sample_bounds.p_max.x {
            CURRENT_X.with(|current_x| *current_x.borrow_mut() = x);
            CURRENT_Y.with(|current_y| *current_y.borrow_mut() = y);

            let mut sample_results: Vec<SampleResult> =
                Vec::with_capacity(settings.max_samples as usize);

            for _ in 0..settings.max_samples {
                let camera_sample = sampler.get_camera_sample(Point2::new(x as f64, y as f64));
                let ray = camera.generate_ray(camera_sample);
                debug_write_pixel((ray.direction * 0.5) + Vector3::repeat(0.5));
                let (radiance, normal) = trace(settings, ray, scene).unwrap();

                sample_results.push(SampleResult {
                    radiance,
                    p_film: camera_sample.p_film,
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
    let direction = (light_sample.point - interaction.point).normalize();
    let ray = Ray {
        point: interaction.point + (direction * 1e-9),
        direction,
    };

    let distance = nalgebra::distance(&interaction.point, &light_sample.point) - 1e-7;

    if check_intersect_scene_simple(ray, scene, distance) {
        return false;
    }

    true
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

pub fn debug_write_pixel_on_bounce(val: Vector3<f64>, bounce: u32) {
    if CURRENT_BOUNCE.with(|current_bounce| *current_bounce.borrow() != bounce) {
        return;
    }

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

pub fn debug_write_pixel_f64_on_bounce(val: f64, bounce: u32) {
    if CURRENT_BOUNCE.with(|current_bounce| *current_bounce.borrow() != bounce) {
        return;
    }

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
