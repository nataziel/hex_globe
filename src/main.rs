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
    let num_faces = sphere.num_faces();

    // Using Option allows quick O(1) checks for whether a face is colored.
    let mut face_colors: Vec<Option<[f32; 4]>> = vec![None; num_faces];
    let mut uncolored_count = num_faces;

    // Generate n distinct colors.
    let colors: Vec<_> = (0..n)
        .map(|_| Srgba::rgb(rng.r#gen(), rng.r#gen(), rng.r#gen()).to_f32_array())
        .collect();

    // The frontier contains the indices of colored faces that are adjacent to uncolored faces.
    let mut frontier: Vec<usize> = Vec::new();

    // Choose n random starting faces without replacement.
    let starting_faces: Vec<_> = (0..num_faces)
        .collect::<Vec<_>>()
        .choose_multiple(&mut rng, n)
        .cloned()
        .collect();

    for (i, &face_index) in starting_faces.iter().enumerate() {
        if face_colors[face_index].is_none() {
            face_colors[face_index] = Some(colors[i]);
            frontier.push(face_index);
            uncolored_count -= 1;
        }
    }

    // Expansion Loop
    while uncolored_count > 0 && !frontier.is_empty() {
        // Pick a random face from the frontier to expand from.
        let frontier_idx_pos = rng.gen_range(0..frontier.len());
        let face_index = frontier[frontier_idx_pos];
        let color = face_colors[face_index].unwrap(); // It must have a color to be in the frontier.

        // Find uncolored neighbors of the chosen frontier face.
        let uncolored_neighbors: Vec<_> = sphere
            .face(face_index)
            .sides()
            .map(|s| s.twin().inside().index())
            .filter(|&neighbor_idx| face_colors[neighbor_idx].is_none())
            .collect();

        if uncolored_neighbors.is_empty() {
            // This face is no longer on the frontier, remove it.
            // Fast removal by swapping with the last element.
            frontier.swap_remove(frontier_idx_pos);
        } else {
            // Pick a random neighbor to color.
            if let Some(&neighbor_to_color) = uncolored_neighbors.choose(&mut rng) {
                face_colors[neighbor_to_color] = Some(color);
                uncolored_count -= 1;
                // The newly colored neighbor is now part of the frontier.
                frontier.push(neighbor_to_color);
            }
        }
    }

    // Finalization
    // Convert Vec<Option<[f32; 4]>> to Vec<[f32; 4]>.
    // Any remaining `None`s (shouldn't happen in a connected graph) get a default color.
    face_colors
        .into_iter()
        .map(|c| c.unwrap_or([0.0, 0.0, 0.0, 1.0])) // Default to black
        .collect()
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
