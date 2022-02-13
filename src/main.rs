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
use std::sync::{Arc, RwLock};

use bvh::bvh::BVH;
use ggez::conf::{FullscreenType, NumSamples, WindowMode, WindowSetup};
use ggez::event;
use ggez::event::KeyCode;
use ggez::graphics::{self, Color, DrawParam};
use ggez::{Context, GameResult};
use indicatif::ProgressBar;
use nalgebra::{Point3, Vector2, Vector3};
use yaml_rust::YamlLoader;

use film::{Film, FilterMethod};
use helpers::{
    yaml_array_into_point2, yaml_array_into_point3, yaml_array_into_vector3, yaml_into_u32,
};
use renderer::SETTINGS;
use sampler::{Sampler, SamplerMethod};
use scene::*;

mod film;
mod helpers;
mod photon_mapper;
mod renderer;
mod sampler;
mod scene;

const UP_AXIS: &str = "y";

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
    // Cornell box
    let mut objects: Vec<Box<dyn objects::Object>> = vec![];

    let sphere = objects::Sphere {
        position: Point3::new(0.0, -0.3, 0.0),
        radius: 0.4,
        materials: vec![
            Box::new(materials::FresnelReflection {
                weight: 1.0,
                glossiness: 0.98,
                ior: 1.5,
                reflection: 0.0,
                refraction: 0.7,
                color: Vector3::new(0.5, 0.5, 0.5),
            }),
            // Box::new(materials::Lambert {
            //     weight: 1.0,
            //     color: Vector3::new(0.5, 0.5, 0.5),
            // }),
        ],
        node_index: 0,
    };

    objects.push(Box::new(sphere));

    // add a light
    let light = objects::Rectangle {
        position: Point3::new(-0.3, 0.99, 0.7),
        side_a: Vector3::new(0.6, 0.0, 0.0),
        side_b: Vector3::new(0.0, 0.0, 0.6),
        materials: vec![Box::new(materials::Light {
            weight: 1.0,
            intensity: 190.0,
            color: Vector3::new(1.0, 1.0, 1.0),
        })],
        node_index: 0,
    };
    objects.push(Box::new(light));

    // add a mirror
    let mirror = objects::Rectangle {
        position: Point3::new(-1.4, -1.4, 0.0),
        side_a: Vector3::new(3.0, 0.0, -0.8),
        side_b: Vector3::new(0.0, 3.0, 0.4),
        materials: vec![Box::new(materials::Reflection {
            weight: 1.0,
            glossiness: 1.0,
        })],
        node_index: 0,
    };
    objects.push(Box::new(mirror));

    let light_sphere = objects::Sphere {
        position: Point3::new(0.8, -0.8, 0.9),
        radius: 0.12,
        materials: vec![
            Box::new(materials::Light {
                weight: 1.0,
                intensity: 20.0,
                color: Vector3::new(1.0, 0.0, 0.0),
            }),
            // Box::new(materials::Lambert {
            //     weight: 1.0,
            //     color: Vector3::new(0.5, 0.5, 0.5),
            // }),
        ],
        node_index: 0,
    };

    objects.push(Box::new(light_sphere));

    let light_sphere2 = objects::Sphere {
        position: Point3::new(0.8, -0.8, -0.3),
        radius: 0.12,
        materials: vec![
            Box::new(materials::Light {
                weight: 1.0,
                intensity: 20.0,
                color: Vector3::new(0.0, 0.0, 1.0),
            }),
            // Box::new(materials::Lambert {
            //     weight: 1.0,
            //     color: Vector3::new(0.5, 0.5, 0.5),
            // }),
        ],
        node_index: 0,
    };

    objects.push(Box::new(light_sphere2));

    let light_sphere3 = objects::Sphere {
        position: Point3::new(-0.8, -0.8, -0.1),
        radius: 0.12,
        materials: vec![
            Box::new(materials::Light {
                weight: 1.0,
                intensity: 30.0,
                color: Vector3::new(1.0, 1.0, 1.0),
            }),
            // Box::new(materials::Lambert {
            //     weight: 1.0,
            //     color: Vector3::new(0.5, 0.5, 0.5),
            // }),
        ],
        node_index: 0,
    };

    objects.push(Box::new(light_sphere3));

    let light_sphere4 = objects::Sphere {
        position: Point3::new(-0.8, -0.8, 0.5),
        radius: 0.22,
        materials: vec![
            Box::new(materials::FresnelReflection {
                weight: 1.0,
                glossiness: 0.99,
                ior: 1.5,
                reflection: 0.4,
                refraction: 0.2,
                color: Vector3::new(0.7, 0.7, 0.7),
            }),
            // Box::new(materials::Lambert {
            //     weight: 1.0,
            //     color: Vector3::new(0.5, 0.5, 0.5),
            // }),
        ],
        node_index: 0,
    };

    objects.push(Box::new(light_sphere4));

    //
    // let light_1 = lights::Light {
    //     position: Point3::new(0.0, 0.8, 0.0),
    //     intensity: 0.9,
    //     color: Vector3::new(1.0, 1.0, 1.0), // white
    // };

    ////////// load model

    let (models, _) = tobj::load_obj("./scene/box.obj", true).expect("Failed to load file");

    //let models: Vec<Model> = vec!();

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
        println!(
            "model[{}].expected_triangles: {}",
            i,
            mesh.indices.len() / 3
        );
        println!("model[{}].faces: {}", i, mesh.num_face_indices.len());
        println!("model[{}].normals: {}", i, mesh.normals.len() / 3);

        assert_eq!(mesh.indices.len() % 3, 0);

        let bar = ProgressBar::new((mesh.indices.len() / 3) as u64);
        for v in 0..mesh.indices.len() / 3 {
            let v0 = mesh.indices[3 * v] as usize;
            let v1 = mesh.indices[3 * v + 1] as usize;
            let v2 = mesh.indices[3 * v + 2] as usize;

            let (p0, p1, p2) = match UP_AXIS {
                // stored as x y z, where y is up
                "y" => (
                    Point3::new(
                        mesh.positions[3 * v0] as f64,
                        mesh.positions[3 * v0 + 1] as f64,
                        mesh.positions[3 * v0 + 2] as f64,
                    ),
                    Point3::new(
                        mesh.positions[3 * v1] as f64,
                        mesh.positions[3 * v1 + 1] as f64,
                        mesh.positions[3 * v1 + 2] as f64,
                    ),
                    Point3::new(
                        mesh.positions[3 * v2] as f64,
                        mesh.positions[3 * v2 + 1] as f64,
                        mesh.positions[3 * v2 + 2] as f64,
                    ),
                ),
                // stored as x z y, where z is up
                _ => (
                    Point3::new(
                        mesh.positions[3 * v0] as f64,
                        mesh.positions[3 * v0 + 2] as f64,
                        mesh.positions[3 * v0 + 1] as f64,
                    ),
                    Point3::new(
                        mesh.positions[3 * v1] as f64,
                        mesh.positions[3 * v1 + 2] as f64,
                        mesh.positions[3 * v1 + 1] as f64,
                    ),
                    Point3::new(
                        mesh.positions[3 * v2] as f64,
                        mesh.positions[3 * v2 + 2] as f64,
                        mesh.positions[3 * v2 + 1] as f64,
                    ),
                ),
            };

            let p0_normal = Vector3::new(
                mesh.normals[3 * v0] as f64,
                mesh.normals[3 * v0 + 1] as f64,
                mesh.normals[3 * v0 + 2] as f64,
            );
            let p1_normal = Vector3::new(
                mesh.normals[3 * v1] as f64,
                mesh.normals[3 * v1 + 1] as f64,
                mesh.normals[3 * v1 + 2] as f64,
            );
            let p2_normal = Vector3::new(
                mesh.normals[3 * v2] as f64,
                mesh.normals[3 * v2 + 1] as f64,
                mesh.normals[3 * v2 + 2] as f64,
            );

            let triangle = objects::Triangle::new(
                p0,
                p1,
                p2,
                p0_normal,
                p1_normal,
                p2_normal,
                vec![Box::new(materials::Lambert {
                    weight: 1.0,
                    color: Vector3::new(0.5, 0.5, 0.5),
                })],
            );

            objects.push(Box::new(triangle));

            bar.inc(1);
        }

        bar.finish();
    }

    // Get settings
    let mut file = File::open("settings.yaml").expect("Unable to open file");
    let mut contents = String::new();

    file.read_to_string(&mut contents)
        .expect("Unable to read file");
    let settings_yaml = &YamlLoader::load_from_str(&contents).unwrap()[0];

    // Build scene
    println!("Building BVH...");
    let bvh = BVH::build(&mut objects);
    println!("Done!");

    let scene = scene::Scene::new(
        yaml_array_into_vector3(&settings_yaml["scene"]["background_color"]),
        vec![],
        objects,
        bvh,
    );

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
