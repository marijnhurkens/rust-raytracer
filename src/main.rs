extern crate bvh;
extern crate ggez;
extern crate image;
#[macro_use]
extern crate lazy_static;
extern crate nalgebra;
extern crate rand;
extern crate tobj;
extern crate indicatif;
extern crate sobol;
extern crate yaml_rust;

use std::sync::{Arc, RwLock};
use bvh::bvh::BVH;
use ggez::{Context, GameResult};
use ggez::conf::{FullscreenType, NumSamples, WindowMode, WindowSetup};
use ggez::event;
use ggez::graphics::{self, Color, DrawParam};
use nalgebra::{Point3, Vector3, Vector2};
use scene::*;
use indicatif::ProgressBar;
use renderer::{SETTINGS};
use sampler::{Sampler, Method};
use film::{Film, FilterMethod};
use std::time::Instant;
use std::thread::JoinHandle;
use yaml_rust::YamlLoader;
use std::fs::File;
use std::io::Read;
use helpers::{yaml_array_into_point3, yaml_into_u32};

mod helpers;
mod renderer;
mod film;
mod scene;
mod sampler;
mod photon_mapper;

const OUTPUT: &str = "window";
const UP_AXIS: &str = "y";

struct MainState {
    canvas: graphics::Canvas,
    film: Arc<RwLock<Film>>,
}

