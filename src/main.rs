extern crate cgmath;
extern crate glutin_window;
extern crate graphics;
extern crate image;
extern crate opengl_graphics;
extern crate piston;

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
mod renderer;
mod scene;

const IMAGE_WIDTH: u32 = 600;
const IMAGE_HEIGHT: u32 = 400;

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
        Vector3::new(0.0, -2.0, 0.0),
        Vector3::new(0.0, 4.0, 0.0),
        50.0,
    );

    let sphere = scene::Sphere {
        position: Vector3::new(-1.5, 2.0, 0.0),
        radius: 1.0,
        color: Vector3::new(0.0, 0.4, 0.1), // green
        lambert: 0.2,
        specular: 0.7,
        ambient: 0.1,
    };

    let sphere_1 = scene::Sphere {
        position: Vector3::new(1.5, 2.0, 0.0),
        radius: 1.0,
        color: Vector3::new(0.9, 0.1, 0.3), // pink
        lambert: 0.2,
        specular: 0.7,
        ambient: 0.1,
    };

    let sphere_2 = scene::Sphere {
        position: Vector3::new(0.0, 7.0, 0.0),
        radius: 1.0,
        color: Vector3::new(0.1, 0.1, 0.8), // blue
        lambert: 0.2,
        specular: 0.7,
        ambient: 0.1,
    };

    let light = scene::Light {
        position: Vector3::new(0.0, 0.0, -2000.0),
        intensity: 1.0,
        color: Vector3::new(1.0, 1.0, 1.0), // white
    };

    let light_1 = scene::Light {
        position: Vector3::new(-3.0, 4.0, 2.0),
        intensity: 1.0,
        color: Vector3::new(1.0, 1.0, 1.0), // white
    };

    let scene = scene::Scene {
        bg_color: Vector3::new(1.0, 0.0, 0.0), // red
        spheres: vec![sphere, sphere_1, sphere_2],
        lights: vec![light, light_1],
    };

    renderer::render(camera, &scene);

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
