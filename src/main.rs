extern crate bitflags;
extern crate bvh;
extern crate clap;
extern crate core;
extern crate ggez;
extern crate image;
extern crate indicatif;
#[macro_use]
extern crate lazy_static;
extern crate nalgebra;
extern crate num_traits;
extern crate rand;
extern crate sobol;
extern crate tobj;
extern crate yaml_rust;

use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, RwLock};
use std::thread::JoinHandle;

use clap::Parser;
use ggez::conf::{FullscreenType, NumSamples, WindowMode, WindowSetup};
use ggez::event::{run, KeyCode};
use ggez::graphics::{self, Color, DrawParam};
use ggez::input::keyboard;
use ggez::{event, GameError};
use ggez::{Context, GameResult};
use nalgebra::Vector2;
use yaml_rust::YamlLoader;

use denoise::denoise;
use film::{Film, FilterMethod};
use helpers::{yaml_array_into_point2, yaml_array_into_point3, yaml_into_u32};
use objects::Object;
use renderer::{ThreadMessage, SETTINGS};
use sampler::{Sampler, SamplerMethod};

mod bsdf;
mod camera;
mod denoise;
mod film;
mod helpers;
mod lights;
mod materials;
mod objects;
mod renderer;
mod sampler;
mod scene;
mod surface_interaction;
mod tracer;

#[derive(Parser, Debug)]
struct Args {
    scene_file: Option<String>,
    settings_file: Option<String>,
}

struct MainState {
    redraw: bool,
    film: Arc<RwLock<Film>>,
    threads: Vec<JoinHandle<()>>,
    receiver: Receiver<ThreadMessage>,
    running_threads: usize,
    finished: bool,
    denoised: bool,
    debug_normals: bool,
}

impl MainState {
    fn new(
        film: Arc<RwLock<Film>>,
        threads: Vec<JoinHandle<()>>,
        receiver: Receiver<ThreadMessage>,
        running_threads: usize,
    ) -> GameResult<MainState> {
        Ok(MainState {
            redraw: true,
            film,
            threads,
            receiver,
            running_threads,
            finished: false,
            denoised: false,
            debug_normals: false,
        })
    }
}

