use nalgebra::{Point3, Vector3, SimdPartialOrd};
use image;
use rand::*;
use scene::Scene;
use scene::lights::Light;
use scene::objects::Object;

use std::cmp;
use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::thread::JoinHandle;
use std::time::SystemTime;
use IMAGE_BUFFER;
use sampler::{Method, Sampler};
use scene::camera::Camera;

#[derive(Debug, Copy, Clone)]
pub struct Settings {
    pub thread_count: u32,
    pub bucket_width: u32,
    pub bucket_height: u32,
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
        bucket_width: 32,
        bucket_height: 32,
        depth_limit: 4,
        max_samples: 32,
        min_samples: 16,
        gamma: 1.2,
        sampler: Sampler::new(Method::Random, Camera::new(Point3::new(0.0,0.0,-1.0), Point3::new(0.0,0.0,0.0), 80.0), ::IMAGE_WIDTH, ::IMAGE_HEIGHT)
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

#[derive(Debug)]
struct WorkQueue {
    queue: Vec<Work>,
    buckets: usize,
}

#[derive(Debug, Copy, Clone)]
struct Work {
    x: u32,
    y: u32,
}

impl WorkQueue {
    fn new() -> WorkQueue {
        let mut queue = Vec::new();
        let mut buckets = 0;

        let settings = SETTINGS.read().unwrap();

        for x in 0..(::IMAGE_WIDTH as f32 / settings.bucket_width as f32).ceil() as u32 {
            for y in 0..(::IMAGE_HEIGHT as f32 / settings.bucket_height as f32).ceil() as u32 {
                queue.push(Work {
                    x: x * settings.bucket_width,
                    y: y * settings.bucket_height,
                });
                buckets += 1;
            }
        }

        let work_queue = WorkQueue { queue, buckets };

        work_queue
    }

    fn get_work(&mut self) -> Option<Work> {
        let mut rng = thread_rng();
        let len = self.queue.len();

        if len == 0 {
            return None;
        }

        Some(self.queue.remove(rng.gen_range(0, len)))
    }
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
) -> (Vec<JoinHandle<()>>, Vec<Sender<ThreadMessage>>) {
    let image_width = IMAGE_BUFFER.read().unwrap().width();
    let image_height = IMAGE_BUFFER.read().unwrap().height();
    let mut threads: Vec<JoinHandle<()>> = vec![];
    let mut thread_senders: Vec<Sender<ThreadMessage>> = vec![];
    let settings = SETTINGS.read().unwrap();

    let work_queue = Arc::new(Mutex::new(WorkQueue::new()));

    println!("Start render, w{} px, h{} px", image_width, image_height);
    println!("Settings: {:#?}", settings);

    // thread id is used to divide the work
    for thread_id in 0..settings.thread_count {
        let thread_scene = scene.clone();
        let work_queue = work_queue.clone();

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

            let settings = SETTINGS.read().unwrap();


            // use loop to split getting work and executing work. Else the lock
            // would be retained during the rendering process.
            loop {
                let work = work_queue.lock().unwrap().get_work(); // drop lock
                match work {
                    Some(work) => {
                        // prevent rounding error, cap at image size
                        let x_end = cmp::min(work.x + settings.bucket_width, image_width);
                        let y_end = cmp::min(work.y + settings.bucket_height, image_height);

                        // returns false if thread was requested to stop
                        if !render_work(work.x, x_end, work.y, y_end, &thread_scene, &thread_receiver)
                        {
                            return;
                        }

                        let mut stats = STATS.write().unwrap();

                        let rays_done = ((x_end - work.x) * (y_end - work.y)) * settings.max_samples;

                        stats.rays_done += rays_done;

                        if let Some(stats_thread) = stats.threads.get_mut(&thread_id) {
                            let duration =
                                stats_thread.start_time.elapsed().expect("Duration failed!");
                            let secs = duration.as_secs();
                            let sub_nanos = duration.subsec_nanos();
                            let nanos = secs * 1_000_000_000 + sub_nanos as u64;

                            stats_thread.rays_done += rays_done;
                            stats_thread.ns_per_ray = nanos as f64 / stats_thread.rays_done as f64;
                        }
                    }
                    None => break,
                }
            } // end of loop
        }); // end of thread

