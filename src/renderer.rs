use camera::Camera;
use cgmath::*;
use image;
use rand::*;
use scene::{Light, Scene, Sphere};
use std::cmp;
use std::sync::Mutex;
use std::thread;
use IMAGE_BUFFER;

const THREAD_COUNT: u32 = 1;
const BUCKETS: u32 = 1;
const MAX_DEPTH: u32 = 5;
const SAMPLES: u32 = 1; // total sampels squared
const WORK: u32 = ::IMAGE_WIDTH * ::IMAGE_HEIGHT;

#[derive(Copy, Clone, Debug)]
struct Work {
    start: u32,
    end: u32,
}

lazy_static! {
    static ref WORK_QUEUE: Mutex<Vec<Work>> = {
        let mut vec: Vec<Work> = Vec::new();
        let per_bucket = WORK / BUCKETS;

        for x in 0..BUCKETS {
            vec.push(Work {
                start: x * per_bucket,
                end: x * per_bucket + per_bucket,
            });
        }

        Mutex::new(vec)
    };
}

fn get_work() -> Option<Work> {
    let mut rng = thread_rng();
    let mut queue = WORK_QUEUE.lock().unwrap();
    let len = queue.len();

    if len == 0 {
        return None;
    }

    Some(queue.remove(rng.gen_range(0, len)))
}

pub fn render(camera: Camera, scene: &Scene) {
    let image_width = IMAGE_BUFFER.read().unwrap().width();
    let image_height = IMAGE_BUFFER.read().unwrap().height();

    println!("Start render, w{}, h{}", image_width, image_height);
    println!("Camera {:?}", camera);

    // thread id is used to divide the work
    for id in 0..THREAD_COUNT {
        let thread_scene = scene.clone();
        let _thread = thread::spawn(move || {
            let thread_id = id.clone();

            loop {
                let work_wrapped = get_work();

                if let None = work_wrapped {
                    break;
                }

                let work = work_wrapped.unwrap();

                // prevent rounding error, cap at max work
                let work_end = cmp::min(WORK, work.end);

                for pos in work.start..work_end {
                    let rays =
                        get_rays_at(camera, image_width, image_height, pos, SAMPLES)
                            .unwrap();
                    let rays_len = rays.len();

                    let mut pixel_color = Vector3::new(0.0, 0.0, 0.0);

                    for ray in rays {
                        //println!("pos: {}, ray: {:?}", pos, ray);
                        pixel_color += trace(ray, &thread_scene.clone(), 1).unwrap();
                    }

                    pixel_color /= rays_len as f64;

                    //println!("Thread id {}, pos {}, Ray {:?}", thread_id, pos, ray);

                    let pixel_color_rgba = image::Rgba([
                        (pixel_color.x * 255.0) as u8,
                        (pixel_color.y * 255.0) as u8,
                        (pixel_color.z * 255.0) as u8,
                        255,
                    ]);

                    IMAGE_BUFFER.write().unwrap().put_pixel(
                        pos % image_width,
                        (pos / image_width) % image_height,
                        pixel_color_rgba,
                    );
                }
            }
        });
    }
}

#[derive(Debug, Copy, Clone)]
struct Ray {
    point: Vector3<f64>,
    direction: Vector3<f64>,
}

fn get_rays_at(
    camera: Camera,
    width: u32,
    height: u32,
    pos: u32,
    samples: u32,
) -> Result<Vec<Ray>, &'static str> {
    use std::f64::consts::PI;
    let mut rng = thread_rng();

    let pos_x = pos % width;
    let pos_y = pos / width;

    //println!("x {}, y {}", pos_x, pos_y);

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
        let vertical_offset =
            camera.up * ((-(pos_y as f64) * pixel_height) + sub_pixel_vertical_offset + half_height);

        let ray = Ray {
            point: camera.position,
            direction: (camera.forward + horizontal_offset + vertical_offset).normalize(),
        };

        rays.push(ray);
    }

    // let ray_vector = (camera.forward + horizotal_offset + vertical_offset).normalize();

    // let ray = Ray {
    //     point: camera.position,
    //     direction: ray_vector,
    // };

    Ok(rays)
}

fn trace(ray: Ray, scene: &Scene, depth: u32) -> Option<Vector3<f64>> {
    if depth > MAX_DEPTH {
        return None;
    }

    // for now we just check for intersection with the spheres
    let intersect = check_intersect_scene(ray, scene);

    match intersect {
        None => {
            return Some(scene.bg_color);
        }
        Some((dist, sphere)) => {
            let point_of_intersection = ray.point + (ray.direction * dist);

            return Some(surface_calculate_color(
                ray,
                scene,
                sphere,
                point_of_intersection,
                sphere_normal(sphere, point_of_intersection),
                depth,
            ));
        }
    }
}

