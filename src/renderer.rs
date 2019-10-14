use bvh::nalgebra::{Point3, Vector3};
use camera::Camera;
use image;
use rand::*;
use scene::{Light, Object, Scene};
use std::cmp;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::thread::JoinHandle;
use std::time::SystemTime;
use IMAGE_BUFFER;

const MAX_DEPTH: u32 = 6;
const GAMMA: f64 = 1.2; // ??? this would normally decode from srgb to linear space, looks fine though

#[derive(Debug, Copy, Clone)]
pub struct Settings {
    pub thread_count: u32,
    pub bucket_width: u32,
    pub bucket_height: u32,
    pub depth_limit: u32,
    pub samples: u32,
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

#[derive(Debug)]
struct WorkQueue {
    queue: Vec<Work>,
}

#[derive(Debug, Copy, Clone)]
struct Work {
    x: u32,
    y: u32,
}

impl WorkQueue {
    fn new(settings: Settings) -> WorkQueue {
        let mut queue = Vec::new();

        for x in 0..(::IMAGE_WIDTH as f32 / settings.bucket_width as f32).ceil() as u32 {
            for y in 0..(::IMAGE_HEIGHT as f32 / settings.bucket_height as f32).ceil() as u32 {
                queue.push(Work {
                    x: x * settings.bucket_width,
                    y: y * settings.bucket_height,
                });
            }
        }

        let work_queue = WorkQueue { queue: queue };

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

pub fn render(camera: Camera, scene: Arc<Scene>, settings: Settings) -> Vec<JoinHandle<()>> {
    let image_width = IMAGE_BUFFER.read().unwrap().width();
    let image_height = IMAGE_BUFFER.read().unwrap().height();
    let mut threads: Vec<JoinHandle<()>> = vec![];
    let work_queue = Arc::new(Mutex::new(WorkQueue::new(settings)));

    println!("Start render, w{} px, h{} px", image_width, image_height);
    //println!("Camera {:#?}", camera);

    // thread id is used to divide the work
    for thread_id in 0..settings.thread_count {
        let thread_scene = scene.clone();
        let work_queue = work_queue.clone();

        let thread = thread::spawn(move || {
            STATS.write().unwrap().threads.insert(
                thread_id,
                StatsThread {
                    start_time: SystemTime::now(),
                    rays_done: 0,
                    ns_per_ray: 0.0,
                },
            );

            // use loop to split getting work and executing work. Else the lock
            // would be retained during execution.
            loop {
                let work = work_queue.lock().unwrap().get_work(); // drop lock
                match work {
                    Some(work) => {
                        // prevent rounding error, cap at image size
                        let x_end = cmp::min(work.x + settings.bucket_width, image_width);
                        let y_end = cmp::min(work.y + settings.bucket_height, image_height);

                        for y in work.y..y_end {
                            for x in work.x..x_end {
                                let rays =
                                    get_rays_at(camera, image_width, image_height, x, y, settings.samples)
                                        .unwrap();
                                let rays_len = rays.len();

                                let mut pixel_color = Vector3::new(0.0, 0.0, 0.0);

                                // Get the average pixel color using the samples.
                                for ray in rays {
                                    pixel_color += trace(ray, &thread_scene, 1, 1.0).unwrap();
                                }

                                pixel_color /= rays_len as f64;

                                let pixel_color_rgba = image::Rgba([
                                    ((pixel_color.x.powf(1.0 / GAMMA)) * 255.0) as u8,
                                    ((pixel_color.y.powf(1.0 / GAMMA)) * 255.0) as u8,
                                    ((pixel_color.z.powf(1.0 / GAMMA)) * 255.0) as u8,
                                    255,
                                ]);

                                IMAGE_BUFFER
                                    .write()
                                    .unwrap()
                                    .put_pixel(x, y, pixel_color_rgba);
                            }
                        }

                        let mut stats = STATS.write().unwrap();

                        let rays_done = ((x_end - work.x) * (y_end - work.y)) * settings.samples;

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
    }

    threads
}

fn get_rays_at(
    camera: Camera,
    width: u32,
    height: u32,
    pos_x: u32,
    pos_y: u32,
    samples: u32,
) -> Result<Vec<Ray>, &'static str> {
    use std::f64::consts::PI;
    let mut rng = thread_rng();

    // let pos_x = pos % width;
    // let pos_y = pos / width;

    if pos_y >= height {
        return Err("Position exceeds number of pixels.");
    }

    let aspect_ratio = width as f64 / height as f64;

    // fov = horizontal
    let fov_radians = PI * (camera.fov / 180.0); // for 60 degs: 60deg / 180deg = 0.33 * PI = 1.036 radians
    let half_width = (fov_radians / 2.0).tan();
    let half_height = half_width * (1.0 / aspect_ratio);

    let plane_width = half_width * 2.0;
    let plane_height = half_height * 2.0;

    let pixel_width = plane_width / (width - 1) as f64;
    let pixel_height = plane_height / (height - 1) as f64;

    let half_pixel_width = pixel_width / 2.0;
    let half_pixel_height = pixel_height / 2.0;

    let mut rays = Vec::new();

    for _w in 0..samples {
        let sub_pixel_horizontal_offset = rng.gen_range(-half_pixel_width, half_pixel_width);
        let sub_pixel_vertical_offset = rng.gen_range(-half_pixel_height, half_pixel_height);

        let horizontal_offset = camera.right
            * ((pos_x as f64 * pixel_width) + sub_pixel_horizontal_offset - half_width);
        // pos_y needs to be negated because in the image the upper row is row 0,
        // in the 3d world the y axis descends when going down.
        let vertical_offset = camera.up
            * ((-(pos_y as f64) * pixel_height) + sub_pixel_vertical_offset + half_height);

        let ray = Ray {
            point: camera.position,
            direction: (camera.forward + horizontal_offset + vertical_offset).normalize(),
        };

        rays.push(ray);
    }

    Ok(rays)
}

pub fn trace(ray: Ray, scene: &Scene, depth: u32, contribution: f64) -> Option<Vector3<f64>> {
    // Early exit when max depth is reach or the contribution factor is too low.
    //
    // The contribution factor is checked here to force the user to provide one.
    if contribution < 0.01 {
        return None;
    }

    if depth > MAX_DEPTH {
        return None;
    }

    let intersect = check_intersect_scene(ray, scene);

    match intersect {
        None => {
            return Some(scene.bg_color);
        }
        Some((dist, object)) => {
            let point_of_intersection = ray.point + (ray.direction * dist);

            let mut color = Vector3::new(0.0, 0.0, 0.0);

            for material in object.get_materials() {
                if let Some(calculated_color) = material.get_surface_color(
                    ray,
                    &scene,
                    point_of_intersection,
                    object.get_normal(point_of_intersection),
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

fn check_intersect_scene(ray: Ray, scene: &Scene) -> Option<(f64, &Box<dyn Object>)> {
    let mut closest: Option<(f64, &Box<dyn Object>)> = None;

    let bvh_ray = bvh::ray::Ray::new(
        bvh::nalgebra::convert(ray.point),
        bvh::nalgebra::convert(ray.direction),
    );

    let hit_sphere_aabbs = scene.bvh.traverse(&bvh_ray, &scene.objects);
    for object in hit_sphere_aabbs {
        if let Some(dist) = object.test_intersect(ray) {
            // If we found an intersection we check if the current
            // closest intersection is farther than the intersection
            // we found.
            match closest {
                None => closest = Some((dist, &object)),
                Some((closest_dist, _)) => {
                    if dist < closest_dist {
                        closest = Some((dist, &object));
                    }
                }
            }
        }
    }

    closest
}

fn check_intersect_scene_simple(ray: Ray, scene: &Scene, max_dist: f64) -> bool {
    let bvh_ray = bvh::ray::Ray::new(
        bvh::nalgebra::convert(ray.point),
        bvh::nalgebra::convert(ray.direction),
    );

    let hit_sphere_aabbs = scene.bvh.traverse(&bvh_ray, &scene.objects);
    for object in hit_sphere_aabbs {
        if let Some(dist) = object.test_intersect(ray) {
            // If we found an intersection we check if distance is less
            // than the max distance we want to check. If so -> exit with true
            if dist < max_dist {
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
        direction: bvh::nalgebra::normalize(&(light.position - position)),
    };

    let distance = bvh::nalgebra::distance(&position, &light.position);

    if check_intersect_scene_simple(ray, scene, distance) {
        return false;
    }

    true
}
