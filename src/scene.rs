use std::borrow::BorrowMut;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::Arc;

use bvh::bvh::BVH;
use indicatif::ProgressBar;
use nalgebra::{Point3, Vector3};
use tobj::{LoadOptions, Mesh};
use yaml_rust::YamlLoader;

use crate::helpers::yaml_array_into_vector3;
use crate::lights::area::AreaLight;
use crate::lights::distant::DistantLight;
use crate::lights::point::PointLight;
use crate::lights::Light;
use crate::materials::glass::GlassMaterial;
use crate::materials::matte::MatteMaterial;
use crate::materials::mirror::MirrorMaterial;
use crate::materials::plastic::PlasticMaterial;
use crate::materials::Material;
use crate::objects::plane::Plane;
use crate::objects::rectangle::Rectangle;
use crate::objects::triangle::Triangle;
use crate::objects::ArcObject;
use crate::{yaml_array_into_point3, Object};

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

        let (mut objects, meshes) = if let Some(filename) = scene_yaml["world"]["file"].as_str() {
            let world_model_file = path.join(Path::new(filename));
            let up_axis = scene_yaml["world"]["up_axis"].as_str().unwrap();
            load_model(world_model_file.as_path(), up_axis)
        } else {
            (vec![], vec![])
        };

        let mut lights: Vec<Arc<Light>> = vec![];

        for light_config in scene_yaml["lights"].clone() {
            let l_type = light_config["type"].as_str().unwrap();

            if l_type == "area" {
                let l_pos = yaml_array_into_point3(&light_config["position"]);
                let l_side_a = yaml_array_into_vector3(&light_config["side_a"]);
                let l_side_b = yaml_array_into_vector3(&light_config["side_b"]);
                let l_intensity = yaml_array_into_vector3(&light_config["intensity"]);

                let light_rectangle = ArcObject(Arc::new(Object::Rectangle(Rectangle::new(
                    l_pos,
                    l_side_a,
                    l_side_b,
                    vec![],
                    None,
                ))));

                let light = Arc::new(Light::Area(AreaLight::new(light_rectangle, l_intensity)));

                let light_rectangle = ArcObject(Arc::new(Object::Rectangle(Rectangle::new(
                    l_pos,
                    l_side_a,
                    l_side_b,
                    vec![Material::Matte(MatteMaterial::new(
                        Vector3::repeat(0.9),
                        20.0,
                    ))],
                    Some(light.clone()),
                ))));

                lights.push(light);
                objects.push(light_rectangle);
            }

            if l_type == "distant" {
                let light = Arc::new(Light::Distant(DistantLight::new(
                    Point3::origin(),
                    1e20,
                    yaml_array_into_vector3(&light_config["direction"]),
                    yaml_array_into_vector3(&light_config["intensity"]),
                )));

                lights.push(light);
            }
        }

        let floor = ArcObject(Arc::new(Object::Plane(Plane::new(
            Point3::origin(),
            Vector3::new(0.0, 1.0, 0.0),
            vec![Material::Matte(MatteMaterial::new(
                Vector3::repeat(0.7),
                3.0,
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

fn load_model(model_file: &Path, _up_axis: &str) -> (Vec<ArcObject>, Vec<Arc<Mesh>>) {
    //dbg!(model_file);
    let (models, materials) = tobj::load_obj(
        model_file,
        &LoadOptions {
            single_index: true,
            triangulate: true,
            ignore_points: true,
            ignore_lines: true,
        },
    )
    .expect("Failed to load file");

    let materials = materials.unwrap();

    //dbg!(&materials);
    let mut triangles: Vec<ArcObject> = vec![];
    let mut meshes = vec![];

    for (i, m) in models.iter().enumerate() {
        let mesh = Arc::new(m.mesh.clone());
        println!("model[{}].name = \'{}\'", i, m.name);
        //println!("model[{}].mesh.material_id = {:?}", i, mesh.material_id);

        // Normals and texture coordinates are also loaded, but not printed in this example
        // println!("model[{}].vertices: {}", i, mesh.positions.len() / 3);
        // println!("model[{}].indices: {}", i, mesh.indices.len());
        // println!(
        //     "model[{}].expected_triangles: {}",
        //     i,
        //     mesh.indices.len() / 3
        // );
        // println!("model[{}].normals: {}", i, mesh.normals.len() / 3);

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
                // vec![Material::MatteMaterial(MatteMaterial::new(
                //     //weight: 1.0,
                //     color,
                //     //specular,
                //     20.0, //(material.shininess / 1000.0) as f64,
                //          //ior: material.optical_density as f64,
                //          //refraction: 1.0 - material.dissolve as f64,
                // ))],
                vec![Material::Glass(GlassMaterial::new(
                    Vector3::repeat(1.0),
                    // Vector3::repeat(1.0),
                    // 0.03,
                ))],
                // vec![Material::PlasticMaterial(PlasticMaterial::new(
                //     Vector3::new(0.7, 0.7, 0.7),
                //     Vector3::repeat(0.99),
                //     0.0001,
                // ))],
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
