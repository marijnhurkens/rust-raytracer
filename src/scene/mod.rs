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
use lights::distant::DistantLight;
use lights::Light;
use materials::matte::MatteMaterial;
use materials::Material;
use objects::plane::Plane;
use objects::triangle::Triangle;
use objects::ArcObject;
use Object;

pub struct Scene {
    pub bg_color: Vector3<f64>,
    pub objects: Vec<ArcObject>,
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
            lights,
            bvh,
        }
    }

    pub fn load_from_folder(path: &Path) -> Scene {
        println!("Load scene from {:?}", path.display());
        let mut file = File::open(path.join("scene.yaml")).expect("Unable to open scene.yaml file");
        let mut contents = String::new();

        file.read_to_string(&mut contents)
            .expect("Unable to read file");
        let scene_yaml = &YamlLoader::load_from_str(&contents).unwrap()[0];

        let world_model_file = path.join(Path::new(scene_yaml["world"]["file"].as_str().unwrap()));
        let up_axis = scene_yaml["world"]["up_axis"].as_str().unwrap();

        let (mut objects, meshes) = load_model(world_model_file.as_path(), up_axis);

        let distant_light = Arc::new(Light::Distant(DistantLight::new(
            Point3::origin(),
            1e20,
            Vector3::new(0.4, 1.0, 0.4),
            Vector3::repeat(6.0),
        )));

        let lights: Vec<Arc<Light>> = vec![distant_light];

        let floor = ArcObject(Arc::new(Object::Plane(Plane::new(
            Point3::origin(),
            Vector3::new(0.0, 1.0, 0.0),
            vec![Material::MatteMaterial(MatteMaterial::new(
                Vector3::repeat(1.0),
                20.0,
            ))],
        ))));

        objects.push(floor);

        // Build scene
        println!("Building BVH...");
        let bvh = BVH::build(&mut objects);
        println!("Done!");

        println!("Scene loaded.");

        Scene {
            bg_color: Vector3::new(0.5, 0.5, 0.5),
            objects,
            lights,
            bvh,
        }
    }

    pub fn push_object(&mut self, o: ArcObject) {
        self.objects.push(o);
    }
}

//fn load_lights()

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

        let material = mesh.material_id.map(|material_id| &materials[material_id]);

        for v in 0..mesh.indices.len() / 3 {
            let color = if let Some(material) = material {
                Vector3::new(
                    material.diffuse[0] as f64,
                    material.diffuse[1] as f64,
                    material.diffuse[2] as f64,
                )
            } else {
                Vector3::repeat(0.8)
            };

            // let specular = Vector3::new(
            //     material.specular[0] as f64,
            //     material.specular[1] as f64,
            //     material.specular[2] as f64,
            // );

            //let specular = Vector3::new(1.0, 1.0, 1.0);

            // let _reflection = material.specular[0] as f64;

            let triangle = Triangle::new(
                mesh.clone(),
                mesh.indices[3 * v] as usize,
                mesh.indices[3 * v + 1] as usize,
                mesh.indices[3 * v + 2] as usize,
                vec![Material::MatteMaterial(MatteMaterial::new(
                    //weight: 1.0,
                    color,
                    //specular,
                    20.0, //(material.shininess / 1000.0) as f64,
                         //ior: material.optical_density as f64,
                         //refraction: 1.0 - material.dissolve as f64,
                ))],
                None,
            );

            triangles.push(ArcObject(Arc::new(Object::Triangle(triangle))));

            if v % 1000 == 0 {
                bar.inc(1000);
            }
        }

        meshes.push(mesh.clone());

        bar.finish();
    }

    (triangles, meshes)
}
