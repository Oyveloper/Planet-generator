use std::f32::consts::FRAC_PI_2;

use bevy::reflect::{FromReflect, TypeUuid};
use bevy::render::mesh::Indices;
use bevy::render::render_resource::PrimitiveTopology;
use bevy::{
    input::mouse::MouseMotion,
    prelude::shape,
    prelude::*,
    render::{mesh::VertexAttributeValues, render_resource::AsBindGroup},
};

use bevy_inspector_egui::{Inspectable, WorldInspectorPlugin};
use bevy_rapier3d::prelude::*;

use noise::NoiseFn;
use noise::Perlin;

mod planet;

const TIME_STEP: f32 = 1.0 / 60.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // .add_plugin(LogDiagnosticsPlugin::default())
        // .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        // .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(WorldInspectorPlugin::default())
        .add_startup_system(setup)
        .add_system(bevy::window::close_on_esc)
        // .add_startup_system(grab_mouse)
        .add_system(camera_move_system)
        // .add_system(camera_rotate_system)
        .run();
}

#[derive(Component, Default)]
struct Camera {
    movement_speed: f32,
    rotation_speed: f32,

    pitch: f32,
    yaw: f32,
}

#[derive(Inspectable)]
struct PlanetData {
    radius: f32,
    resolution: u8,
    noise_settings: NoiseSetting,
}

#[derive(Inspectable, Clone)]
struct NoiseSetting {
    filters: Vec<NoiseFilter>,
}

#[derive(Component, Default, Clone, Copy, Inspectable)]
struct NoiseFilter {
    pub seed: Vec3,
    pub scale: f64,
    pub strength: f64,
}

impl NoiseFilter {
    pub fn new(seed: Vec3, scale: f64, strength: f64) -> Self {
        Self {
            seed,
            scale,
            strength,
        }
    }

    pub fn get_noise(&self, x: f64, y: f64, z: f64) -> f64 {
        let mut base_noise_fn = Perlin::new();
        base_noise_fn.get([
            (x + self.seed.x as f64) * self.scale,
            (y + self.seed.y as f64) * self.scale,
            (z + self.seed.z as f64) * self.scale,
        ]) * self.strength
    }
}

#[derive(Component, Default, Reflect, Copy, Clone)]
#[reflect(Component)]
struct Planet {}

#[derive(AsBindGroup, TypeUuid, Clone)]
#[uuid = "1319ad1c-420a-4c34-8abe-62d6508588c4"]
pub struct CoolMaterial {}

impl Material for CoolMaterial {}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    // asset_server: Res<AssetServer>,
) {
    let planet = Planet {
        radius: 1.0,
        seed: Vec3::new(0.0, 0.0, 0.0),
        resolution: 200,
        origin: Vec3::new(0.0, 0.0, 0.0),
    };
    let planes = generate_planet(&planet);
    for plane in planes {
        commands
            .spawn_bundle(PbrBundle {
                mesh: meshes.add(plane),
                material: materials.add(Color::WHITE.into()),
                ..default()
            })
            .insert(planet);
    }
    // Bouncing ball
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Icosphere {
                radius: 1.0,
                subdivisions: 6,
            })),
            material: materials.add(StandardMaterial {
                base_color: Color::rgb(0.3, 0.6, 0.9).into(),
                metallic: 1.0,
                perceptual_roughness: 0.5,
                ..Default::default()
            }),
            ..default()
        })
        // .insert(RigidBody::Dynamic)
        .insert(Collider::ball(1.))
        .insert(Restitution::coefficient(0.7))
        .insert_bundle(TransformBundle::from(Transform::from_xyz(0.0, 14.0, 0.0)));

    // Lighting
    const HALF_SIZE: f32 = 10.0;
    commands.spawn_bundle(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadow_projection: OrthographicProjection {
                left: -HALF_SIZE,
                right: HALF_SIZE,
                bottom: -HALF_SIZE,
                top: HALF_SIZE,
                near: -10.0 * HALF_SIZE,
                far: 10.0 * HALF_SIZE,
                ..default()
            },
            shadows_enabled: true,
            ..default()
        },
        transform: Transform {
            translation: Vec3::Y * 2.0,
            rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4),
            ..default()
        },
        ..default()
    });

    commands.insert_resource(AmbientLight {
        color: Color::ANTIQUE_WHITE,
        brightness: 0.2,
    });

    // camera
    commands
        .spawn_bundle(Camera3dBundle {
            transform: Transform::from_xyz(0.0, 20.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        })
        .insert(Camera {
            movement_speed: 2.0,
            rotation_speed: 1.7,
            ..default()
        });
}
fn planet_generate_system(
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<(&Planet, &Handle<Mesh>)>,
) {
    for (planet, mesh) in query.iter() {
        let mesh = meshes.get_mut(mesh).unwrap();

        *mesh = generate_planet(&planet)[0];
        // material.base_color = data.color;
    }
}

