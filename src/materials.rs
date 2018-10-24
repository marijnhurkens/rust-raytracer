use cgmath::*;
use helpers::*;
use renderer;
use scene;
use std::clone::Clone;
use std::fmt::Debug;

pub trait Material: Debug + Send + Sync {
    fn get_surface_color(
        &self,
        ray: renderer::Ray,
        scene: &scene::Scene,
        point_of_intersection: Vector3<f64>,
        normal: Vector3<f64>,
        depth_immutable: u32,
    ) -> Option<Vector3<f64>>;

    fn get_weight(&self) -> f64;
}

#[derive(Debug, Clone)]
pub struct Lambert {
    pub color: Vector3<f64>,
    pub weight: f64,
}

impl Material for Lambert {
    fn get_weight(&self) -> f64 {
        self.weight
    }

    fn get_surface_color(
        &self,
        _ray: renderer::Ray,
        scene: &scene::Scene,
        point_of_intersection: Vector3<f64>,
        normal: Vector3<f64>,
        depth: u32,
    ) -> Option<Vector3<f64>> {
        let mut lights_contribution = 0.0;
        let mut lambert_color = Vector3::new(0.0, 0.0, 0.0);

        // Check all the lights for visibility if visible use
        // the cross product of the normal and the vector to the light
        // to calculate the lambert contribution.
        for light in scene.lights.clone() {
            if !renderer::check_light_visible(point_of_intersection, &scene, light) {
                continue;
            }

            let contribution = (light.position - point_of_intersection)
                .normalize()
                .dot(normal)
                * light.intensity;
            if contribution > 0.0 {
                lights_contribution += contribution;
            }
        }

        // cap the lambert amount to 1
        if lights_contribution > 1.0 {
            lights_contribution = 1.0;
        }

        // throw random ray
        let target = point_of_intersection + normal + get_random_in_unit_sphere();

        let reflect_ray = renderer::Ray {
            point: point_of_intersection,
            direction: target - point_of_intersection,
        };

        if let Some(lambert_surface_color) = renderer::trace(reflect_ray, scene, depth + 1) {
            lambert_color += lambert_surface_color * 0.5;
        }

        Some(((self.color * lights_contribution) + lambert_color) * self.weight)
    }
}

#[derive(Debug, Clone)]
pub struct Reflection {
    pub weight: f64,
    pub glossiness: f64,
}

impl Material for Reflection {
    fn get_weight(&self) -> f64 {
        self.weight
    }

    // Normal reflections reflect the same amount
    // independant of the angle of intersection of the ray.
    //
    // A random angle is added when glosiness < 1
    fn get_surface_color(
        &self,
        ray: renderer::Ray,
        scene: &scene::Scene,
        point_of_intersection: Vector3<f64>,
        normal: Vector3<f64>,
        depth: u32,
    ) -> Option<Vector3<f64>> {
        let reflect_ray = renderer::Ray {
            point: point_of_intersection,
            direction: vector_reflect(ray.direction, normal)
                + (1.0 - self.glossiness) * get_random_in_unit_sphere(),
        };

        if let Some(reflect_surface_color) = renderer::trace(reflect_ray, scene, depth + 1) {
            return Some(reflect_surface_color * self.weight);
        }

        None
    }
}

#[derive(Debug, Clone)]
pub struct Refraction {
    pub weight: f64,
    pub ior: f64,
}

impl Material for Refraction {
    fn get_weight(&self) -> f64 {
        self.weight
    }

    fn get_surface_color(
        &self,
        ray: renderer::Ray,
        scene: &scene::Scene,
        point_of_intersection: Vector3<f64>,
        normal: Vector3<f64>,
        depth: u32,
    ) -> Option<Vector3<f64>> {
        let mut refract_ray_color = Vector3::new(1.0, 0.0, 0.0);

        if let Some(refract_vector) = get_refract_ray(normal, ray.direction, self.ior) {
            let refract_ray = renderer::Ray {
                point: point_of_intersection,
                direction: refract_vector,
            };

            if let Some(refract_surface_color) = renderer::trace(refract_ray, scene, depth + 1) {
                refract_ray_color = refract_surface_color;
            }
        }

        Some(refract_ray_color * self.weight)
    }
}

#[derive(Debug, Clone)]
pub struct FresnelReflection {
    pub weight: f64,
    pub glossiness: f64,
    pub ior: f64,
}

impl Material for FresnelReflection {
    fn get_weight(&self) -> f64 {
        self.weight
    }

    fn get_surface_color(
        &self,
        ray: renderer::Ray,
        scene: &scene::Scene,
        point_of_intersection: Vector3<f64>,
        normal: Vector3<f64>,
        depth: u32,
    ) -> Option<Vector3<f64>> {
        let fresnel_ratio = get_fresnel_ratio(normal, ray.direction, self.ior);
        let mut reflect_ray_color = Vector3::new(0.0, 0.0, 1.0);
        let mut refract_ray_color = Vector3::new(1.0, 0.0, 0.0);
        let outside = ray.direction.dot(normal) < 0.0;

        let bias = 0.01 * normal;

        if fresnel_ratio < 1.0 {
            if let Some(refract_vector) = get_refract_ray(normal, ray.direction, self.ior) {
                let refract_ray_start = if outside {
                    point_of_intersection - bias
                } else {
                    point_of_intersection + bias
                };

                let refract_ray = renderer::Ray {
                    point: refract_ray_start,
                    direction: refract_vector,
                };

                if let Some(refract_surface_color) = renderer::trace(refract_ray, scene, depth + 1)
                {
                    refract_ray_color = refract_surface_color;
                }
            } else {
                println!("Shouldn't happen?");
            }
        }

        let reflect_ray_start = if outside {
            point_of_intersection + bias
        } else {
            point_of_intersection - bias
        };

        let reflect_ray = renderer::Ray {
            point: reflect_ray_start,
            direction: vector_reflect(ray.direction, normal)
                + (1.0 - self.glossiness) * get_random_in_unit_sphere(),
        };

        if let Some(reflect_surface_color) = renderer::trace(reflect_ray, scene, depth + 1) {
            reflect_ray_color = reflect_surface_color;
        }

        // Some(
        //     ((reflect_ray_color * fresnel_ratio) + (refract_ray_color * (1.0 - fresnel_ratio)))
        //         * self.weight,
        // )

        if fresnel_ratio < 0.0 {
            println!("Negative");
        }

        if fresnel_ratio > 1.0 {
            println!("Too large");
        }

        println!("Fresnel ratio: {}", fresnel_ratio);
        Some(Vector3::new(1.0,0.0,0.0) * (fresnel_ratio/100.0))
    }
}
