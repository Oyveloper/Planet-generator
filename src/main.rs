use std::f32::consts::{FRAC_PI_2, TAU};

use bevy::reflect::TypeUuid;
use bevy::render::mesh::Indices;
use bevy::render::render_resource::PrimitiveTopology;
use bevy::{
    input::mouse::MouseMotion,
    prelude::shape,
    prelude::*,
    render::{mesh::VertexAttributeValues, render_resource::AsBindGroup},
};

use bevy_inspector_egui::{Inspectable, InspectorPlugin};
use bevy_rapier3d::prelude::*;

use noise::NoiseFn;
use noise::Perlin;

use crate::planet::{NoiseSettings, PlanetGeneratorData, PlanetShape};
mod planet;

const TIME_STEP: f32 = 1.0 / 60.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // .add_plugin(LogDiagnosticsPlugin::default())
        // .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        // .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(InspectorPlugin::<PlanetGeneratorData>::new())
        .add_startup_system(setup)
        .add_system(generate)
        .add_system(rotate_planet_system)
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

#[derive(Component)]
struct Planet;

#[derive(AsBindGroup, TypeUuid, Clone)]
#[uuid = "1319ad1c-420a-4c34-8abe-62d6508588c4"]
pub struct CoolMaterial {}

impl Material for CoolMaterial {}

fn generate(
    data: Res<PlanetGeneratorData>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<(&Planet, &Handle<Mesh>, &Handle<StandardMaterial>)>,
) {
    if !data.is_changed() {
        return;
    }

    for (_, mesh, material) in query.iter() {
        let mesh = meshes.get_mut(mesh).unwrap();
        // let material = materials.get_mut(material).unwrap();

        *mesh = PlanetShape::new(&data).into()
    }
}

fn rotate_planet_system(
    time: Res<Time>,
    data: Res<PlanetGeneratorData>,
    mut query: Query<&mut Transform>,
) {
    for mut transform in &mut query {
        // transform.rotate_y(
        //     // data.origin
        //     // Quat::from_rotation_y(time.delta_seconds() * TAU * 0.2),
        //     // time.delta_seconds() * TAU * 0.2,
        // );
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    planet_data: Res<PlanetGeneratorData>,
    // asset_server: Res<AssetServer>,
) {
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(PlanetShape::new(&planet_data).into()),
            material: materials.add(Color::WHITE.into()),
            transform: Transform::from_translation(planet_data.origin),
            ..default()
        })
        .insert(Planet);

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
            transform: Transform::from_xyz(0.0, 20.0, 10.0).looking_at(planet_data.origin, Vec3::Y),
            ..default()
        })
        .insert(Camera {
            movement_speed: 2.0,
            rotation_speed: 1.7,
            ..default()
        });
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
