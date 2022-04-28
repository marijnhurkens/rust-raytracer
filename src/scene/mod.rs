use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::Arc;

use bvh::bvh::BVH;
use indicatif::ProgressBar;
use nalgebra::{Point3, Vector3};
use tobj::Mesh;
use yaml_rust::YamlLoader;

use lights::{Light, PointLight};
use materials::matte::MatteMaterial;
use materials::Material;
use materials::plastic::PlasticMaterial;
use objects::triangle::Triangle;
use Object;

pub struct Scene {
    pub bg_color: Vector3<f64>,
    pub objects: Vec<Object>,
    pub meshes: Vec<Arc<Mesh>>,
    pub bvh: BVH,
    pub lights: Vec<Light>,
}

impl Scene {
    pub fn new(
        bg_color: Vector3<f64>,
        lights: Vec<Light>,
        objects: Vec<Object>,
        meshes: Vec<Arc<Mesh>>,
        bvh: BVH,
    ) -> Scene {
        Scene {
            bg_color,
            objects,
            meshes,
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

        let (mut objects, meshes) = load_model(world_model_file.as_path(), up_axis);

        // Build scene
        println!("Building BVH...");
        let bvh = BVH::build(&mut objects);
        println!("Done!");

        let lights = vec![Light::PointLight(PointLight::new(
            Point3::new(0.0, 0.9, 0.0),
            Vector3::new(1.0, 1.0, 1.0),
        ))];

        println!("Scene loaded.");

        Scene {
            bg_color: Vector3::new(0.5, 0.5, 0.5),
            objects,
            meshes,
            lights,
            bvh,
        }
    }

    pub fn push_object(&mut self, o: Object) {
        self.objects.push(o);
    }
}

fn load_model(model_file: &Path, _up_axis: &str) -> (Vec<Object>, Vec<Arc<Mesh>>) {
    dbg!(model_file);
    let (models, materials) = tobj::load_obj(model_file, true).expect("Failed to load file");

    dbg!(&materials);
    let mut triangles = vec![];
    let mut meshes = vec![];

    for (i, m) in models.iter().enumerate() {
        let mesh = Arc::new(m.mesh.clone());
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
            let color = Vector3::new(
                material.diffuse[0] as f64,
                material.diffuse[1] as f64,
                material.diffuse[2] as f64,
            );

            // let specular = Vector3::new(
            //     material.specular[0] as f64,
            //     material.specular[1] as f64,
            //     material.specular[2] as f64,
            // );

            let specular = Vector3::new(
               1.0,1.0,1.0
            );

            let _reflection = material.specular[0] as f64;

            let triangle = Triangle::new(
                mesh.clone(),
                mesh.indices[3 * v] as usize,
                mesh.indices[3 * v + 1] as usize,
                mesh.indices[3 * v + 2] as usize,
                vec![Material::PlasticMaterial(PlasticMaterial::new(
                    //weight: 1.0,
                    color,
                    specular,
                    0.0,//(material.shininess / 1000.0) as f64,
                    //ior: material.optical_density as f64,
                    //refraction: 1.0 - material.dissolve as f64,
                ))],
            );

            triangles.push(Object::Triangle(triangle));

            bar.inc(1);
        }

        meshes.push(mesh.clone());

        bar.finish();
    }

    (triangles, meshes)
}
