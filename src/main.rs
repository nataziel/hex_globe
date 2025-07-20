#![allow(clippy::pedantic)]
//! A simple 3D scene with light shining over a cube sitting on a plane.

use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::PrimitiveTopology;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};

use rand::Rng;
use rand::seq::SliceRandom;

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
        Transform::from_xyz(0.0, 0.0, 0.0),
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

fn flood_fill_colors(
    sphere: &subsphere::HexSphere<subsphere::proj::Fuller>,
    n: usize,
) -> Vec<[f32; 4]> {
    let mut rng = rand::thread_rng();
    let mut face_colors = vec![[0.0, 0.0, 0.0, 0.0]; sphere.num_faces()];
    let mut uncolored_faces: Vec<_> = (0..sphere.num_faces()).collect();
    let mut regions: Vec<Vec<usize>> = Vec::new();

    let colors: Vec<_> = (0..n)
        .map(|_| Srgba::rgb(rng.r#gen(), rng.r#gen(), rng.r#gen()).to_f32_array())
        .collect();

    let starting_faces: Vec<_> = uncolored_faces
        .choose_multiple(&mut rng, n)
        .cloned()
        .collect();

    for (i, &face_index) in starting_faces.iter().enumerate() {
        face_colors[face_index] = colors[i];
        regions.push(vec![face_index]);
        uncolored_faces.retain(|&x| x != face_index);
    }

    while !uncolored_faces.is_empty() {
        let i = rng.gen_range(0..regions.len());
        if let Some(region) = regions.get_mut(i) {
            let mut new_neighbors = Vec::new();
            for &face_index in region.iter() {
                let face = sphere.face(face_index);
                for side in face.sides() {
                    let neighbor = side.twin().inside();
                    if uncolored_faces.contains(&neighbor.index()) {
                        new_neighbors.push(neighbor.index());
                    }
                }
            }

            if let Some(neighbor_to_color) = new_neighbors.choose(&mut rng) {
                face_colors[*neighbor_to_color] = colors[i];
                region.push(*neighbor_to_color);
                uncolored_faces.retain(|&x| x != *neighbor_to_color);
            }
        }
    }

    face_colors
}

fn create_subsphere_mesh(subdivisions: u32) -> Mesh {
    let sphere = subsphere::HexSphere::from_kis(
        subsphere::icosphere()
            .subdivide_edge(NonZero::new(subdivisions).unwrap())
            .with_projector(subsphere::proj::Fuller),
    )
    .unwrap();

    let face_colors = flood_fill_colors(&sphere, 10);

    let mut positions = Vec::new();
    let mut colors = Vec::new();

    for (face_index, face) in sphere.faces().enumerate() {
        let face_vertices: Vec<_> = face.vertices().collect();
        let face_color = face_colors[face_index];

        // fan triangulation
        let v0 = face_vertices[0].pos();

        for i in 1..(face_vertices.len() - 1) {
            let v1 = face_vertices[i].pos();
            let v2 = face_vertices[i + 1].pos();

            positions.push([v0[0] as f32, v0[1] as f32, v0[2] as f32]);
            positions.push([v1[0] as f32, v1[1] as f32, v1[2] as f32]);
            positions.push([v2[0] as f32, v2[1] as f32, v2[2] as f32]);

            colors.push(face_color);
            colors.push(face_color);
            colors.push(face_color);
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