fn generate_planet(planet: &Planet) -> Vec<Mesh> {
    let rotations = [
        Mat3::from_rotation_x(0.0),
        Mat3::from_rotation_x(std::f32::consts::PI / 2.0),
        Mat3::from_rotation_x(-std::f32::consts::PI / 2.0),
        Mat3::from_rotation_x(std::f32::consts::PI),
        Mat3::from_rotation_y(std::f32::consts::PI / 2.0),
        Mat3::from_rotation_y(-std::f32::consts::PI / 2.0),
    ];

    let noise = Perlin::new();

    println!("{:?}", noise.get([0.0, 0.0, 0.0]));

    rotations
        .iter()
        .map(|rot| {
            generate_planet_side_plane(planet.resolution, planet.origin, planet.radius, *rot)
        })
        .collect()
}

fn calculate_normals_for_mesh(mesh: &Mesh, resolution: usize) -> Vec<[f32; 3]> {
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

fn generate_planet_side_plane(
    resolution: u8,
    origin: Vec3,
    planet_radius: f32,
    rotation: Mat3,
) -> Mesh {
    let noise = Perlin::new();

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    let start = -1.;
    let mut vertices: Vec<[f32; 3]> = vec![];
    // let mut normals: Vec<[f32; 3]> = vec![];
    let mut uvs: Vec<[f32; 2]> = vec![];
    let mut colors: Vec<[f32; 4]> = vec![];
    let mut index_values: Vec<u32> = vec![];
    let mut current_index: u32 = 0;
    for i in 0..resolution {
        let y_percentage = i as f32 / (resolution - 1) as f32;
        let is_bottom_edge = y_percentage == 1.;
        for j in 0..resolution {
            let x_percentage = j as f32 / (resolution - 1) as f32;
            let is_right_edge = x_percentage == 1.;

            let uv = [x_percentage, y_percentage];

            let vertex_direction_vec = rotation
                * Vec3::new(start + 2. * x_percentage, start + 2. * y_percentage, 1.).normalize();
            let offset = noise.get([
                vertex_direction_vec.x.into(),
                vertex_direction_vec.y.into(),
                vertex_direction_vec.z.into(),
            ]) * 0.7;

            let vertex_position = origin + vertex_direction_vec * (planet_radius + offset as f32);

            vertices.extend([[vertex_position.x, -vertex_position.y, vertex_position.z]]);
            uvs.push(uv);
            colors.push([1.0, 0.8, 1.0, 1.0]);

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

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        calculate_normals_for_mesh(&mesh, resolution as usize),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.set_indices(Some(Indices::U32(index_values)));
    mesh.generate_tangents().unwrap();

    mesh
}

fn camera_move_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&Camera, &mut Transform)>,
) {
    let (camera, mut transform) = query.single_mut();

    let mut forward_factor = 0.0;
    let mut sideways_factor = 0.0;
    let mut up_factor = 0.0;

    if keyboard_input.pressed(KeyCode::W) {
        forward_factor = -1.0
    }
    if keyboard_input.pressed(KeyCode::S) {
        forward_factor = 1.0
    }
    if keyboard_input.pressed(KeyCode::A) {
        sideways_factor = -1.0
    }
    if keyboard_input.pressed(KeyCode::D) {
        sideways_factor = 1.0
    }

    if keyboard_input.pressed(KeyCode::Space) {
        up_factor = 1.0
    }
    if keyboard_input.pressed(KeyCode::LShift) {
        up_factor = -1.0
    }

    let forward = transform.rotation * Vec3::new(0.0, 0.0, 1.0);
    let right = transform.rotation * Vec3::new(1.0, 0.0, 0.0);
    let move_factor = camera.movement_speed * TIME_STEP;

    transform.translation += forward * forward_factor * move_factor;
    transform.translation += right * sideways_factor * move_factor;
    transform.translation += Vec3::Y * up_factor * move_factor;
}

fn camera_rotate_system(
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut query: Query<(&mut Camera, &mut Transform)>,
) {
    let (mut camera, mut transform) = query.single_mut();

    for event in mouse_motion_events.iter() {
        let mouse_delta = event.delta;

        let yaw_delta = -mouse_delta.x * 0.5 * TIME_STEP * camera.rotation_speed;
        let pitch_delta = -mouse_delta.y * 0.5 * TIME_STEP * camera.rotation_speed;

        camera.yaw += yaw_delta;
        camera.pitch += pitch_delta;
        camera.pitch = camera.pitch.clamp(-FRAC_PI_2, FRAC_PI_2);

        transform.rotation =
            Quat::from_rotation_y(camera.yaw) * Quat::from_rotation_x(camera.pitch);
    }
}

fn grab_mouse(mut windows: ResMut<Windows>) {
    let window = windows.get_primary_mut().unwrap();
    window.set_cursor_visibility(false);
    window.set_cursor_lock_mode(true);
}
