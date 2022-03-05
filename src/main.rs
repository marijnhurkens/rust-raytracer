extern crate bvh;
extern crate ggez;
extern crate image;
extern crate indicatif;
#[macro_use]
extern crate lazy_static;
extern crate nalgebra;
extern crate rand;
extern crate sobol;
extern crate tobj;
extern crate yaml_rust;

use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::{Arc, RwLock};

use ggez::conf::{FullscreenType, NumSamples, WindowMode, WindowSetup};
use ggez::event;
use ggez::event::KeyCode;
use ggez::graphics::{self, Color, DrawParam};
use ggez::{Context, GameResult};
use nalgebra::{Vector2};
use yaml_rust::YamlLoader;

use film::{Film, FilterMethod};
use helpers::{
    yaml_array_into_point2, yaml_array_into_point3, yaml_into_u32,
};
use objects::Object;
use renderer::SETTINGS;
use sampler::{Sampler, SamplerMethod};
use scene::*;

mod camera;
mod film;
mod helpers;
mod renderer;
mod sampler;
mod scene;

struct MainState {
    film: Arc<RwLock<Film>>,
}

impl MainState {
    fn new(_ctx: &mut Context, film: Arc<RwLock<Film>>) -> GameResult<MainState> {
        Ok(MainState { film })
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        if !ggez::timer::check_update_time(ctx, 1) {
            return Ok(());
        }

        let film = self.film.read().unwrap();
        let image_width = film.image_size.x;
        let image_height = film.image_size.y;

        let denoise = true;

        let mut output = vec![0u8; image_width as usize * image_height as usize * 4];

        if ggez::input::keyboard::is_key_pressed(ctx, KeyCode::N) {
            let mut i = 0;
            film.pixels.clone().iter().for_each(|pixel| {
                output[i] = (pixel.normal.x * 255.0) as u8;
                output[i + 1] = (pixel.normal.y * 255.0) as u8;
                output[i + 2] = (pixel.normal.z * 255.0) as u8;
                output[i + 3] = 255;
                i += 4;
            });
        } else if denoise && ggez::input::keyboard::is_key_pressed(ctx, KeyCode::D) {
            println!("denoise");

            let mut normal_map = vec![0f32; image_width as usize * image_height as usize * 3];
            let mut i = 0;
            film.pixels.clone().iter().for_each(|pixel| {
                normal_map[i] = pixel.normal.x as f32;
                normal_map[i + 1] = pixel.normal.y as f32;
                normal_map[i + 2] = pixel.normal.z as f32;
                i += 3;
            });

            let temp = film.image_buffer.clone();
            let input_img: Vec<f32> = temp
                .into_raw()
                .iter()
                .map(|val| (*val as f32) / 255.0)
                .collect();
            let mut filter_output = vec![0.0f32; input_img.len()];

            let device = oidn::Device::new();

            oidn::RayTracing::new(&device)
                .srgb(false)
                //.albedo_normal(&input_img[..], &normal_map[..])
                //.clean_aux(true)
                .image_dimensions(image_width as usize, image_height as usize)
                .filter(&input_img[..], &mut filter_output[..])
                .expect("Filter config error!");

            if let Err(e) = device.get_error() {
                println!("Error denosing image: {}", e.1);
            }

            let mut i = 0;
            for chunk in filter_output.chunks(3) {
                output[i] = (chunk[0] * 255.0) as u8;
                output[i + 1] = (chunk[1] * 255.0) as u8;
                output[i + 2] = (chunk[2] * 255.0) as u8;
                output[i + 3] = 255;
                i += 4;
            }
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

        let image = ggez::graphics::Image::from_rgba8(
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
    // let mut objects: Vec<objects::Object> = vec![];
    //
    // // add a light
    // let light = objects::Rectangle {
    //     position: Point3::new(0.5, 2.50, -0.7),
    //     side_a: Vector3::new(3.6, 0.0, 0.0),
    //     side_b: Vector3::new(0.0, 0.0, 0.6),
    //     materials: vec![Box::new(materials::Light {
    //         weight: 1.0,
    //         intensity: 190.0,
    //         color: Vector3::new(1.0, 1.0, 1.0),
    //     })],
    //     node_index: 0,
    // };
    // objects.push(Object::Rectangle(light));

    // Load scene from yaml file
    let scene = scene::Scene::load_from_file(Path::new("./scene/cornell/scene.yaml"));

    // Get settings from yaml file
    let mut file = File::open("./scene/cornell/render_settings.yaml").expect("Unable to open file");
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
    renderer::render(Arc::new(scene), film.clone());

    let cb = ggez::ContextBuilder::new("render_to_image", "ggez")
        .window_setup(WindowSetup {
            title: "Test".to_string(),
            samples: NumSamples::Zero,
            vsync: true,
            icon: "".to_string(),
            srgb: false,
        })
        .window_mode(WindowMode {
            width: image_width as f32 / 1.5,
            height: image_height as f32 / 1.5,
            maximized: false,
            fullscreen_type: FullscreenType::Windowed,
            borderless: false,
            min_width: 0.0,
            min_height: 0.0,
            max_width: 0.0,
            max_height: 0.0,
            resizable: true,
        });
    let (ctx, event_loop) = &mut cb.build()?;
    let state = &mut MainState::new(ctx, film)?;
    event::run(ctx, event_loop, state)
}
