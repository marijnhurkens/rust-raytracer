use std::sync::Arc;

use bvh::aabb::{Bounded, AABB};
use bvh::bounding_hierarchy::BHShape;
use nalgebra::{Point2, Point3, Vector2, Vector3};
use tobj::Mesh;

use crate::helpers::{
    coordinate_system, gamma, max_dimension_vec_3, permute, uniform_sample_triangle,
};
use crate::lights::area::AreaLight;
use crate::lights::Light;
use crate::materials::Material;
use crate::objects::ObjectTrait;
use crate::renderer;
use crate::renderer::{check_intersect_scene, debug_write_pixel, Ray};
use crate::surface_interaction::{Interaction, SurfaceInteraction};

#[derive(Debug, Clone)]
pub struct Triangle {
    pub mesh: Arc<Mesh>,
    pub p0: Point3<f64>,
    p1: Point3<f64>,
    p2: Point3<f64>,
    n0: Vector3<f64>,
    n1: Vector3<f64>,
    n2: Vector3<f64>,
    pub materials: Vec<Material>,
    pub light: Option<Arc<Light>>,
    pub node_index: usize,
}

impl Triangle {
    pub fn new(
        mesh: Arc<Mesh>,
        v0_index: usize,
        v1_index: usize,
        v2_index: usize,
        materials: Vec<Material>,
        light: Option<Arc<Light>>,
    ) -> Triangle {
        let (p0, p1, p2) = Triangle::get_vertices(&mesh, v0_index, v1_index, v2_index);
        let (n0, n1, n2) = Triangle::get_normals(&mesh, v0_index, v1_index, v2_index);

        Triangle {
            mesh,
            p0,
            p1,
            p2,
            n0,
            n1,
            n2,
            materials,
            light,
            node_index: 0,
        }
    }

    fn get_vertices(
        mesh: &Arc<Mesh>,
        v0_index: usize,
        v1_index: usize,
        v2_index: usize,
    ) -> (Point3<f64>, Point3<f64>, Point3<f64>) {
        (
            Point3::new(
                mesh.positions[3 * v0_index] as f64,
                mesh.positions[3 * v0_index + 1] as f64,
                mesh.positions[3 * v0_index + 2] as f64,
            ),
            Point3::new(
                mesh.positions[3 * v1_index] as f64,
                mesh.positions[3 * v1_index + 1] as f64,
                mesh.positions[3 * v1_index + 2] as f64,
            ),
            Point3::new(
                mesh.positions[3 * v2_index] as f64,
                mesh.positions[3 * v2_index + 1] as f64,
                mesh.positions[3 * v2_index + 2] as f64,
            ),
        )
    }

    fn get_normals(
        mesh: &Arc<Mesh>,
        v0_index: usize,
        v1_index: usize,
        v2_index: usize,
    ) -> (Vector3<f64>, Vector3<f64>, Vector3<f64>) {
        (
            Vector3::new(
                mesh.normals[3 * v0_index] as f64,
                mesh.normals[3 * v0_index + 1] as f64,
                mesh.normals[3 * v0_index + 2] as f64,
            ),
            Vector3::new(
                mesh.normals[3 * v1_index] as f64,
                mesh.normals[3 * v1_index + 1] as f64,
                mesh.normals[3 * v1_index + 2] as f64,
            ),
            Vector3::new(
                mesh.normals[3 * v2_index] as f64,
                mesh.normals[3 * v2_index + 1] as f64,
                mesh.normals[3 * v2_index + 2] as f64,
            ),
        )
    }
}

impl ObjectTrait for Triangle {
    fn get_materials(&self) -> &Vec<Material> {
        &self.materials
    }

    fn get_light(&self) -> Option<&Arc<Light>> {
        self.light.as_ref()
    }

    fn test_intersect(&self, ray: renderer::Ray) -> Option<(f64, SurfaceInteraction)> {
        let p0 = self.p0;
        let p1 = self.p1;
        let p2 = self.p2;

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

        let (dpdu, dpdv) = if determinant == 0.0 {
            let (_, u, v) = coordinate_system((p2 - p0).cross(&(p1 - p0)).normalize());
            (u,v)
        } else {
            let inv_det = 1.0 / determinant;
            let dpdu = (duv12[1] * dp02 - duv02[1] * dp12) * inv_det;
            let dpdv = (-duv12[0] * dp02 + duv02[0] * dp12) * inv_det;
            (dpdu, dpdv)
        };

        let p0_normal = self.n0;
        let p1_normal = self.n1;
        let p2_normal = self.n2;
        let shading_normal = (b0 * p0_normal + b1 * p1_normal + b2 * p2_normal).normalize();

        let (ss, ts) = {
            let mut ss = dpdu.normalize();
            let mut ts = shading_normal.cross(&ss);
            if ts.magnitude_squared() > 0.0 {
                ts = ts.normalize();
                ss = ts.cross(&shading_normal);
                (ss, ts)
            } else {
                let (_, ss, ts) = coordinate_system(shading_normal);
                (ss, ts)
            }
        };

        let uv_hit = b0 * uv[0].coords + b1 * uv[1].coords + b2 * uv[2].coords;

        let x_abs_sum = (b0 * p0.x).abs() + (b1 * p1.x).abs() + (b2 * p2.x).abs();
        let y_abs_sum = (b0 * p0.y).abs() + (b1 * p1.y).abs() + (b2 * p2.y).abs();
        let z_abs_sum = (b0 * p0.z).abs() + (b1 * p1.z).abs() + (b2 * p2.z).abs();

        let p_error: Vector3<f64> = gamma(7.0) * Vector3::new(x_abs_sum, y_abs_sum, z_abs_sum);
        let mut p_hit: Point3<f64> = (b0 * p0.coords + b1 * p1.coords + b2 * p2.coords).into();

        // p_hit = compute_shading_position(
        //     p_hit, p0, p1, p2, p0_normal, p1_normal, p2_normal, b0, b1, b2, normal,
        // );
        let p1p0 = p1 - p0;
        let geometry_normal = (p2 - p0).cross(&p1p0).normalize();

        p_hit += shading_normal * 1.0e-9;

        Some((
            t,
            SurfaceInteraction::new(
                p_hit,
                geometry_normal,
                -ray.direction,
                uv_hit,
                ss,
                ts,
                dpdu,
                dpdv,
                p_error,
            ),
        ))
    }

