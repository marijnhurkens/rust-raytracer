extern crate bvh;
extern crate ggez;
extern crate image;
#[macro_use]
extern crate lazy_static;
extern crate nalgebra;
extern crate rand;
extern crate serde;
extern crate serde_yaml;

use std::env;
use std::fs;
use std::sync::{Arc, RwLock};

use bvh::bvh::BVH;
use ggez::{Context, GameResult};
use ggez::conf::{FullscreenType, NumSamples, WindowMode, WindowSetup};
use ggez::event;
use ggez::graphics::{self, Color, Drawable, DrawParam};
use nalgebra::{Point3, Vector3};

use scene::*;

mod helpers;
mod renderer;
mod scene;

const IMAGE_WIDTH: u32 = 1600;
const IMAGE_HEIGHT: u32 = 800;
const OUTPUT: &str = "window";

lazy_static! {
    static ref IMAGE_BUFFER: Arc<RwLock<image::RgbaImage>> = Arc::new(RwLock::new(
        image::ImageBuffer::new(IMAGE_WIDTH, IMAGE_HEIGHT)
    ));
}

struct MainState {
    canvas: graphics::Canvas,
    text: graphics::Text,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let canvas = graphics::Canvas::with_window_size(ctx)?;
        let font = graphics::Font::default();
        let text = graphics::Text::new(("Hello world!", font, 24.0));
        Ok(MainState { canvas, text })
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        dbg!(ggez::timer::fps(_ctx));
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let image = ggez::graphics::Image::from_rgba8(ctx, IMAGE_WIDTH as u16, IMAGE_HEIGHT as u16, &IMAGE_BUFFER.read().unwrap().clone().into_raw())?;

        // //now lets render our scene once in the top left and in the bottom
        // // right
        let window_size = graphics::size(ctx);
        let image_ratio = IMAGE_WIDTH as f32 / IMAGE_HEIGHT as f32;
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
        // graphics::draw(
        //     ctx,
        //     &self.canvas,
        //     DrawParam::default()
        //         .dest(Point2::new(400.0, 300.0))
        //         .scale(scale),
        // )?;
        graphics::present(ctx)?;

        Ok(())
    }

    fn resize_event(&mut self, ctx: &mut Context, width: f32, height: f32) {
        let new_rect = graphics::Rect::new(0.0, 0.0, width, height);
        graphics::set_screen_coordinates(ctx, new_rect).unwrap();
    }
}

fn main() -> GameResult {
    let camera = camera::Camera::new(
        Point3::new(0.5, 0.0, 1.0),
        Point3::new(0.0, 0.0, 0.0),
        90.0,
    );


    let _sphere = objects::Sphere {
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

    let light = lights::Light {
        position: Point3::new(7.0, 2.0, 0.0),
        intensity: 1.0,
        color: Vector3::new(1.0, 1.0, 1.0), // white
    };

    let light_1 = lights::Light {
        position: Point3::new(-1.0, 0.0, 8.0),
        intensity: 1.0,
        color: Vector3::new(1.0, 1.0, 1.0), // white
    };

    // // Setup a basic test scene
    let mut spheres_boxed: Vec<Box<dyn objects::Object>> = vec![];

    //spheres_boxed.push(Box::new(_sphere));

    let spacing = 0.3;
    let radius = 0.1;
    let xcount = 3;
    let ycount = 3;
    let zcount = 3;

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
                let sphere = objects::Sphere {
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

    let plane = objects::Plane {
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

    let plane2 = objects::Plane {
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

    //spheres_boxed.push(Box::new(plane));
    //spheres_boxed.push(Box::new(plane2));

    let triangle = objects::Triangle::new(
        Point3::new(-1.0, -1.0, 0.0),
        Point3::new(1.0, -1.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
        vec![
            // Box::new(materials::Lambert {
            //     color: Vector3::new(0.9, 0.5, 0.5),
            //     weight: 1.0,
            // }),
            Box::new(materials::FresnelReflection {
                weight: 1.0,
                glossiness: 0.9,
                ior: 1.5,
                reflection: 1.0,
                refraction: 0.0,
                color: Vector3::new(0.5, 0.5, 0.5),
            }),
        ],
    );

    spheres_boxed.push(Box::new(triangle));

    let bvh = BVH::build(&mut spheres_boxed);

    let scene = scene::Scene::new(
        Vector3::new(0.7, 0.7, 0.9),
        vec![light, light_1],
        spheres_boxed,
        bvh,
    );


    let settings = renderer::Settings {
        thread_count: 8,
        bucket_width: 32,
        bucket_height: 32,
        depth_limit: 8,
        samples: 40,
    };

    // Start the render threads
    let (threads, thread_senders) = renderer::render(camera, Arc::new(scene), settings);

    let cb = ggez::ContextBuilder::new("render_to_image", "ggez")
        .window_setup(WindowSetup {
            title: "Test".to_string(),
            samples: NumSamples::Zero,
            vsync: true,
            icon: "".to_string(),
            srgb: false,
        })
        .window_mode(WindowMode {
            width: IMAGE_WIDTH as f32,
            height: IMAGE_HEIGHT as f32,
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
    let state = &mut MainState::new(ctx)?;
    event::run(ctx, event_loop, state)
}