impl MainState {
    fn new(ctx: &mut Context, film: Arc<RwLock<Film>>) -> GameResult<MainState> {
        let canvas = graphics::Canvas::with_window_size(ctx)?;

        Ok(MainState { canvas, film })
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
        let image = ggez::graphics::Image::from_rgba8(ctx, image_width as u16, image_height as u16, &film.image_buffer.clone().into_raw())?;

        // now lets render our scene once in the top left and in the bottom right
        let window_size = graphics::size(ctx);
        let image_ratio = image_width as f32 / image_height as f32;
        let window_ratio = window_size.0 as f32 / window_size.1 as f32;

        let scale = if window_ratio > image_ratio {
            // window is wider, use max height
            let scale = window_size.1 as f32 / image.height() as f32;
            ggez::mint::Vector2 {
                x: scale,
                y: scale,
            }
        } else {
            // window is narrower, use max width
            let scale = window_size.0 as f32 / image.width() as f32;
            ggez::mint::Vector2 {
                x: scale,
                y: scale,
            }
        };

        // let scale = Vector2::new(1.0, 1.0);
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
    // Cornell box
    let mut objects: Vec<Box<dyn objects::Object>> = vec![];

    let sphere = objects::Sphere {
        position: Point3::new(0.3, -0.3, 0.2),
        radius: 0.4,
        materials: vec![
            Box::new(materials::FresnelReflection {
                weight: 1.0,
                glossiness: 0.98,
                ior: 1.5,
                reflection: 0.8,
                refraction: 0.0,
                color: Vector3::new(0.0, 0.0, 0.8),
            }),
        ],
        node_index: 0,
    };

    let sphere_2 = objects::Sphere {
        position: Point3::new(-0.3, -0.2, -0.2),
        radius: 0.3,
        materials: vec![
            Box::new(materials::FresnelReflection {
                weight: 1.0,
                glossiness: 0.90,
                ior: 1.5,
                reflection: 0.95,
                refraction: 0.0,
                color: Vector3::new(0.8, 0.0, 0.2),
            }),
        ],
        node_index: 0,
    };

    let sphere_3 = objects::Sphere {
        position: Point3::new(-0.3, -0.4, 0.7),
        radius: 0.3,
        materials: vec![
            Box::new(materials::FresnelReflection {
                weight: 1.0,
                glossiness: 0.98,
                ior: 1.5,
                reflection: 0.95,
                refraction: 0.9,
                color: Vector3::new(0.8, 0.8, 0.8),
            }),
        ],
        node_index: 0,
    };

    objects.push(Box::new(sphere));
    objects.push(Box::new(sphere_2));
    objects.push(Box::new(sphere_3));

    let sphere_light = objects::Sphere {
        position: Point3::new(0.0, 1.6, 0.0),
        radius: 0.8,
        materials: vec![
            Box::new(materials::Light {
                weight: 1.0,
                intensity: 20.0,
                color: Vector3::new(0.8, 0.8, 0.8),
            }),
        ],
        node_index: 0,
    };

    objects.push(Box::new(sphere_light));


    let light = lights::Light {
        position: Point3::new(0.0, 0.8, 0.0),
        intensity: 0.9,
        color: Vector3::new(1.0, 1.0, 1.0), // white
    };


    ////////// load model

    let (models, materials) = tobj::load_obj("./scene/box.obj", true)
        .expect("Failed to load file");

    for (i, m) in models.iter().enumerate() {
        let mesh = &m.mesh;
        println!("model[{}].name = \'{}\'", i, m.name);
        println!("model[{}].mesh.material_id = {:?}", i, mesh.material_id);

        println!(
            "Size of model[{}].num_face_indices: {}",
            i,
            mesh.num_face_indices.len()
        );

        // Normals and texture coordinates are also loaded, but not printed in this example
        println!("model[{}].vertices: {}", i, mesh.positions.len() / 3);
        println!("model[{}].indices: {}", i, mesh.indices.len());
        println!("model[{}].expected_triangles: {}", i, mesh.indices.len() / 3);
        println!("model[{}].faces: {}", i, mesh.num_face_indices.len());
        println!("model[{}].normals: {}", i, mesh.normals.len() / 3);

        assert!(mesh.indices.len() % 3 == 0);
        //let v = 0;

        let bar = ProgressBar::new((mesh.indices.len() / 3) as u64);
        for v in 0..mesh.indices.len() / 3 {
            let v0 = mesh.indices[3 * v] as usize;
            let v1 = mesh.indices[3 * v + 1] as usize;
            let v2 = mesh.indices[3 * v + 2] as usize;

            // dbg!(v,v0,v1,v2);

            let (p0, p1, p2) = match UP_AXIS {
                // stored as x y z, where y is up
                "y" => (
                    Point3::new(mesh.positions[3 * v0] as f64, mesh.positions[3 * v0 + 1] as f64, mesh.positions[3 * v0 + 2] as f64),
                    Point3::new(mesh.positions[3 * v1] as f64, mesh.positions[3 * v1 + 1] as f64, mesh.positions[3 * v1 + 2] as f64),
                    Point3::new(mesh.positions[3 * v2] as f64, mesh.positions[3 * v2 + 1] as f64, mesh.positions[3 * v2 + 2] as f64)
                ),
                // stored as x z y, where z is up
                _ => (
                    Point3::new(mesh.positions[3 * v0] as f64, mesh.positions[3 * v0 + 2] as f64, mesh.positions[3 * v0 + 1] as f64),
                    Point3::new(mesh.positions[3 * v1] as f64, mesh.positions[3 * v1 + 2] as f64, mesh.positions[3 * v1 + 1] as f64),
                    Point3::new(mesh.positions[3 * v2] as f64, mesh.positions[3 * v2 + 2] as f64, mesh.positions[3 * v2 + 1] as f64)
                )
            };

            let p0_normal = Vector3::new(mesh.normals[3 * v0] as f64, mesh.normals[3 * v0 + 1] as f64, mesh.normals[3 * v0 + 2] as f64);
            let p1_normal = Vector3::new(mesh.normals[3 * v1] as f64, mesh.normals[3 * v1 + 1] as f64, mesh.normals[3 * v1 + 2] as f64);
            let p2_normal = Vector3::new(mesh.normals[3 * v2] as f64, mesh.normals[3 * v2 + 1] as f64, mesh.normals[3 * v2 + 2] as f64);

            let triangle = objects::Triangle::new(
                p0,
                p1,
                p2,
                p0_normal,
                p1_normal,
                p2_normal,
                vec![
                    Box::new(materials::Lambert {
                        color: Vector3::new(0.5, 0.5, 0.5),

                        weight: 1.0,
                    }),
                ],
            );

            objects.push(Box::new(triangle));

            bar.inc(1);
        }

        bar.finish();
    }


    // Build scene
    let bvh = BVH::build(&mut objects);

    let scene = scene::Scene::new(
        Vector3::new(0.0, 0.0, 0.0),
        vec![],
        objects,
        bvh,
    );

    // Get settings
    let mut file = File::open("settings.yaml").expect("Unable to open file");
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

    let film = Arc::new(RwLock::new(Film::new(
        Vector2::new(image_width, image_height),
        Vector2::new(
            settings_yaml["film"]["bucket_width"].as_i64().unwrap() as u32,
            settings_yaml["film"]["bucket_height"].as_i64().unwrap() as u32,
        ),
        settings_yaml["film"]["film_size"].as_i64().unwrap() as u32,
        None,
        None,
        FilterMethod::Box,
    )));

    {
        let mut settings = SETTINGS.write().unwrap();
        settings.max_samples = yaml_into_u32(&settings_yaml["sampler"]["max_samples"]);
        settings.min_samples = yaml_into_u32(&settings_yaml["sampler"]["min_samples"]);
        settings.depth_limit = yaml_into_u32(&settings_yaml["renderer"]["depth_limit"]);
        settings.thread_count = yaml_into_u32(&settings_yaml["renderer"]["threads"]);
        settings.sampler = Sampler::new(
            Method::Sobol,
            camera,
            image_width,
            image_height,
        )
    }

    // Start the render threads
    let (threads, thread_senders) = renderer::render(Arc::new(scene), film.clone());

    let cb = ggez::ContextBuilder::new("render_to_image", "ggez")
        .window_setup(WindowSetup {
            title: "Test".to_string(),
            samples: NumSamples::Zero,
            vsync: true,
            icon: "".to_string(),
            srgb: false,
        })
        .window_mode(WindowMode {
            width: image_width as f32,
            height: image_height as f32,
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
    let state = &mut MainState::new(ctx, film.clone())?;
    event::run(ctx, event_loop, state)
}
