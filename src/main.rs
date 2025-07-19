#![allow(clippy::pedantic)]
//! A simple 3D scene with light shining over a cube sitting on a plane.

use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::PrimitiveTopology;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use hexasphere::shapes::NormIcoSphere;
use rand::Rng;
use std::num::NonZero;
use subsphere::{Face, prelude::*};

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
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Mesh3d(meshes.add(create_subsphere_mesh(9))),
        // Use a default material, as vertex colors will override it
        MeshMaterial3d(materials.add(StandardMaterial::default())),
        Transform::from_xyz(1.0, 0.0, 0.0),
    ));
    // hexasphere with random colors
    commands.spawn((
        Mesh3d(meshes.add(create_hexasphere_mesh(5))),
        // Use a default material, as vertex colors will override it
        MeshMaterial3d(materials.add(StandardMaterial::default())),
        Transform::from_xyz(-1.0, 0.0, 0.0),
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
    let mut colors = Vec::new();
    let mut rng = rand::thread_rng();

    for i in 0..indices.len() / 3 {
        let i1 = indices[i * 3] as usize;
        let i2 = indices[i * 3 + 1] as usize;
        let i3 = indices[i * 3 + 2] as usize;

        let p1 = Vec3::new(points[i1].x, points[i1].y, points[i1].z);
        let p2 = Vec3::new(points[i2].x, points[i2].y, points[i2].z);
        let p3 = Vec3::new(points[i3].x, points[i3].y, points[i3].z);

        positions.push(p1);
        positions.push(p2);
        positions.push(p3);

        // Generate random color for this triangle
        let color = Srgba::rgb(rng.r#gen(), rng.r#gen(), rng.r#gen());
        colors.push(color.to_f32_array());
        colors.push(color.to_f32_array());
        colors.push(color.to_f32_array());
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.compute_flat_normals();
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors); // Add colors to the mesh
    mesh
}

fn create_subsphere_mesh(subdivisions: u32) -> Mesh {
    let sphere = subsphere::HexSphere::from_kis(
        subsphere::icosphere()
            .subdivide_edge(NonZero::new(subdivisions).unwrap())
            .with_projector(subsphere::proj::Fuller),
    )
    .unwrap();

    let mut positions = Vec::new();
    let mut colors = Vec::new();
    let mut rng = rand::thread_rng();

    for face in sphere.faces() {
        let face_vertices: Vec<_> = face.vertices().collect();
        let color = Srgba::rgb(rng.r#gen(), rng.r#gen(), rng.r#gen());

        // fan triangulation
        let v0 = face_vertices[0].pos();

        for i in 1..(face_vertices.len() - 1) {
            let v1 = face_vertices[i].pos();
            let v2 = face_vertices[i + 1].pos();

            positions.push([v0[0] as f32, v0[1] as f32, v0[2] as f32]);
            positions.push([v1[0] as f32, v1[1] as f32, v1[2] as f32]);
            positions.push([v2[0] as f32, v2[1] as f32, v2[2] as f32]);

            colors.push(color.to_f32_array());
            colors.push(color.to_f32_array());
            colors.push(color.to_f32_array());
        }
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);

    mesh.compute_flat_normals();
    mesh
}
