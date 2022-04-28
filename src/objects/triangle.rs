use std::sync::Arc;

use bvh::aabb::{Bounded, AABB};
use bvh::bounding_hierarchy::BHShape;
use nalgebra::{Point2, Point3, Vector2, Vector3};
use tobj::Mesh;

use helpers::{coordinate_system, gamma, max_dimension_vec_3, permute};
use materials::Material;
use renderer;
use surface_interaction::SurfaceInteraction;

#[derive(Debug)]
pub struct Triangle {
    pub mesh: Arc<Mesh>,
    pub v0_index: usize,
    pub v1_index: usize,
    pub v2_index: usize,
    pub materials: Vec<Material>,
    pub node_index: usize,
}

impl Triangle {
    pub fn new(
        mesh: Arc<Mesh>,
        v0_index: usize,
        v1_index: usize,
        v2_index: usize,
        materials: Vec<Material>,
    ) -> Triangle {
        Triangle {
            mesh,
            v0_index,
            v1_index,
            v2_index,
            materials,
            node_index: 0,
        }
    }

    fn get_vertices(&self) -> (Point3<f64>, Point3<f64>, Point3<f64>) {
        (
            Point3::new(
                self.mesh.positions[3 * self.v0_index] as f64,
                self.mesh.positions[3 * self.v0_index + 1] as f64,
                self.mesh.positions[3 * self.v0_index + 2] as f64,
            ),
            Point3::new(
                self.mesh.positions[3 * self.v1_index] as f64,
                self.mesh.positions[3 * self.v1_index + 1] as f64,
                self.mesh.positions[3 * self.v1_index + 2] as f64,
            ),
            Point3::new(
                self.mesh.positions[3 * self.v2_index] as f64,
                self.mesh.positions[3 * self.v2_index + 1] as f64,
                self.mesh.positions[3 * self.v2_index + 2] as f64,
            ),
        )
    }

    fn get_normals(&self) -> (Vector3<f64>, Vector3<f64>, Vector3<f64>) {
        (
            Vector3::new(
                self.mesh.normals[3 * self.v0_index] as f64,
                self.mesh.normals[3 * self.v0_index + 1] as f64,
                self.mesh.normals[3 * self.v0_index + 2] as f64,
            ),
            Vector3::new(
                self.mesh.normals[3 * self.v1_index] as f64,
                self.mesh.normals[3 * self.v1_index + 1] as f64,
                self.mesh.normals[3 * self.v1_index + 2] as f64,
            ),
            Vector3::new(
                self.mesh.normals[3 * self.v2_index] as f64,
                self.mesh.normals[3 * self.v2_index + 1] as f64,
                self.mesh.normals[3 * self.v2_index + 2] as f64,
            ),
        )
    }
}