impl event::EventHandler<GameError> for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        if !ggez::timer::check_update_time(ctx, 1) {
            return Ok(());
        }

        self.redraw = true;

        let message = self.receiver.try_recv();
        if let Ok(message) = message {
            if message.finished {
                self.running_threads -= 1;
            }
        }

        if self.running_threads == 0 && !self.finished {
            println!("All work is done.");
            self.finished = true;

            if !self.denoised {
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

        if keyboard::is_key_pressed(ctx, KeyCode::N) && !keyboard::is_key_repeated(ctx) {
            self.debug_normals = !self.debug_normals;
            println!("Normals debug view {:?}.", self.debug_normals);
        }

        let film = self.film.read().unwrap();
        let image_width = film.image_size.x;
        let image_height = film.image_size.y;
        let mut output = vec![0u8; image_width as usize * image_height as usize * 4];

        if self.debug_normals {
            let mut i = 0;
            film.pixels.clone().iter().for_each(|pixel| {
                output[i] = (pixel.normal.x * 255.0) as u8;
                output[i + 1] = (pixel.normal.y * 255.0) as u8;
                output[i + 2] = (pixel.normal.z * 255.0) as u8;
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

        let image = graphics::Image::from_rgba8(
            ctx,
            image_width as u16,
            image_height as u16,
            &output,
        )?;

        // now lets render our scene once in the top left and in the bottom right
        let window_size = graphics::size(ctx);
        let image_ratio = image_width as f32 / image_height as f32;
        let window_ratio = window_size.0 as f32 / window_size.1 as f32;

        let scale = if window_ratio > image_ratio {
            // window is wider, use max height
            let scale = window_size.1 as f32 / image.height() as f32;
            ggez::mint::Vector2 { x: scale, y: scale }
        } else {
            // window is narrower, use max width
            let scale = window_size.0 as f32 / image.width() as f32;
            ggez::mint::Vector2 { x: scale, y: scale }
        };

        graphics::set_canvas(ctx, None);
        graphics::clear(ctx, Color::new(0.0, 0.0, 0.0, 1.0));
        graphics::draw(
            ctx,
            &image,
            DrawParam::default()
                .dest(ggez::mint::Point2 { x: 0.0, y: 0.0 })
                .scale(scale),
        )?;
        graphics::present(ctx)?;

        Ok(())
    }

    fn resize_event(&mut self, ctx: &mut Context, width: f32, height: f32) {
        let new_rect = graphics::Rect::new(0.0, 0.0, width, height);
        graphics::set_screen_coordinates(ctx, new_rect).unwrap();
    }
}

fn main() -> GameResult {
    let args = Args::parse();

    // Load scene from yaml file
    let scene = scene::Scene::load_from_file(Path::new(&args.scene_file.unwrap()));

    // Get settings from yaml file
    let mut file = File::open(&args.settings_file.unwrap()).expect("Unable to open file");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Unable to read file");
    let settings_yaml = &YamlLoader::load_from_str(&contents).unwrap()[0];

    let camera = camera::Camera::new(
        yaml_array_into_point3(&settings_yaml["camera"]["position"]),
        yaml_array_into_point3(&settings_yaml["camera"]["target"]),
        settings_yaml["camera"]["fov"].as_f64().unwrap(),
    );

    let image_width = settings_yaml["film"]["image_width"].as_i64().unwrap() as u32;
    let image_height = settings_yaml["film"]["image_height"].as_i64().unwrap() as u32;

    let crop_start = if !settings_yaml["film"]["crop"]["start"].is_badvalue() {
        Some(yaml_array_into_point2(
            &settings_yaml["film"]["crop"]["start"],
        ))
    } else {
        None
    };

    let crop_end = if !settings_yaml["film"]["crop"]["end"].is_badvalue() {
        Some(yaml_array_into_point2(
            &settings_yaml["film"]["crop"]["end"],
        ))
    } else {
        None
    };

    let film = Arc::new(RwLock::new(Film::new(
        Vector2::new(image_width, image_height),
        Vector2::new(
            settings_yaml["film"]["bucket_width"].as_i64().unwrap() as u32,
            settings_yaml["film"]["bucket_height"].as_i64().unwrap() as u32,
        ),
        crop_start,
        crop_end,
        FilterMethod::from_str(settings_yaml["film"]["filter_method"].as_str().unwrap()).unwrap(),
        settings_yaml["film"]["filter_radius"].as_f64().unwrap(),
    )));

    {
        let mut settings = SETTINGS.write().unwrap();
        settings.max_samples = yaml_into_u32(&settings_yaml["sampler"]["max_samples"]);
        settings.min_samples = yaml_into_u32(&settings_yaml["sampler"]["min_samples"]);
        settings.depth_limit = yaml_into_u32(&settings_yaml["renderer"]["depth_limit"]);
        settings.thread_count = yaml_into_u32(&settings_yaml["renderer"]["threads"]);
        settings.sampler = Sampler::new(
            SamplerMethod::from_str(settings_yaml["sampler"]["method"].as_str().unwrap()).unwrap(),
            camera,
            image_width,
            image_height,
        )
    }

    // Start the render threads
    println!("Start rendering...");
    let (threads, receiver) = renderer::render(scene, film.clone());

    let cb = ggez::ContextBuilder::new("render_to_image", "ggez")
        .window_setup(WindowSetup {
            title: "Rust Raytracer".to_string(),
            samples: NumSamples::One,
            vsync: true,
            icon: "".to_string(),
            srgb: false,
        })
        .window_mode(WindowMode {
            width: image_width as f32 * 1.5,
            height: image_height as f32 * 1.5,
            maximized: false,
            fullscreen_type: FullscreenType::Windowed,
            borderless: false,
            min_width: 0.0,
            min_height: 0.0,
            max_width: 0.0,
            max_height: 0.0,
            resizable: true,
            visible: true,
            resize_on_scale_factor_change: true,
        });

    let (ctx, event_loop) = cb.build()?;
    let running_threads = threads.len();
    let state = MainState::new(film, threads, receiver, running_threads)?;

    event::run(ctx, event_loop, state)
}