    fn sample_point(&self, sample: Vec<f64>) -> Interaction {
        let sample = uniform_sample_triangle(sample);

        let point = sample.x * self.p0
            + sample.y * self.p1.coords
            + (1.0 - sample.x - sample.y) * self.p2.coords;

        let shading_normal =
            (sample.x * self.n0 + sample.y * self.n1 + (1.0 - sample.x - sample.y) * self.n2)
                .normalize();

        Interaction {
            point,
            normal: shading_normal,
        }
    }

    fn pdf(&self, interaction: &Interaction, wi: Vector3<f64>) -> f64 {
        let ray = Ray {
            point: interaction.point + wi * 1e-9,
            direction: wi,
        };

        let intersect_object = self.test_intersect(ray);

        if intersect_object.is_none() {
            return 0.0;
        }

        let (_, surface_interaction) = intersect_object.unwrap();

        nalgebra::distance_squared(&interaction.point, &surface_interaction.point)
            / (surface_interaction.shading_normal.dot(&-wi).abs() * self.area())
    }

    fn area(&self) -> f64 {
        let p0p1 = self.p1 - self.p0;
        let p0p2 = self.p2 - self.p0;

        0.5 * p0p1.cross(&p0p2).magnitude()
    }
}

fn project_on_plane(p: Point3<f64>, origin: Point3<f64>, normal: Vector3<f64>) -> Point3<f64> {
    p - (p - origin).dot(&normal) * normal
}

#[allow(clippy::too_many_arguments)]
fn compute_shading_position(
    p_hit: Point3<f64>,
    v0: Point3<f64>,
    v1: Point3<f64>,
    v2: Point3<f64>,
    p0_normal: Vector3<f64>,
    p1_normal: Vector3<f64>,
    p2_normal: Vector3<f64>,
    b0: f64,
    b1: f64,
    b2: f64,
    shading_normal: Vector3<f64>,
) -> Point3<f64> {
    let p0 = project_on_plane(p_hit, v0, p0_normal);
    let p1 = project_on_plane(p_hit, v1, p1_normal);
    let p2 = project_on_plane(p_hit, v2, p2_normal);

    let shading_pos: Point3<f64> = (b0 * p0.coords + b1 * p1.coords + b2 * p2.coords).into();

    let convex = (shading_pos - p_hit).dot(&shading_normal) > 0.0;

    if convex {
        shading_pos
    } else {
        p_hit
    }
}

impl Bounded for Triangle {
    fn aabb(&self) -> AABB {
        let min_x = self.p0.x.min(self.p1.x.min(self.p2.x));
        let min_y = self.p0.y.min(self.p1.y.min(self.p2.y));
        let min_z = self.p0.z.min(self.p1.z.min(self.p2.z));
        let max_x = self.p0.x.max(self.p1.x.max(self.p2.x));
        let max_y = self.p0.y.max(self.p1.y.max(self.p2.y));
        let max_z = self.p0.z.max(self.p1.z.max(self.p2.z));

        AABB::with_bounds(
            bvh::Point3::new(min_x as f32, min_y as f32, min_z as f32),
            bvh::Point3::new(
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

    use crate::materials;
    use crate::materials::matte::MatteMaterial;
    use crate::materials::Material;
    use crate::objects::triangle::Triangle;
    use crate::objects::ObjectTrait;
    use crate::renderer::Ray;

    #[test]
    fn it_tests_intersects() {
        let mesh = Mesh {
            positions: vec![-1.0, -1.0, 0.0, 1.0, -1.0, 0.0, 1.0, 1.0, 0.0],
            vertex_color: vec![],
            normals: vec![0.6, 0.0, -1.0, 0.0, 0.5, -1.0, 0.4, 0.0, -1.0],
            texcoords: vec![],
            indices: vec![],
            face_arities: vec![],
            texcoord_indices: vec![],
            material_id: None,
            normal_indices: vec![],
        };

        let triangle = Triangle::new(
            Arc::new(mesh),
            0,
            1,
            2,
            vec![Material::Matte(MatteMaterial::new(
                Vector3::new(1.0, 1.0, 1.0),
                100.0,
            ))],
            None,
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
                &i.shading_normal,
                f64::EPSILON,
                1.0e-6
            )
        );

        assert_eq!(2.0, distance);
    }
}
