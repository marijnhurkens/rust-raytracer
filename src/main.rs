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
use opengl_graphics::{GlGraphics, OpenGL, Texture, TextureSettings};
use piston::event_loop::*;
use piston::input::*;
use piston::window::WindowSettings;
use std::sync::{Arc, RwLock};

mod camera;
mod helpers;
mod materials;
mod renderer;
mod scene;

const IMAGE_WIDTH: u32 = 700;
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

    // setup separate thread for rendering

    //let mut image_buffer: image::RgbaImage = image::ImageBuffer::new(IMAGE_WIDTH, IMAGE_HEIGHT);
    let mut texture: Texture =
        Texture::from_image(&IMAGE_BUFFER.read().unwrap(), &TextureSettings::new());

    let camera = camera::Camera::new(
        Vector3::new(0.0, 0.3, 0.0),
        Vector3::new(0.0, -0.1, -3.0),
        40.0,
    );

    let sphere = scene::Sphere {
        position: Vector3::new(0.0, 0.0, -3.0),
        radius: 0.3,
        materials: vec![
            Box::new(materials::Lambert {
                weight: 0.0,
                color: Vector3::new(0.7, 0.0, 0.0),
            }),
            Box::new(materials::FresnelReflection {
                weight: 0.9,

                glossiness: 1.0,
                ior: 0.8,
            }),
        ],
    };

    let sphere_1 = scene::Sphere {
        position: Vector3::new(0.0, -1.8, -3.0),
        radius: 1.5,
        materials: vec![Box::new(materials::Lambert {
            weight: 1.0,
            color: Vector3::new(0.7, 0.7, 0.0),
        })],
    };

    let sphere_2 = scene::Sphere {
        position: Vector3::new(1.0, 1.0, -4.0),
        radius: 1.0,
        materials: vec![Box::new(materials::Lambert {
            weight: 1.0,
            color: Vector3::new(0.7, 0.7, 0.0),
        })],
    };

    let sphere_3 = scene::Sphere {
        position: Vector3::new(2.0, -1.0, -2.0),
        radius: 1.0,
        materials: vec![Box::new(materials::Lambert {
            weight: 1.0,
            color: Vector3::new(0.7, 0.7, 0.0),
        })],
    };

    let light = scene::Light {
        position: Vector3::new(-0.2, 5.0, -2.2),
        intensity: 0.3,
        color: Vector3::new(1.0, 1.0, 1.0), // white
    };

    let light_1 = scene::Light {
        position: Vector3::new(1.0, 0.0, 0.0),
        intensity: 0.4,
        color: Vector3::new(1.0, 1.0, 1.0), // white
    };

    let scene: Arc<scene::Scene> = Arc::new(scene::Scene {
        bg_color: Vector3::new(0.9, 0.9, 0.9), //Vector3::new(0.62, 0.675, 0.855), // red
        spheres: vec![sphere, sphere_1, sphere_2], //, sphere_3],
        lights: vec![light, light_1],
    });

    renderer::render(camera, scene);

    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        // draw current render state
        if let Some(args) = e.render_args() {
            texture.update(&IMAGE_BUFFER.read().unwrap());

            gl.draw(args.viewport(), |c, gl| {
                clear([0.0, 1.0, 1.0, 1.0], gl);
                image(&texture, c.transform, gl);
            });
        }
    }
}
