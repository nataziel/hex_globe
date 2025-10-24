#![warn(clippy::pedantic)]
#![allow(clippy::cast_possible_truncation)]
//! Generate a sphere of hexagons and pentagons, render it nicely

mod setup;
mod states;
mod ui;
mod worldgen;

use bevy::prelude::*;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};

use std::time::Duration;

use crate::{setup::SetupPlugin, states::StatePlugin, ui::UiPlugin, worldgen::WorldGenPlugin};

const TICK_RATE: u64 = 100;

fn main() -> AppExit {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PanOrbitCameraPlugin)
        .insert_resource(Time::<Fixed>::from_duration(Duration::from_millis(
            TICK_RATE,
        )))
        .add_systems(Startup, setup)
        .add_plugins(SetupPlugin)
        .add_plugins(WorldGenPlugin)
        .add_plugins(StatePlugin)
        .add_plugins(UiPlugin)
        .add_systems(Update, update_directional_light)
        .run()
}

#[derive(Component)]
struct OrbitingDirectionalLight;

/// set up a simple 3D scene
fn setup(mut commands: Commands) {
    // light
    commands.spawn((
        DirectionalLight {
            ..Default::default()
        },
        Transform::default(),
        OrbitingDirectionalLight,
    ));

    // camera
    commands.spawn((
        Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
        PanOrbitCamera::default(),
    ));
}

fn update_directional_light(
    camera_query: Query<
        &Transform,
        // need to disambiguate query to keep bevy happy
        // this is so we don't reference the same value mutably & immutably at the same time
        (With<PanOrbitCamera>, Without<OrbitingDirectionalLight>),
    >,
    mut light_query: Query<&mut Transform, With<OrbitingDirectionalLight>>,
) {
    let camera_transform = camera_query.single().unwrap();
    let mut light_transform = light_query.single_mut().unwrap();

    let focus = Vec3::ZERO; // we always focus on the origin
    // camera is facing directly towards the focus point (flip the camera transform vector)
    let camera_direction = (focus - camera_transform.translation).normalize();

    // Rotate the light so it shines from the same direction as the camera
    let light_forward = -Vec3::Z; // this is the default rotation of a transform in bevy
    // the rotation of a transform is always calculated in reference to the default
    let rotation = Quat::from_rotation_arc(light_forward, camera_direction);
    light_transform.rotation = rotation;
}
