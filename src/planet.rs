#![allow(clippy::type_complexity, clippy::identity_op)]
use bevy::render::mesh::VertexAttributeValues;
use bevy::render::render_resource::PrimitiveTopology;
use bevy::{prelude::*, render::mesh::Indices};
use bevy_inspector_egui::Inspectable;
use noise::{NoiseFn, Perlin};

#[derive(Debug, Clone, Inspectable)]
pub struct PlanetGeneratorData {
    pub radius: f64,
    pub noise_settings: [NoiseSettings; 3],
    pub resolution: i32,
    pub origin: Vec3,
}

#[derive(Debug, Clone, Default, Inspectable)]
pub struct NoiseSettings {
    pub seed: u64,
    pub origin: Vec3,
    #[inspectable(min = 0.0)]
    pub amplitude: f64,
    #[inspectable(min = 0.0)]
    pub frequency: f32,
    pub mask_by_previous: bool,
}

impl Default for PlanetGeneratorData {
    fn default() -> Self {
        Self {
            origin: Vec3::new(0.0, 10.0, -50.0),
            radius: 20.0,
            noise_settings: [
                NoiseSettings {
                    seed: 0,
                    origin: Vec3::new(0.0, 0.0, 0.0),
                    amplitude: 1.0,
                    frequency: 1.0,
                    mask_by_previous: false,
                },
                NoiseSettings {
                    seed: 1,
                    origin: Vec3::new(0.0, 0.0, 0.0),
                    amplitude: 1.0,
                    frequency: 1.0,
                    mask_by_previous: false,
                },
                NoiseSettings {
                    seed: 2,
                    origin: Vec3::new(0.0, 0.0, 0.0),
                    amplitude: 1.0,
                    frequency: 1.0,
                    mask_by_previous: false,
                },
            ],
            resolution: 32,
        }
    }
}

pub struct PlanetShape {
    resolution: u8,
    radius: f64,
    noise_settings: [NoiseSettings; 3],
}

impl PlanetShape {
    pub fn new(data: &PlanetGeneratorData) -> Self {
        Self {
            resolution: data.resolution as u8,
            radius: data.radius,
            noise_settings: data.noise_settings.clone(),
        }
    }

    fn get_noise_value(&self, noise_fn: Perlin, vertex_direction_vec: Vec3) -> f64 {
        let mut noise_value = 0.0;
        for noise_setting in self.noise_settings.iter() {
            if noise_setting.mask_by_previous && noise_value == 0.0 {
                continue;
            }
            let noise_orig = noise_setting.origin + vertex_direction_vec * noise_setting.frequency;
            let n = noise_fn.get([
                noise_orig.x.into(),
                noise_orig.y.into(),
                noise_orig.z.into(),
            ]) * noise_setting.amplitude;

            noise_value += n;
        }

        noise_value.max(0.0)
    }

    fn generate_mesh(&self) -> Mesh {
        // Noise funciton to use for all sampling
        let noise = Perlin::new();
        let resolution = self.resolution;

        // Mesh data initialization
        let mut uvs: Vec<[f32; 2]> = vec![];
        let mut colors: Vec<[f32; 4]> = vec![];
        let mut index_values: Vec<u32> = vec![];
        let mut normals: Vec<[f32; 3]> = vec![];
        let mut vertices: Vec<[f32; 3]> = vec![];

        let origin = Vec3::new(0.0, 0.0, 0.0);

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        let start = -1.;
        // let mut normals: Vec<[f32; 3]> = vec![];
        let mut current_index: u32 = 0;

        let rotations = [
            Mat3::from_rotation_x(0.0),
            Mat3::from_rotation_x(std::f32::consts::PI / 2.0),
            Mat3::from_rotation_x(-std::f32::consts::PI / 2.0),
            Mat3::from_rotation_x(std::f32::consts::PI),
            Mat3::from_rotation_y(std::f32::consts::PI / 2.0),
            Mat3::from_rotation_y(-std::f32::consts::PI / 2.0),
        ];

        for &rotation in &rotations {
            for y in 0..resolution {
                let y_percentage = y as f32 / (resolution - 1) as f32;
                let is_bottom_edge = y_percentage == 1.;

                for x in 0..resolution {
                    let x_percentage = x as f32 / (resolution - 1) as f32;
                    let is_right_edge = x_percentage == 1.;

                    let uv = [x_percentage, y_percentage];

                    let vertex_direction_vec = rotation
                        * Vec3::new(start + 2. * x_percentage, start + 2. * y_percentage, 1.)
                            .normalize();
                    let offset = self.get_noise_value(noise, vertex_direction_vec);

                    let vertex_position =
                        origin + vertex_direction_vec * ((self.radius + offset) as f32);

                    vertices.extend([[vertex_position.x, -vertex_position.y, vertex_position.z]]);
                    uvs.push(uv);
                    colors.push([1.0, 0.8, 1.0, 1.0]);

                    normals.push([
                        vertex_direction_vec.x,
                        vertex_direction_vec.y,
                        vertex_direction_vec.z,
                    ]);

                    if !(is_right_edge || is_bottom_edge) {
                        index_values.extend([
                            current_index,
                            current_index + resolution as u32,
                            current_index + 1,
                            current_index + resolution as u32,
                            current_index + resolution as u32 + 1,
                            current_index + 1,
                        ]);
                    }

                    current_index += 1;
                }
            }
        }

        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_NORMAL,
            self.calculate_normals_for_mesh(&mesh, resolution as usize),
        );
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
        mesh.set_indices(Some(Indices::U32(index_values)));
        mesh.generate_tangents().unwrap();

        mesh
    }

    fn calculate_normals_for_mesh(&self, mesh: &Mesh, resolution: usize) -> Vec<[f32; 3]> {
        let mut normals: Vec<[f32; 3]> = vec![];
        if let Some(VertexAttributeValues::Float32x3(vertices)) =
            mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        {
            for (i, vertex) in vertices.iter().enumerate() {
                let is_right_edge = i % resolution == resolution - 1;
                let is_bottom_edge = i >= (resolution * (resolution - 1));

                let vertex_vec = Vec3::new(vertex[0], vertex[1], vertex[2]);

                let x_dir: i32 = if is_right_edge { -1 } else { 1 };
                let y_dir: i32 = if is_bottom_edge { -1 } else { 1 };

                let x_vertex = vertices[(i as i32 + x_dir) as usize];
                let y_vertex = vertices[(i as i32 + y_dir * resolution as i32) as usize];

                let x_dir_vec = Vec3::new(x_vertex[0], x_vertex[1], x_vertex[2]) - vertex_vec;
                let y_dir_vec = Vec3::new(y_vertex[0], y_vertex[1], y_vertex[2]) - vertex_vec;

                let normal = x_dir_vec.cross(y_dir_vec).normalize();

                let normal_multiplier = normal.dot(Vec3::from(*vertex)).signum();

                normals.push((normal * normal_multiplier).into());
            }
        }

        normals
    }
}

impl From<PlanetShape> for Mesh {
    fn from(shape: PlanetShape) -> Self {
        shape.generate_mesh()
    }
}
