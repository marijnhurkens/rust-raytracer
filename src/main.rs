#![allow(unused)]
#![warn(clippy::all, clippy::cargo)]

extern crate core;

use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, RwLock};
use std::thread::JoinHandle;
use std::time::Duration;

use clap::Parser;
use ggez::conf::{FullscreenType, NumSamples, WindowMode, WindowSetup};
use ggez::event::run;
use ggez::graphics::{self, Color, DrawParam, ImageEncodingFormat, ImageFormat};
use ggez::input::keyboard;
use ggez::input::keyboard::KeyCode;
use ggez::winit::dpi::LogicalSize;
use ggez::{event, GameError};
use ggez::{Context, GameResult};
use nalgebra::{Point2, Vector2};
use yaml_rust::YamlLoader;

use denoise::denoise;
use film::{Film, FilterMethod};
use helpers::{yaml_array_into_point2, yaml_array_into_point3, yaml_into_u32};
use objects::Object;
use renderer::{DebugBuffer, ThreadMessage, DEBUG_BUFFER};

use crate::camera::Camera;
use crate::helpers::Bounds;
use crate::renderer::{debug_write_pixel_f64, Settings};
use crate::sampler::SobolSampler;

mod bsdf;
mod camera;
mod denoise;
mod film;
mod helpers;
mod lights;
mod materials;
mod normal;
mod objects;
mod renderer;
mod sampler;
mod scene;
mod surface_interaction;
mod textures;
mod tracer;

#[derive(Parser, Debug)]
struct Args {
    scene_folder: Option<String>,
    #[arg(short, long, default_value_t = false)]
    skip_obj: bool,
}

struct MainState {
    redraw: bool,
    film: Arc<RwLock<Film>>,
    threads: Vec<JoinHandle<()>>,
    receiver: Receiver<ThreadMessage>,
    running_threads: usize,
    finished: bool,
    denoised: bool,
    should_denoise: bool,
    debug_normals: bool,
    debug_albedo: bool,
    debug_buffer: bool,
}

