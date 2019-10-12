extern crate bvh;
extern crate glutin_window;
extern crate graphics;
extern crate image;
extern crate opengl_graphics;
extern crate piston;
extern crate rand;

#[macro_use]
extern crate lazy_static;

use bvh::bvh::BVH;
use bvh::nalgebra::{Point3, Vector3};
use glutin_window::GlutinWindow as Window;
use graphics::*;
use opengl_graphics::{GlGraphics, GlyphCache, OpenGL, Texture, TextureSettings};
use piston::event_loop::*;
use piston::input::*;
use piston::window::WindowSettings;
use std::sync::{Arc, RwLock};

mod camera;
mod helpers;
mod materials;
mod renderer;
mod scene;

const IMAGE_WIDTH: u32 = 800;
const IMAGE_HEIGHT: u32 = 600;
const OUTPUT: &str = "window";

lazy_static! {
    static ref IMAGE_BUFFER: Arc<RwLock<image::RgbaImage>> = Arc::new(RwLock::new(
        image::ImageBuffer::new(IMAGE_WIDTH, IMAGE_HEIGHT)
    ));
}

fn main() {
    //let args: Vec<String> = env::args().collect();

    let camera = camera::Camera::new(Point3::new(3.0, 0.0, 2.2), Point3::new(0.0, 0.0, 1.0), 50.0);

    let _sphere = scene::Sphere {
        position: Point3::new(0.0, 0.0, 2.7),
        radius: 1.0,
        materials: vec![
        //     Box::new(materials::FresnelReflection {
        //     weight: 1.0,
        //     glossiness: 1.0,
        //     ior: 1.2,
        //     reflection: 1.0,
        //     refraction: 0.0,
        //     color: Vector3::new(1.0,1.0,1.0),
        // }),
           Box::new(materials::Reflection {
            weight: 1.0,
            glossiness: 1.0,
        }),
        ],
        node_index: 0,
    };

    let light = scene::Light {
        position: Point3::new(7.0, 2.0, 0.0),
        intensity: 1.0,
        color: Vector3::new(1.0, 1.0, 1.0), // white
    };

    let light_1 = scene::Light {
        position: Point3::new(-1.0, 0.0, 8.0),
        intensity: 1.0,
        color: Vector3::new(1.0, 1.0, 1.0), // white
    };

    // // Setup a basic test scene
    let mut spheres_boxed: Vec<Box<dyn scene::Object>> = vec![];

    spheres_boxed.push(Box::new(_sphere));

    let spacing = 0.3;
    let radius = 0.1;
    let xcount = 10;
    let ycount = 10;
    let zcount = 10;

    for x in -xcount / 2..xcount / 2 + 1 {
        for y in -ycount / 2..ycount / 2 + 1 {
            for z in -zcount / 2..zcount / 2 + 1 {
                // skip own position
                // if x == 0 && y == 0 && z == 0 {
                //     continue;
                // }

                let c = Vector3::new(
                    (x + xcount / 2) as f64 / xcount as f64,
                    (y + ycount / 2) as f64 / ycount as f64,
                    (z + zcount / 2) as f64 / zcount as f64,
                );
                let sphere = scene::Sphere {
                    position: Point3::new(
                        x as f64 * spacing,
                        y as f64 * spacing,
                        -z as f64 * spacing,
                    ),
                    radius: radius,
                    materials: vec![
                        Box::new(materials::FresnelReflection {
                            weight: 1.0,
                            glossiness: 0.9,
                            ior: 1.5,
                            reflection: 1.0,
                            refraction: 0.0,
                            color: c,
                        }),
                        // Box::new(materials::Lambert {
                        //     color: Vector3::new(0.5, 0.5, 0.5),
                        //     weight: 1.0,
                        // }),
                    ],
                    node_index: 0,
                };

                spheres_boxed.push(Box::new(sphere));
            }
        }
    }

    let plane = scene::Plane {
        position: Point3::new(0.0, -2.0, 0.0),
        normal: Vector3::new(0.0, 1.0, 0.0), // up
        materials: vec![
            Box::new(materials::Lambert {
                color: Vector3::new(0.5, 0.5, 0.5),
                weight: 1.0,
            }),
            // Box::new(materials::FresnelReflection {
            //     color: Vector3::new(0.9, 0.9, 0.9),
            //     glossiness: 1.0,
            //     ior: 1.5,
            //     reflection: 0.8,
            //     refraction: 0.0,
            //     weight: 1.0,
            // }),
        ],
        node_index: 0,
    };

    let plane2 = scene::Plane {
        position: Point3::new(-2.0, 0.0, 0.0),
        normal: Vector3::new(1.0, 0.0, 0.0), // right
        materials: vec![
            Box::new(materials::Lambert {
                color: Vector3::new(0.5, 0.5, 0.5),
                weight: 1.0,
            }),
            // Box::new(materials::FresnelReflection {
            //     color: Vector3::new(0.9, 0.9, 0.9),
            //     glossiness: 1.0,
            //     ior: 1.5,
            //     reflection: 0.8,
            //     refraction: 0.0,
            //     weight: 1.0,
            // }),
        ],
        node_index: 0,
    };

    spheres_boxed.push(Box::new(plane));
    spheres_boxed.push(Box::new(plane2));

    let bvh = BVH::build(&mut spheres_boxed);

    let scene = scene::Scene {
        bg_color: Vector3::new(0.7, 0.7, 0.9), //Vector3::new(0.62, 0.675, 0.855), // red
        objects: spheres_boxed,                //spheres,
        bvh: bvh,
        lights: vec![light, light_1],
    };

    // scene.push_object(Box::new(plane));

    // Start the render threads
    let threads = renderer::render(camera, Arc::new(scene));

    if OUTPUT == "window" {
        let opengl = OpenGL::V3_2;
        let mut window: Window = WindowSettings::new("Rust Raytracer", [IMAGE_WIDTH, IMAGE_HEIGHT])
            .graphics_api(opengl)
            .exit_on_esc(true)
            .build()
            .unwrap();

        let mut gl = GlGraphics::new(opengl);
        let mut texture: Texture =
            Texture::from_image(&IMAGE_BUFFER.read().unwrap(), &TextureSettings::new());
        let mut glyph_cache =
            GlyphCache::new("assets/FiraSans-Regular.ttf", (), TextureSettings::new()).unwrap();

        let event_settings = EventSettings::new();
        event_settings.max_fps(3);

        let mut events = Events::new(event_settings);
        while let Some(e) = events.next(&mut window) {
            // draw current render state
            if let Some(args) = e.render_args() {
                // The RwLock doesn't require locking on read, so this doesn't
                // block the rendering.
                texture.update(&IMAGE_BUFFER.read().unwrap());

                let mut stats_text: String = "".to_owned();
                for (_, thread) in &renderer::STATS.read().unwrap().threads {
                    stats_text.push_str(" ");
                    stats_text.push_str(&format!("| {:.0} ns, ", thread.ns_per_ray));
                    stats_text.push_str(&format!("{} rays done ", thread.rays_done));
                }

                gl.draw(args.viewport(), |c, gl| {
                    clear([0.0, 0.0, 0.0, 1.0], gl);
                    image(&texture, c.transform, gl);

                    rectangle(
                        [0.0, 0.0, 0.0, 0.9],
                        rectangle::rectangle_by_corners(0.0, 0.0, IMAGE_WIDTH as f64, 25.0),
                        c.transform.trans(0.0, 0.0),
                        gl,
                    );

                    let transform = c.transform.trans(10.0, 15.0);
                    text(
                        [1.0, 0.0, 0.0, 1.0],
                        10,
                        &stats_text,
                        &mut glyph_cache,
                        transform,
                        gl,
                    )
                    .expect("Error drawing text.");
                });
            }
        }
    } else {
        println!("Waiting...");

        let mut done = false;

        let total_rays = IMAGE_WIDTH * IMAGE_HEIGHT * renderer::SAMPLES;

        // while !done {
        //     let stats = renderer::STATS.read().unwrap();

        //     println!("{:?}", stats);
        //     println!(
        //         "Progress {}/{} ({:.3}%)",
        //         stats.rays_done,
        //         total_rays,
        //         (stats.rays_done as f64 / total_rays as f64) * 100.0
        //     );

        //     thread::sleep(time::Duration::from_millis(1000));
        // }
        // wait for all threads to finish
        for handle in threads {
            let _ = handle.join();
        }

        println!("Done!");
    }
}
