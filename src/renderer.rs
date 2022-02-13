use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, RwLock};
use std::thread;
use std::thread::JoinHandle;
use std::time::SystemTime;

use nalgebra::{Point2, Point3, SimdPartialOrd, Vector3};

use film::{Bucket, Film};
use sampler::Sampler;
use scene::camera::Camera;
use scene::lights::Light;
use scene::objects::Object;
use scene::Scene;
use SamplerMethod;

#[derive(Debug, Copy, Clone)]
pub struct Settings {
    pub thread_count: u32,
    pub depth_limit: u32,
    pub max_samples: u32,
    pub min_samples: u32,
    pub gamma: f64,
    pub sampler: Sampler,
}

lazy_static! {
    pub static ref SETTINGS: RwLock<Settings> = {
        RwLock::new(Settings {
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
}

pub struct ThreadMessage {
    pub exit: bool,
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

pub fn render(
    scene: Arc<Scene>,
    film: Arc<RwLock<Film>>,
) -> (Vec<JoinHandle<()>>, Vec<Sender<ThreadMessage>>) {
    let mut threads: Vec<JoinHandle<()>> = vec![];
    let mut thread_senders: Vec<Sender<ThreadMessage>> = vec![];
    let settings = SETTINGS.read().unwrap();

    // thread id is used to divide the work
    for thread_id in 0..settings.thread_count {
        let thread_scene = scene.clone();
        let thread_film = film.clone();

        let (thread_sender, thread_receiver): (Sender<ThreadMessage>, Receiver<ThreadMessage>) =
            mpsc::channel();

        let thread = thread::spawn(move || {
            STATS.write().unwrap().threads.insert(
                thread_id,
                StatsThread {
                    start_time: SystemTime::now(),
                    rays_done: 0,
                    ns_per_ray: 0.0,
                },
            );

            loop {
                let bucket = thread_film.write().unwrap().get_bucket();

                match bucket {
                    Some(bucket) => {
                        // this lock should always work so do try_lock
                        let mut bucket_lock = bucket.try_lock().unwrap();

                        // returns false if thread was requested to stop
                        if !render_work(&mut bucket_lock, &thread_scene, &thread_receiver) {
                            return;
                        }

                        thread_film
                            .read()
                            .unwrap()
                            .write_bucket_pixels(&mut bucket_lock);
                        thread_film
                            .write()
                            .unwrap()
                            .merge_bucket_pixels_to_image_buffer(&mut bucket_lock);

                        // let mut stats = STATS.write().unwrap();
                        //
                        // let rays_done = ((x_end - work.x) * (y_end - work.y)) * settings.max_samples;
                        //
                        // stats.rays_done += rays_done;
                        //
                        // if let Some(stats_thread) = stats.threads.get_mut(&thread_id) {
                        //     let duration =
                        //         stats_thread.start_time.elapsed().expect("Duration failed!");
                        //     let secs = duration.as_secs();
                        //     let sub_nanos = duration.subsec_nanos();
                        //     let nanos = secs * 1_000_000_000 + sub_nanos as u64;
                        //
                        //     stats_thread.rays_done += rays_done;
                        //     stats_thread.ns_per_ray = nanos as f64 / stats_thread.rays_done as f64;
                        // }
                    }
                    None => {
                        break;
                    }
                }
            } // end of loop
        }); // end of thread

        threads.push(thread);
        thread_senders.push(thread_sender);
    }

    (threads, thread_senders)
}

fn render_work(
    bucket: &mut Bucket,
    scene: &Scene,
    thread_receiver: &Receiver<ThreadMessage>,
) -> bool {
    let settings = SETTINGS.read().unwrap();

    for y in bucket.sample_bounds.p_min.y..bucket.sample_bounds.p_max.y {
        match thread_receiver.try_recv() {
            Ok(thread_message) => {
                if thread_message.exit {
                    println!("Stopping...");
                    return false;
                }
            }
            Err(_err) => {}
        }

        for x in bucket.sample_bounds.p_min.x..bucket.sample_bounds.p_max.x {
            let samples = settings.sampler.get_samples(settings.max_samples, x, y);

            let mut sample_results: Vec<SampleResult> = Vec::with_capacity(samples.len());

            for sample in samples {
                // todo: remove clamp?
                let new_pixel_color = trace(&settings, sample.ray, scene, 1, 1.0)
                    .unwrap()
                    .simd_clamp(Vector3::new(0.0, 0.0, 0.0), Vector3::new(1.0, 1.0, 1.0));

                sample_results.push(SampleResult {
                    radiance: new_pixel_color,
                    pixel_location: sample.pixel_position,
                });
            }

            bucket.add_samples(&sample_results);
        }
    }

    true
}

pub fn trace(
    settings: &Settings,
    ray: Ray,
    scene: &Scene,
    depth: u32,
    contribution: f64,
) -> Option<Vector3<f64>> {
    // Early exit when max depth is reach or the contribution factor is too low.
    //
    // The contribution factor is checked here to force the user to provide one.
    if contribution < 0.01 {
        return None;
    }

    if depth > settings.depth_limit {
        return None;
    }

    let intersect = check_intersect_scene(ray, scene);

    match intersect {
        None => Some(scene.bg_color),
        Some((intersection, object)) => {
            let point_of_intersection = ray.point + (ray.direction * intersection.distance);

            let mut color = Vector3::new(0.0, 0.0, 0.0);

            for material in object.get_materials() {
                if let Some(calculated_color) = material.get_surface_color(
                    settings,
                    ray,
                    scene,
                    point_of_intersection,
                    intersection.normal, //object.get_normal(point_of_intersection),
                    depth,
                    contribution,
                ) {
                    color += calculated_color;
                }
            }

            Some(color)
        }
    }
}

fn check_intersect_scene(ray: Ray, scene: &Scene) -> Option<(Intersection, &Box<dyn Object>)> {
    let mut closest: Option<(Intersection, &Box<dyn Object>)> = None;

    let bvh_ray = bvh::ray::Ray::new(
        bvh::nalgebra::Point3::new(ray.point.x as f32, ray.point.y as f32, ray.point.z as f32),
        bvh::nalgebra::Vector3::new(
            ray.direction.x as f32,
            ray.direction.y as f32,
            ray.direction.z as f32,
        ),
    );

    let hit_sphere_aabbs = scene.bvh.traverse(&bvh_ray, &scene.objects);
    for object in hit_sphere_aabbs {
        if let Some(intersection) = object.test_intersect(ray) {
            // If we found an intersection we check if the current
            // closest intersection is farther than the intersection
            // we found.
            match closest {
                None => closest = Some((intersection, object)),
                Some((closest_dist, _)) => {
                    if intersection.distance < closest_dist.distance {
                        closest = Some((intersection, object));
                    }
                }
            }
        }
    }

    closest
}

fn check_intersect_scene_simple(ray: Ray, scene: &Scene, max_dist: f64) -> bool {
    let bvh_ray = bvh::ray::Ray::new(
        bvh::nalgebra::Point3::new(ray.point.x as f32, ray.point.y as f32, ray.point.z as f32),
        bvh::nalgebra::Vector3::new(
            ray.direction.x as f32,
            ray.direction.y as f32,
            ray.direction.z as f32,
        ),
    );

    let hit_sphere_aabbs = scene.bvh.traverse(&bvh_ray, &scene.objects);
    for object in hit_sphere_aabbs {
        if let Some(intersection) = object.test_intersect(ray) {
            // If we found an intersection we check if distance is less
            // than the max distance we want to check. If so -> exit with true
            if intersection.distance < max_dist {
                return true;
            }
        }
    }

    // No intersections found within distance
    false
}

pub fn check_light_visible(position: Point3<f64>, scene: &Scene, light: &Light) -> bool {
    let ray = Ray {
        point: position,
        direction: (light.position - position).try_normalize(1.0e-6).unwrap(),
    };

    let distance = nalgebra::distance(&position, &light.position);

    if check_intersect_scene_simple(ray, scene, distance) {
        return false;
    }

    true
}