fn check_intersect_scene(ray: Ray, scene: &Scene) -> Option<(f64, Sphere)> {
    let mut closest: Option<(f64, Sphere)> = None;

    for sphere in &scene.spheres {
        if let Some(dist) = check_intersect_sphere(ray, *sphere) {
            // If we found an intersection we check if the current
            // closest intersection is farther than the intersection
            // we found.
            match closest {
                None => closest = Some((dist, *sphere)),
                Some((closest_dist, _)) => {
                    if dist < closest_dist {
                        closest = Some((dist, *sphere));
                    }
                }
            }
        }
    }

    closest
}

/// Checks if the ray intersects with the sphere and if so
/// returns the distance at which this intersection occurs.
fn check_intersect_sphere(ray: Ray, sphere: Sphere) -> Option<f64> {
    let camera_to_sphere_center = sphere.position - ray.point;
    let v = camera_to_sphere_center.dot(ray.direction);
    let eo_dot = camera_to_sphere_center.dot(camera_to_sphere_center); // camera_to_sphere length squared
    let discriminant = sphere.radius.powf(2.0) - eo_dot + v.powf(2.0); // radius - camera_to_sphere_center length + v all squared

    // Example:
    //
    // ray starts from 0,0,0 with direction 0,0,1 (forward and a bit up)
    // sphere is as 0,4,0 with radius 1
    //
    // camera_to_sphere_center = 0,4,0 - 0,0,0 = 0,4,0
    // v = camera_to_sphere_center . ray.direction = 0*0 + 4*0 + 0*1 = 0
    // eoDot = camera_to_sphere_center . itself = 0*0 + 4*4 + 0*0 = 16
    // discriminant = sphere radius ^ 2 - eoDot + v ^ 2 = (1*1) - 16 + (0*0) = -15
    // no hit

    //discriminant = (sphere.radius * sphere.radius) - eoDot + (v * v);

    // Some(1.0)
    if discriminant < 0.0 {
        return None;
    }

    Some(v - discriminant.sqrt())
}

fn sphere_normal(sphere: Sphere, position: Vector3<f64>) -> Vector3<f64> {
    (position - sphere.position).normalize()
}

fn vector_reflect(vec: Vector3<f64>, normal: Vector3<f64>) -> Vector3<f64> {
    2.0 * vec.dot(normal) * normal - vec
}

fn check_light_visible(position: Vector3<f64>, scene: &Scene, light: Light) -> bool {
    let ray = Ray {
        point: position,
        direction: light.position - position,
    };

    if let Some((dist, sphere)) = check_intersect_scene(ray, scene) {
        if dist < -0.005 {
            return true;
        } else {
            return false;
        }
    }

    true
}

/// Calculate the lambert, specular and ambient color.
fn surface_calculate_color(
    ray: Ray,
    scene: &Scene,
    sphere: Sphere,
    point_of_intersection: Vector3<f64>,
    normal: Vector3<f64>,
    depth_immutable: u32,
) -> Vector3<f64> {
    let mut depth = depth_immutable;
    let sphere_color = sphere.color;
    let mut c = Vector3::new(0.0, 0.0, 0.0);
    let mut lambert_amount = 0.0;

    if sphere.lambert > 0.0 {
        // Check all the lights for visibility if visible use
        // the cross product of the normal and the vector to the light
        // to calculate the lambert contribution.
        for light in scene.lights.clone() {
            if !check_light_visible(point_of_intersection, &scene, light) {
                continue;
            }

            let contribution = (light.position - point_of_intersection)
                .normalize()
                .dot(normal);
            if contribution > 0.0 {
                lambert_amount += contribution;
            }
        }

        // cap the lambert amount to 1
        if lambert_amount > 1.0 {
            lambert_amount = 1.0;
        }
    }

    if sphere.specular > 0.0 {
        let reflect_ray = Ray {
            point: point_of_intersection,
            direction: vector_reflect(ray.direction, normal),
        };

        depth += 1;

        if let Some(reflect_surface_color) = trace(reflect_ray, scene, depth) {
            c += reflect_surface_color * sphere.specular;
        }
    }

    return c + (sphere_color * (lambert_amount * sphere.lambert)) + (sphere_color * sphere.ambient);
}