impl Triangle {
    pub fn get_materials(&self) -> &Vec<Material> {
        &self.materials
    }
    pub fn test_intersect(&self, ray: renderer::Ray) -> Option<(f64, SurfaceInteraction)> {
        let (p0, p1, p2) = self.get_vertices();

        let mut p0t = p0 - ray.point;
        let mut p1t = p1 - ray.point;
        let mut p2t = p2 - ray.point;

        let kz = max_dimension_vec_3(ray.direction.abs());
        let kx = (kz + 1) % 3;
        let ky = (kx + 1) % 3;

        let d = permute(ray.direction, kx, ky, kz);
        p0t = permute(p0t, kx, ky, kz);
        p1t = permute(p1t, kx, ky, kz);
        p2t = permute(p2t, kx, ky, kz);

        let s_x = -d.x / d.z;
        let s_y = -d.y / d.z;
        let s_z = 1.0 / d.z;
        p0t.x += s_x * p0t.z;
        p0t.y += s_y * p0t.z;
        p1t.x += s_x * p1t.z;
        p1t.y += s_y * p1t.z;
        p2t.x += s_x * p2t.z;
        p2t.y += s_y * p2t.z;

        let e0 = p1t.x * p2t.y - p1t.y * p2t.x;
        let e1 = p2t.x * p0t.y - p2t.y * p0t.x;
        let e2 = p0t.x * p1t.y - p0t.y * p1t.x;

        if (e0 < 0.0 || e1 < 0.0 || e2 < 0.0) && (e0 > 0.0 || e1 > 0.0 || e2 > 0.0) {
            return None;
        }

        let det = e0 + e1 + e2;
        if det == 0.0 {
            return None;
        }

        p0t.z *= s_z;
        p1t.z *= s_z;
        p2t.z *= s_z;
        let t_scaled = e0 * p0t.z + e1 * p1t.z + e2 * p2t.z;
        // todo: implement ray.t_max instead of 1000.0
        if det < 0.0 && (t_scaled >= 0.0 || t_scaled < 1000.0 * det) {
            return None;
        }

        if det > 0.0 && (t_scaled <= 0.0 || t_scaled > 1000.0 * det) {
            return None;
        }

        let inv_det = 1.0 / det;
        let b0 = e0 * inv_det;
        let b1 = e1 * inv_det;
        let b2 = e2 * inv_det;
        let t = t_scaled * inv_det;

        if t < f64::EPSILON {
            return None;
        }

        let uv = vec![
            Point2::new(0.0, 0.0),
            Point2::new(1.0, 0.0),
            Point2::new(1.0, 1.0),
        ];

        let duv02: Vector2<f64> = uv[0] - uv[2];
        let duv12: Vector2<f64> = uv[1] - uv[2];
        let dp02: Vector3<f64> = p0 - p2;
        let dp12: Vector3<f64> = p1 - p2;

        let determinant = duv02.x * duv12.y - duv02.y * duv12.x;

        let dpdu;
        let dpdv;

        if determinant == 0.0 {
            (_, dpdu, dpdv) = coordinate_system((p2 - p0).cross(&(p1 - p0)).normalize());
        } else {
            let inv_det = 1.0 / determinant;
            dpdu = (duv12[1] * dp02 - duv02[1] * dp12) * inv_det;
            dpdv = (-duv12[0] * dp02 + duv02[0] * dp12) * inv_det;
        }

        let (p0_normal, p1_normal, p2_normal) = self.get_normals();
        let normal = (b0 * p0_normal + b1 * p1_normal + b2 * p2_normal).normalize();
        let uv_hit = b0 * uv[0].coords + b1 * uv[1].coords + b2 * uv[2].coords;


        let x_abs_sum = (b0 * p0.x).abs() + (b1 * p1.x).abs() + (b2 * p2.x).abs();
        let y_abs_sum = (b0 * p0.y).abs() + (b1 * p1.y).abs() + (b2 * p2.y).abs();
        let z_abs_sum = (b0 * p0.z).abs() + (b1 * p1.z).abs() + (b2 * p2.z).abs();

        let p_error: Vector3<f64> = gamma(7.0) * Vector3::new(x_abs_sum, y_abs_sum, z_abs_sum);
        let mut p_hit: Point3<f64> = (b0 * p0.coords + b1 * p1.coords + b2 * p2.coords).into();

        // todo: fix how to handle error, otherwise light leaks
        p_hit += normal * 1.0e-5;

        Some((
            t,
            SurfaceInteraction::new(p_hit, normal, -ray.direction, uv_hit, dpdu, dpdv, p_error),
        ))
    }
}

impl Bounded for Triangle {
    fn aabb(&self) -> AABB {
        let (p0, p1, p2) = self.get_vertices();

        let min_x = p0.x.min(p1.x.min(p2.x));
        let min_y = p0.y.min(p1.y.min(p2.y));
        let min_z = p0.z.min(p1.z.min(p2.z));
        let max_x = p0.x.max(p1.x.max(p2.x));
        let max_y = p0.y.max(p1.y.max(p2.y));
        let max_z = p0.z.max(p1.z.max(p2.z));

        AABB::with_bounds(
            bvh::nalgebra::Point3::new(min_x as f32, min_y as f32, min_z as f32),
            bvh::nalgebra::Point3::new(
                (max_x + 0.001) as f32,
                (max_y + 0.001) as f32,
                (max_z + 0.001) as f32,
            ),
        )
    }
}

impl BHShape for Triangle {
    fn set_bh_node_index(&mut self, index: usize) {
        self.node_index = index;
    }

    fn bh_node_index(&self) -> usize {
        self.node_index
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use nalgebra::{Point3, Vector3};
    use tobj::Mesh;

    use materials;
    use materials::Material;
    use objects::triangle::Triangle;
    use renderer::Ray;

    #[test]
    fn it_tests_intersects() {
        let mesh = Mesh {
            positions: vec![-1.0, -1.0, 0.0, 1.0, -1.0, 0.0, 1.0, 1.0, 0.0],
            normals: vec![0.6, 0.0, -1.0, 0.0, 0.5, -1.0, 0.4, 0.0, -1.0],
            texcoords: vec![],
            indices: vec![],
            num_face_indices: vec![],
            material_id: None,
        };

        let triangle = Triangle::new(
            Arc::new(mesh),
            0,
            1,
            2,
            vec![Material::MatteMaterial(materials::MatteMaterial::new(
                Vector3::new(1.0, 1.0, 1.0),
                100.0,
            ))],
        );

        let ray = Ray {
            point: Point3::new(0.0, 0.0, -2.0),
            direction: Vector3::new(0.0, 0.0, 1.0),
        };

        let option_intersection = triangle.test_intersect(ray);

        assert_eq!(true, option_intersection.is_some());

        let (distance, i) = option_intersection.unwrap();

        assert_eq!(
            true,
            Vector3::<f64>::new(0.5, 0.0, -1.0).normalize().relative_eq(
                &i.surface_normal,
                f64::EPSILON,
                1.0e-6
            )
        );

        assert_eq!(2.0, distance);

        dbg!(i);
    }
}
