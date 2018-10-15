extern crate glutin_window;
extern crate graphics;
extern crate image;
extern crate opengl_graphics;
extern crate piston;

#[macro_use]
extern crate lazy_static;

use glutin_window::GlutinWindow as Window;
use graphics::*;
use opengl_graphics::{GlGraphics, OpenGL, Texture, TextureSettings};
use piston::event_loop::*;
use piston::input::*;
use piston::window::WindowSettings;
use std::sync::{Arc, RwLock};

mod renderer;

const IMAGE_WIDTH: u32 = 200;
const IMAGE_HEIGHT: u32 = 200;

lazy_static! {
    static ref IMAGE_BUFFER: Arc<RwLock<image::RgbaImage>> = Arc::new(RwLock::new(
        image::ImageBuffer::new(IMAGE_WIDTH, IMAGE_HEIGHT)
    ));
}

fn main() {
    println!("Hello, world!");

    let opengl = OpenGL::V3_2;
    let mut window: Window = WindowSettings::new("Rust Raytracer", [IMAGE_WIDTH, IMAGE_HEIGHT])
        .opengl(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    let mut gl = GlGraphics::new(opengl);

    // setup separate thread for rendering

    let mut image_buffer: image::RgbaImage = image::ImageBuffer::new(IMAGE_WIDTH, IMAGE_HEIGHT);
    let mut texture: Texture = Texture::from_image(&image_buffer, &TextureSettings::new());

    image_buffer.put_pixel(100, 100, image::Rgba([255, 0, 0, 255]));
    texture.update(&image_buffer);
    let mut events = Events::new(EventSettings::new());

    //renderer::render();

    let mut loop_count = 0;

    while let Some(e) = events.next(&mut window) {
        // draw current render state
        if let Some(args) = e.render_args() {
            println!("Write pixel, at x{}, y{}",  loop_count % IMAGE_WIDTH,
                (loop_count / IMAGE_WIDTH) % IMAGE_HEIGHT);

            image_buffer.put_pixel(
                loop_count % IMAGE_WIDTH,
                (loop_count / IMAGE_WIDTH) % IMAGE_HEIGHT,
                image::Rgba([loop_count as u8 % 255, (loop_count / IMAGE_WIDTH) as u8 % 255, 0, 255]),
            );
            //image_buffer.put_pixel(200, 200, image::Rgba([255, 0, 0, 255]));

            texture.update(&image_buffer);

            //IMAGE_BUFFER.write().unwrap().put_pixel(x, y, image::Rgba([ 255, 255, 255, 255]);

            println!("Write pixel, loop {}", loop_count);
            //println!("Write pixel, buffer {:?}", IMAGE_BUFFER.read().unwrap());

            gl.draw(args.viewport(), |c, gl| {
                clear([0.0, 0.0, 0.0, 1.0], gl);
                image(&texture, c.transform, gl);
            });

            loop_count += 1;
        }
    }
}
