//! A simple 3D scene with light shining over a cube sitting on a plane.

use bevy::prelude::*;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use bevy::render::render_resource::PrimitiveTopology;
use bevy::render::render_asset::RenderAssetUsages;
use hexasphere::shapes::NormIcoSphere;
use rand::Rng;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PanOrbitCameraPlugin)
        .add_systems(Startup, setup)
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,) {
    // circular base
    commands.spawn((
        Mesh3d(meshes.add(Circle::new(4.0))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    ));
    // hexasphere with random colors
    commands.spawn((
        Mesh3d(meshes.add(create_hexasphere_mesh(5))),
        // Use a default material, as vertex colors will override it
        MeshMaterial3d(materials.add(StandardMaterial::default())),
        Transform::from_xyz(0.0, 1.0, 0.0),
    ));
    // light
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));
    // camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
        PanOrbitCamera::default(),
    ));
}

fn create_hexasphere_mesh(subdivisions: u32) -> Mesh {
    let sphere = NormIcoSphere::new(subdivisions.try_into().unwrap(), |_| ());
    let points = sphere.raw_points();
    let indices = sphere.get_all_indices();

    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut colors = Vec::new();
    let mut rng = rand::thread_rng();

    for i in 0..indices.len() / 3 {
        let i1 = indices[i * 3] as usize;
        let i2 = indices[i * 3 + 1] as usize;
        let i3 = indices[i * 3 + 2] as usize;

        let p1 = Vec3::new(points[i1].x as f32, points[i1].y as f32, points[i1].z as f32);
        let p2 = Vec3::new(points[i2].x as f32, points[i2].y as f32, points[i2].z as f32);
        let p3 = Vec3::new(points[i3].x as f32, points[i3].y as f32, points[i3].z as f32);

        positions.push(p1);
        positions.push(p2);
        positions.push(p3);

        // Calculate face normal (for flat shading)
        let normal = (p2 - p1).cross(p3 - p1).normalize();
        normals.push(normal);
        normals.push(normal);
        normals.push(normal);

        // Generate random color for this triangle
        let color = Srgba::rgb(rng.r#gen(), rng.r#gen(), rng.r#gen());
        colors.push(color.to_f32_array());
        colors.push(color.to_f32_array());
        colors.push(color.to_f32_array());
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors); // Add colors to the mesh
    mesh
}
