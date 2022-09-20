use bevy::prelude::*;
use bevy_inspector_egui::Inspectable;

#[derive(Debug, Clone, Default, Inspectable, Component)]
struct Planet {
    radius: f64,
    origin: Vec3,
    rasolution: u8,
    noise_settings: NoiseSettings,
}

#[derive(Debug, Clone, Default, Inspectable, Component)]
struct NoiseSettings {
    frequency: f64,
    lacunarity: f64,
    persistence: f64,
    octaves: u8,
}

struct PlanetShape {}