        threads.push(thread);
        thread_senders.push(thread_sender);
    }

    (threads, thread_senders)
}

fn render_work(x: u32, x_end: u32, y: u32, y_end: u32,
               scene: &Scene,
               thread_receiver: &Receiver<ThreadMessage>) -> bool
{
    let settings = SETTINGS.read().unwrap();

    for y in y..y_end {
        for x in x..x_end {
            match thread_receiver.try_recv() {
                Ok(thread_message) => {
                    if thread_message.exit {
                        println!("Stopping...");
                        return false;
                    }
                }
                Err(_err) => {}
            }

            let rays = settings.sampler.get_samples(
                settings.max_samples,
                x,
                y,
            );

            let mut pixel_color = Vector3::new(0.0, 0.0, 0.0);

            // Get the average pixel color using the samples.
            let mut samples = 0;
            // Get a minimum number of samples to avoid bailing
            // if the first couple of samples happen to be close to each other
            let adaptive_sampling = true;

            for ray in rays {
                samples += 1;
                let new_pixel_color = trace(ray, &scene, 1, 1.0).unwrap().simd_clamp(Vector3::new(0.0,0.0,0.0), Vector3::new(1.0,1.0,1.0));

                // If the new sample is almost exactly the same as the current average we stop
                // sampling.
                if adaptive_sampling && samples > settings.min_samples && (pixel_color - new_pixel_color).magnitude().abs() < (1.0 / 2000.0) {
                    pixel_color = pixel_color + ((new_pixel_color - pixel_color) / samples as f64);
                    break;
                } else {
                    pixel_color = pixel_color + ((new_pixel_color - pixel_color) / samples as f64);
                }
            }

            let pixel_color_rgba = image::Rgba([
                ((pixel_color.x.powf(1.0 / settings.gamma)) * 255.0) as u8,
                ((pixel_color.y.powf(1.0 / settings.gamma)) * 255.0) as u8,
                ((pixel_color.z.powf(1.0 / settings.gamma)) * 255.0) as u8,
                255,
            ]);

            IMAGE_BUFFER
                .write()
                .unwrap()
                .put_pixel(x, y, pixel_color_rgba);
        }
    }
    true
}

pub fn trace(ray: Ray, scene: &Scene, depth: u32, contribution: f64) -> Option<Vector3<f64>> {
    let settings = SETTINGS.read().unwrap();

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
        None => {
            return Some(scene.bg_color);
        }
        Some((intersection, object)) => {
            let point_of_intersection = ray.point + (ray.direction * intersection.distance);

            let mut color = Vector3::new(0.0, 0.0, 0.0);

            for material in object.get_materials() {
                if let Some(calculated_color) = material.get_surface_color(
                    ray,
                    &scene,
                    point_of_intersection,
                    intersection.normal,//object.get_normal(point_of_intersection),
                    depth,
                    contribution,
                ) {
                    color += calculated_color;
                }
            }

            return Some(color);
        }
    }
}

fn check_intersect_scene(ray: Ray, scene: &Scene) -> Option<(Intersection, &Box<dyn Object>)> {
    let mut closest: Option<(Intersection, &Box<dyn Object>)> = None;

    let bvh_ray = bvh::ray::Ray::new(
        bvh::nalgebra::Point3::new(ray.point.x as f32, ray.point.y as f32, ray.point.z as f32),
        bvh::nalgebra::Vector3::new(ray.direction.x as f32, ray.direction.y as f32, ray.direction.z as f32),
    );

    let hit_sphere_aabbs = scene.bvh.traverse(&bvh_ray, &scene.objects);
    for object in hit_sphere_aabbs {
        if let Some(intersection) = object.test_intersect(ray) {
            // If we found an intersection we check if the current
            // closest intersection is farther than the intersection
            // we found.
            match closest {
                None => closest = Some((intersection, &object)),
                Some((closest_dist, _)) => {
                    if intersection.distance < closest_dist.distance {
                        closest = Some((intersection, &object));
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
        bvh::nalgebra::Vector3::new(ray.direction.x as f32, ray.direction.y as f32, ray.direction.z as f32),
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

pub fn check_light_visible(position: Point3<f64>, scene: &Scene, light: Light) -> bool {
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
