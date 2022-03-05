use std::fs::File;
use std::io::Read;
use std::path::Path;

use bvh::bvh::BVH;
use indicatif::ProgressBar;
use nalgebra::{Point3, Vector3};
use yaml_rust::YamlLoader;

use Object;

pub mod lights;
pub mod materials;
pub mod objects;

pub struct Scene {
    pub bg_color: Vector3<f64>,
    pub objects: Vec<objects::Object>,
    pub bvh: BVH,
    pub lights: Vec<lights::Light>,
}

impl Scene {
    pub fn new(
        bg_color: Vector3<f64>,
        lights: Vec<lights::Light>,
        objects: Vec<objects::Object>,
        bvh: BVH,
    ) -> Scene {
        Scene {
            bg_color,
            objects,
            lights,
            bvh,
        }
    }

    pub fn load_from_file(path: &Path) -> Scene {
        println!("Load scene from {:?}", path.display());
        let mut file = File::open(path).expect("Unable to open file");
        let mut contents = String::new();

        file.read_to_string(&mut contents)
            .expect("Unable to read file");
        let scene_yaml = &YamlLoader::load_from_str(&contents).unwrap()[0];

        let world_model_file = path
            .parent()
            .unwrap()
            .join(Path::new(scene_yaml["world"]["file"].as_str().unwrap()));
        let up_axis = scene_yaml["world"]["up_axis"].as_str().unwrap();

        let mut objects = load_model(world_model_file.as_path(), up_axis);

        // Build scene
        println!("Building BVH...");
        let bvh = BVH::build(&mut objects);
        println!("Done!");

        let lights = vec![];

        println!("Scene loaded.");

        Scene {
            bg_color: Vector3::new(1.0, 1.0, 1.0),
            objects,
            lights,
            bvh,
        }
    }

    pub fn push_object(&mut self, o: objects::Object) {
        self.objects.push(o);
    }
}

fn load_model(model_file: &Path, up_axis: &str) -> Vec<objects::Object> {
    dbg!(model_file);
    let (models, materials) = tobj::load_obj(model_file, true).expect("Failed to load file");

    dbg!(&materials);
    let mut triangles = vec![];

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

        let material = &materials[mesh.material_id.unwrap()];

        for v in 0..mesh.indices.len() / 3 {
            let v0 = mesh.indices[3 * v] as usize;
            let v1 = mesh.indices[3 * v + 1] as usize;
            let v2 = mesh.indices[3 * v + 2] as usize;

            let (p0, p1, p2) = match up_axis {
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

            let color = Vector3::new(
                material.diffuse[0] as f64,
                material.diffuse[1] as f64,
                material.diffuse[2] as f64,
            );

            let reflection = material.specular[0] as f64;

            let triangle = objects::Triangle::new(
                p0,
                p1,
                p2,
                p0_normal,
                p1_normal,
                p2_normal,
                vec![Box::new(materials::FresnelReflection {
                    weight: 1.0,
                    color,
                    glossiness: (material.shininess/1000.0) as f64,
                    ior: material.optical_density as f64,
                    reflection,
                    refraction: 1.0 - material.dissolve as f64,
                })],
            );

            triangles.push(Object::Triangle(triangle));

            bar.inc(1);
        }

        bar.finish();
    }

    triangles
}
