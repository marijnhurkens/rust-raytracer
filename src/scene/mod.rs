use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::Arc;

use bvh::bvh::BVH;
use indicatif::ProgressBar;
use nalgebra::{Point3, Vector3};
use tobj::Mesh;
use yaml_rust::YamlLoader;

use lights::area::AreaLight;
use lights::Light;
use materials::Material;
use materials::matte::MatteMaterial;
use Object;
use objects::ArcObject;
use objects::triangle::Triangle;

pub struct Scene {
    pub bg_color: Vector3<f64>,
    pub objects: Vec<ArcObject>,
    pub meshes: Vec<Arc<Mesh>>,
    pub lights: Vec<Arc<Light>>,
    pub bvh: BVH,
}

impl Scene {
    pub fn new(
        bg_color: Vector3<f64>,
        lights: Vec<Arc<Light>>,
        objects: Vec<ArcObject>,
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

        let triangle_light_mesh = Arc::new(Mesh {
            positions: vec![-0.5, 0.9, 0.0, 0.5, 0.9, 0.0, 0.0, 0.9, 0.5],
            normals: vec![0.0, -1.0, 0.0, 0.0, -1.0, 0.0, 0.0, -1.0, 0.0],
            texcoords: vec![],
            indices: vec![],
            num_face_indices: vec![],
            material_id: None,
        });

        let triangle_light =  Arc::new(Light::Area(AreaLight::new(
            ArcObject(Arc::new(Object::Triangle(Triangle::new(
                triangle_light_mesh.clone(),
                0,
                1,
                2,
                vec![],
                None
            )))),
            Vector3::repeat(4.4),
        )));

        let triangle_light_object = ArcObject(Arc::new(Object::Triangle(Triangle::new(
            triangle_light_mesh,
            0,
            1,
            2,
            vec![Material::MatteMaterial(MatteMaterial::new(
                Vector3::repeat(1.0),
                1000.0,
            ))],
            Some(triangle_light.clone())
        ))));

        let lights: Vec<Arc<Light>> = vec![triangle_light];

        objects.push(triangle_light_object);


        // Build scene
        println!("Building BVH...");
        let bvh = BVH::build(&mut objects);
        println!("Done!");

        println!("Scene loaded.");

        Scene {
            bg_color: Vector3::new(0.5, 0.5, 0.5),
            objects,
            meshes,
            lights,
            bvh,
        }
    }

    pub fn push_object(&mut self, o: ArcObject) {
        self.objects.push(o);
    }
}

fn load_model(model_file: &Path, _up_axis: &str) -> (Vec<ArcObject>, Vec<Arc<Mesh>>) {
    dbg!(model_file);
    let (models, materials) = tobj::load_obj(model_file, true).expect("Failed to load file");

    dbg!(&materials);
    let mut triangles: Vec<ArcObject> = vec![];
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

            let specular = Vector3::new(1.0, 1.0, 1.0);

            let _reflection = material.specular[0] as f64;

            let triangle = Triangle::new(
                mesh.clone(),
                mesh.indices[3 * v] as usize,
                mesh.indices[3 * v + 1] as usize,
                mesh.indices[3 * v + 2] as usize,
                vec![Material::MatteMaterial(MatteMaterial::new(
                    //weight: 1.0,
                    color,
                    //specular,
                    1000.0, //(material.shininess / 1000.0) as f64,
                           //ior: material.optical_density as f64,
                           //refraction: 1.0 - material.dissolve as f64,
                ))],
                None,
            );

            triangles.push(ArcObject(Arc::new(Object::Triangle(triangle))));

            bar.inc(1);
        }

        meshes.push(mesh.clone());

        bar.finish();
    }

    (triangles, meshes)
}
