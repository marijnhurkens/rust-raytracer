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
        ray: renderer::Ray,
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
        let mut fresnel_ratio = 0.0;

        let i_dot_n = ray.direction.dot(normal);
        let mut eta_i = 1.0;
        let mut eta_t = self.ior;
        if i_dot_n > 0.0 {
            eta_i = eta_t;
            eta_t = 1.0;
        }

        let sin_t = eta_i / eta_t * (1.0 - i_dot_n * i_dot_n).max(0.0).sqrt();
        if sin_t > 1.0 {
            //Total internal reflection
            fresnel_ratio = 1.0;
        } else {
            let cos_t = (1.0 - sin_t * sin_t).max(0.0).sqrt();
            let cos_i = cos_t.abs();
            let r_s = ((eta_t * cos_i) - (eta_i * cos_t)) / ((eta_t * cos_i) + (eta_i * cos_t));
            let r_p = ((eta_i * cos_i) - (eta_t * cos_t)) / ((eta_i * cos_i) + (eta_t * cos_t));
            fresnel_ratio = (r_s * r_s + r_p * r_p) / 2.0;
        }



        let reflect_ray = renderer::Ray {
            point: point_of_intersection,
            direction: vector_reflect(ray.direction, normal)
                + (1.0 - self.glossiness) * get_random_in_unit_sphere(),
        };

        //println!("Ray {:?}, normal {:?}, fresnel {}", ray, normal, fresnel_ratio);

        let mut reflect_ray_color = Vector3::new(0.0, 0.0, 0.0);

        if let Some(reflect_surface_color) = renderer::trace(reflect_ray, scene, depth + 1) {
            reflect_ray_color = reflect_surface_color;
        }

        Some(
            (reflect_ray_color * fresnel_ratio)
                + (Vector3::new(0.8, 0.0, 0.0) * (1.0 - fresnel_ratio)) * self.weight,
        )
    }
}
