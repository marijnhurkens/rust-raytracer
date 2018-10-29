extern crate cgmath;
extern crate glutin_window;

extern crate graphics;
extern crate image;
extern crate opengl_graphics;
extern crate piston;
extern crate rand;

#[macro_use]
extern crate lazy_static;

use cgmath::Vector3;
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
const IMAGE_HEIGHT: u32 = 500;

lazy_static! {
    static ref IMAGE_BUFFER: Arc<RwLock<image::RgbaImage>> = Arc::new(RwLock::new(
        image::ImageBuffer::new(IMAGE_WIDTH, IMAGE_HEIGHT)
    ));
}

fn main() {
    let opengl = OpenGL::V3_2;
    let mut window: Window = WindowSettings::new("Rust Raytracer", [IMAGE_WIDTH, IMAGE_HEIGHT])
        .opengl(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    let mut gl = GlGraphics::new(opengl);

    let mut texture: Texture =
        Texture::from_image(&IMAGE_BUFFER.read().unwrap(), &TextureSettings::new());

    let camera = camera::Camera::new(
        Vector3::new(2.0, 1.4, 4.0),
        Vector3::new(0.0, -0.5, 0.0),
        80.0,
    );

    let _sphere = scene::Sphere {
        position: Vector3::new(0.0, 0.0, 0.0),
        radius: 0.5,
        materials: vec![Box::new(materials::FresnelReflection {
            weight: 1.0,
            glossiness: 1.0,
            ior: 1.3,
            reflection: 1.0,
            refraction: 0.4,
            color: Vector3::new(0.4, 0.4, 0.4),
        })],
    };

    let light = scene::Light {
        position: Vector3::new(-10.0, 2.0, 10.0),
        intensity: 1.0,
        color: Vector3::new(1.0, 1.0, 1.0), // white
    };

       let light_1 = scene::Light {
        position: Vector3::new(-2.0, 1.5, -5.0),
        intensity: 1.0,
        color: Vector3::new(1.0, 1.0, 1.0), // white
    };

    // Setup a basic test scene
    let mut spheres: Vec<Box<scene::Object>> = vec![];

    let spacing = 1.0;
    let radius = 0.4;
    let xcount = 4;
    let ycount = 4;
    let zcount = 4;

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
                    position: Vector3::new(
                        x as f64 * spacing,
                        y as f64 * spacing,
                        -z as f64 * spacing,
                    ),
                    radius: radius,
                    materials: vec![
                            Box::new(materials::FresnelReflection {
                            weight: 1.0,
                            glossiness: 1.0,
                            ior: 1.5,
                            reflection: 0.8,
                            refraction: 0.0,
                            color: c,
                        }),
                        // Box::new(materials::Lambert {
                        //     color: Vector3::new(0.5, 0.5, 0.5),
                        //     weight: 1.0,
                        // }),
                    ],
                };

                spheres.push(Box::new(sphere));
            }
        }
    }

    let plane = scene::Plane {
        position: Vector3::new(0.0, -1.3, 0.0),
        normal: Vector3::new(0.0, 1.0, 0.0), // up
        materials: vec![
            // Box::new(materials::Lambert {
            //     color: Vector3::new(0.5, 0.5, 0.5),
            //     weight: 1.0,
            // }),
            Box::new(materials::FresnelReflection {
                color: Vector3::new(0.5, 0.5, 0.5),
                glossiness: 0.9,
                ior: 1.5,
                reflection: 0.0,
                refraction: 0.0,
                weight: 1.0,
            }),
        ],
    };

    let mut scene = scene::Scene {
        bg_color: Vector3::new(0.7, 0.7, 0.9), //Vector3::new(0.62, 0.675, 0.855), // red
        spheres: vec![],                       //spheres,
        objects: spheres,
        lights: vec![light, light_1],
    };

    scene.push_object(Box::new(plane));

    // println!("{:#?}", scene);

    // Start the render threads
    renderer::render(camera, Arc::new(scene));

    let mut glyph_cache =
        GlyphCache::new("assets/FiraSans-Regular.ttf", (), TextureSettings::new()).unwrap();

    let mut events = Events::new(EventSettings::new());
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

                let transform = c.transform.trans(10.0, 15.0);
                text(
                    [1.0, 0.0, 0.0, 1.0],
                    10,
                    &stats_text,
                    &mut glyph_cache,
                    transform,
                    gl,
                ).expect("Error drawing text.");
            });
        }
    }
}
