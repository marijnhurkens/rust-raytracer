use bvh::bvh::Bvh;
use image::ImageReader;
use indicatif::ProgressBar;
use nalgebra::{Matrix3, Matrix4, Point3, Rotation3, Translation3, Vector3};
use std::borrow::BorrowMut;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::Arc;
use image::codecs::avif::ColorSpace;
use tobj::{LoadOptions, Mesh};
use yaml_rust::YamlLoader;

use crate::helpers::yaml_array_into_vector3;
use crate::lights::area::AreaLight;
use crate::lights::distant::DistantLight;
use crate::lights::infinite_area::InfiniteAreaLight;
use crate::lights::point::PointLight;
use crate::lights::Light;
use crate::materials::glass::GlassMaterial;
use crate::materials::matte::MatteMaterial;
use crate::materials::mirror::MirrorMaterial;
use crate::materials::plastic::PlasticMaterial;
use crate::materials::Material;
use crate::objects::plane::Plane;
use crate::objects::rectangle::Rectangle;
use crate::objects::sphere::Sphere;
use crate::objects::triangle::Triangle;
use crate::objects::ArcObject;
use crate::{yaml_array_into_point3, Object};

pub struct Scene {
    pub bg_color: Vector3<f64>,
    pub objects: Vec<ArcObject>,
    pub lights: Vec<Arc<Light>>,
    pub bvh: Bvh<f32, 3>,
}

impl Scene {
    pub fn new(
        bg_color: Vector3<f64>,
        lights: Vec<Arc<Light>>,
        objects: Vec<ArcObject>,
        meshes: Vec<Arc<Mesh>>,
        bvh: Bvh<f32, 3>,
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
                    vec![Arc::new(Material::Matte(MatteMaterial::new(
                        Vector3::repeat(0.9),
                        20.0,
                    )))],
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

        if let Some(environment_map) = scene_yaml["environment_map"].as_str() {
            let image_map = ImageReader::open(path.join(environment_map))
                .expect("Environment map not found.")
                .decode()
                .expect("Cannot decode environment map.");
            let infinite_light = Light::InfiniteArea(InfiniteAreaLight::new(
                &Vector3::repeat(1.0),
                image_map.to_rgb8(),
                Matrix4::new_translation(&Vector3::new(0.0, 1.0, 0.0)),
            ));

            lights.push(Arc::new(infinite_light));
        }

        // let cube = ArcObject(Arc::new(Object::Sphere(Sphere::new(
        //     Point3::new(0.3, 0.0, 0.0),
        //     0.5,
        //     vec![Arc::new(
        //         Material::Plastic(PlasticMaterial::new(Vector3::repeat(0.2), Vector3::repeat(1.0), 0.1, 1.8))
        //     )],
        // ))));
        //
        // objects.push(cube);

        let floor = ArcObject(Arc::new(Object::Plane(Plane::new(
            Point3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
            vec![Arc::new(Material::Plastic(PlasticMaterial::new(
                Vector3::repeat(0.9),
                Vector3::repeat(1.0),
                0.0,
                1.5,
            )))],
        ))));

      //  objects.push(floor);

        // let mesh = Arc::new(Mesh{
        //     positions: vec![
        //         0.0,0.0,0.0,
        //         1.0,0.0,1.0,
        //         0.0,1.0,1.0,
        //     ],
        //     vertex_color: vec![],
        //     normals: vec![
        //         0.0,0.0,1.0,
        //         0.0,0.0,1.0,
        //         0.0,0.0,1.0,
        //     ],
        //     texcoords: vec![],
        //     indices: vec![],
        //     face_arities: vec![],
        //     texcoord_indices: vec![],
        //     normal_indices: vec![],
        //     material_id: None,
        // });
        //
        // let triangle = ArcObject(Arc::new(Object::Triangle(Triangle::new(
        //     mesh,
        //     0,
        //     1,
        //     2,
        //     vec![
        //         Material::Glass(GlassMaterial::new(Vector3::repeat(1.0))),
        //         // Material::Matte(MatteMaterial::new(
        //         //     Vector3::new(0.5, 0.5, 0.5),
        //         //     0.5,
        //         // ))
        //     ],
        //     None,
        // ))));

        // objects.push(triangle);

        // Build scene
        println!("Building BVH...");
        let bvh = Bvh::build(&mut objects);
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

        let material = mesh.material_id.map(|material_id| &materials[material_id]);

        let color = if let Some(material) = material {
            Vector3::new(
                material.diffuse[0] as f64,
                material.diffuse[1] as f64,
                material.diffuse[2] as f64,
            )
        } else {
            Vector3::repeat(0.8)
        };

        let ior: f64 = if let Some(material) = material {
            material.optical_density.into()
        } else {
            1.5
        };

        let roughness = if let Some(material) = material {
            material
                .unknown_param
                .get("Pr")
                .and_then(|pr| pr.parse::<f64>().ok())
                .unwrap_or(32.0)
        } else {
            0.5
        };

        let (is_translucent, translucence) = if let Some(material) = material {
            let tf_param = material.unknown_param.get("Tf");
            if let Some(tf) = tf_param {
                // split string on space, parse as f64 and create vector3
                let tf_values: Vec<f64> = tf
                    .as_str()
                    .split_whitespace()
                    .map(|s| s.parse::<f64>().unwrap())
                    .collect();
                (true, Vector3::new(tf_values[0], tf_values[1], tf_values[2]))
            } else {
                (false, Vector3::repeat(0.0))
            }
        } else {
            (false, Vector3::repeat(0.0))
        };

        let specular = if let Some(material) = material {
            Vector3::new(
                material.specular[0] as f64,
                material.specular[1] as f64,
                material.specular[2] as f64,
            )
        } else {
            Vector3::repeat(0.0)
        };

        let internal_material = if is_translucent {
            Arc::new(Material::Glass(GlassMaterial::new(ior, color, translucence)))
        } else {
            Arc::new(Material::Plastic(PlasticMaterial::new(color, specular, roughness, ior)))
            //Arc::new(Material::Matte(MatteMaterial::new(color, roughness)))
            //Arc::new(Material::Glass(GlassMaterial::new(ior, color, color)))
        };

        dbg!(&internal_material);

        let bar = ProgressBar::new((mesh.indices.len() / 3) as u64);
        for v in 0..mesh.indices.len() / 3 {
            let triangle = Triangle::new(
                mesh.clone(),
                mesh.indices[3 * v] as usize,
                mesh.indices[3 * v + 1] as usize,
                mesh.indices[3 * v + 2] as usize,
                vec![internal_material.clone()],
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