impl MainState {
    fn new(
        film: Arc<RwLock<Film>>,
        threads: Vec<JoinHandle<()>>,
        receiver: Receiver<ThreadMessage>,
        running_threads: usize,
        should_denoise: bool,
    ) -> GameResult<MainState> {
        Ok(MainState {
            redraw: true,
            film,
            threads,
            receiver,
            running_threads,
            finished: false,
            should_denoise,
            denoised: false,
            debug_normals: false,
            debug_buffer: false,
            debug_albedo: false,
        })
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        if !ctx.time.check_update_time(30) {
            ggez::timer::sleep(Duration::from_secs_f64(1.0 / 10.0));
            return Ok(());
        }

        if ctx.time.ticks() % 3 == 0 {
            self.redraw = true;
        }

        self.debug_normals = ctx.keyboard.is_key_pressed(KeyCode::N);
        self.debug_albedo = ctx.keyboard.is_key_pressed(KeyCode::A);
        self.debug_buffer = ctx.keyboard.is_key_pressed(KeyCode::D);

        let message = self.receiver.try_recv();
        if let Ok(message) = message {
            if message.finished {
                self.running_threads -= 1;
            }
        }

        if self.running_threads == 0 && !self.finished {
            println!("All work is done.");
            self.finished = true;

            if !self.denoised && self.should_denoise {
                print!("Denoising...");
                let mut film = self.film.write().unwrap();
                denoise(&mut film);
                self.denoised = true;
                println!(" done!");
            }
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        if !self.redraw {
            return Ok(());
        }
        self.redraw = false;

        let film = self.film.read().unwrap();
        let image_width = film.image_size.x;
        let image_height = film.image_size.y;
        let mut output = vec![0u8; image_width as usize * image_height as usize * 4];

        if self.debug_normals {
            let mut i = 0;
            film.pixels.clone().iter().for_each(|pixel| {
                let scaled_normal =
                    (pixel.normal * 0.5 + nalgebra::Vector3::new(0.5, 0.5, 0.5)) * 255.0;
                output[i] = scaled_normal.x as u8;
                output[i + 1] = scaled_normal.y as u8;
                output[i + 2] = scaled_normal.z as u8;
                output[i + 3] = 255;
                i += 4;
            });
        } else if self.debug_albedo {
            let mut i = 0;
            film.pixels.clone().iter().for_each(|pixel| {
                output[i] = (pixel.albedo.x * 255.0) as u8;
                output[i + 1] = (pixel.albedo.y * 255.0) as u8;
                output[i + 2] = (pixel.albedo.z * 255.0) as u8;
                output[i + 3] = 255;
                i += 4;
            });
        } else if self.debug_buffer {
            let mut i = 0;
            DEBUG_BUFFER
                .read()
                .unwrap()
                .buffer
                .chunks(3)
                .for_each(|chunk| {
                    output[i] = (chunk[0] * 255.0) as u8;
                    output[i + 1] = (chunk[1] * 255.0) as u8;
                    output[i + 2] = (chunk[2] * 255.0) as u8;
                    output[i + 3] = 255;
                    i += 4;
                });
        } else {
            let mut i = 0;
            for chunk in film.image_buffer.clone().into_raw().chunks(3) {
                output[i] = chunk[0];
                output[i + 1] = chunk[1];
                output[i + 2] = chunk[2];
                output[i + 3] = 255;
                i += 4;
            }
        }

        let image = graphics::Image::from_pixels(
            ctx,
            &output,
            ImageFormat::Rgba8UnormSrgb,
            image_width,
            image_height,
        );

        // now lets render our scene once in the top left and in the bottom right
        let window_size = ctx.gfx.window().inner_size();
        let image_ratio = image_width as f32 / image_height as f32;
        let window_ratio = window_size.width as f32 / window_size.height as f32;

        let scale = if window_ratio > image_ratio {
            // window is wider, use max height
            let scale = window_size.height as f32 / image.height() as f32;
            ggez::glam::vec2(scale, scale)
        } else {
            // window is narrower, use max width
            let scale = window_size.width as f32 / image.width() as f32;
            ggez::glam::vec2(scale, scale)
        };

        let mut canvas = graphics::Canvas::from_frame(ctx, Color::new(0.0, 0.0, 0.0, 1.0));
        canvas.draw(&image, graphics::DrawParam::new().scale(scale));
        canvas.finish(ctx);

        Ok(())
    }

    fn resize_event(
        &mut self,
        ctx: &mut Context,
        width: f32,
        height: f32,
    ) -> Result<(), GameError> {
        let new_rect = graphics::Rect::new(0.0, 0.0, width, height);
        // ctx.gfx.window().set_inner_size()
        // graphics::set_screen_coordinates(ctx, new_rect).unwrap();

        Ok(())
    }
}

fn main() -> GameResult {
    let args = Args::parse();

    // Load scene from yaml file
    let scene_folder_param = args.scene_folder.unwrap();
    let scene_folder = Path::new(&scene_folder_param);
    let scene = scene::Scene::load_from_folder(scene_folder);

    // Get settings from yaml file
    let mut file = File::open(scene_folder.join("render_settings.yaml"))
        .expect("Unable to open render_settings.yaml file");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Unable to read file");
    let settings_yaml = &YamlLoader::load_from_str(&contents).unwrap()[0];

    let settings = Settings {
        thread_count: yaml_into_u32(&settings_yaml["renderer"]["threads"]),
        depth_limit: yaml_into_u32(&settings_yaml["renderer"]["depth_limit"]),
        max_samples: yaml_into_u32(&settings_yaml["sampler"]["max_samples"]),
    };

    let image_width = settings_yaml["film"]["image_width"].as_i64().unwrap() as u32;
    let image_height = settings_yaml["film"]["image_height"].as_i64().unwrap() as u32;
    let aspect_ratio = image_width as f64 / image_height as f64;
    let window_scale = settings_yaml["window"]["scale"].as_f64().unwrap_or(1.5) as f32;
    let crop_start = if !settings_yaml["film"]["crop"]["start"].is_badvalue() {
        yaml_array_into_point2(&settings_yaml["film"]["crop"]["start"])
    } else {
        Point2::origin()
    };
    let crop_end = if !settings_yaml["film"]["crop"]["end"].is_badvalue() {
        yaml_array_into_point2(&settings_yaml["film"]["crop"]["end"])
    } else {
        Point2::new(
            settings_yaml["film"]["image_width"].as_i64().unwrap() as u32,
            settings_yaml["film"]["image_height"].as_i64().unwrap() as u32,
        )
    };
    let should_denoise = settings_yaml["film"]["denoise"].as_bool().unwrap_or(false);

    let film = Arc::new(RwLock::new(Film::new(
        Vector2::new(image_width, image_height),
        Vector2::new(
            settings_yaml["film"]["bucket_width"].as_i64().unwrap() as u32,
            settings_yaml["film"]["bucket_height"].as_i64().unwrap() as u32,
        ),
        Some(crop_start),
        Some(crop_end),
        FilterMethod::from_str(settings_yaml["film"]["filter_method"].as_str().unwrap()).unwrap(),
        settings_yaml["film"]["filter_radius"].as_f64().unwrap(),
    )));

    let camera = camera::Camera::new(
        yaml_array_into_point3(&settings_yaml["camera"]["position"]),
        yaml_array_into_point3(&settings_yaml["camera"]["target"]),
        aspect_ratio,
        settings_yaml["camera"]["fov"].as_f64().unwrap(),
        settings_yaml["camera"]["aperture"].as_f64().unwrap(),
        settings_yaml["camera"]["focal_distance"].as_f64(),
        Bounds {
            p_min: Point2::new(-1.0, -1.0),
            p_max: Point2::new(1.0, 1.0),
        },
        film.clone(),
    );

    let sampler = SobolSampler::new();

    {
        let mut debug_buffer = DEBUG_BUFFER.write().unwrap();
        debug_buffer.width = image_width;
        debug_buffer.height = image_height;
        debug_buffer.buffer = vec![0.0; (image_width as usize) * (image_height as usize) * 3];
    }

    // Start the render threads
    println!("Start rendering...");
    let (threads, receiver) = renderer::render(scene, settings, sampler, Arc::new(camera));

    let cb = ggez::ContextBuilder::new("render_to_image", "ggez")
        .window_setup(WindowSetup {
            title: "Rust Raytracer".to_string(),
            samples: NumSamples::One,
            vsync: true,
            icon: "".to_string(),
            srgb: false,
        })
        .window_mode(WindowMode {
            width: image_width as f32 * window_scale,
            height: image_height as f32 * window_scale,
            maximized: false,
            fullscreen_type: FullscreenType::Windowed,
            borderless: false,
            transparent: false,
            min_width: 1.0,
            min_height: 1.0,
            max_width: 0.0,
            max_height: 0.0,
            resizable: true,
            visible: true,
            resize_on_scale_factor_change: true,
            logical_size: None,
        });

    let (ctx, event_loop) = cb.build()?;
    let running_threads = threads.len();
    let state = MainState::new(film, threads, receiver, running_threads, should_denoise)?;

    event::run(ctx, event_loop, state)
}
