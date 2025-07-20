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
        Mesh3d(meshes.add(create_subsphere_mesh(18))),
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
    const MAX_SIZE_RATIO: f32 = 3.0;

    let mut rng = rand::thread_rng();
    let num_faces = sphere.num_faces();

    let mut face_color_indices: Vec<Option<usize>> = vec![None; num_faces];
    let mut region_sizes = vec![0; n];
    let mut uncolored_count = num_faces;

    let colors: Vec<_> = (0..n)
        .map(|_| Srgba::rgb(rng.r#gen(), rng.r#gen(), rng.r#gen()).to_f32_array())
        .collect();

    let mut frontier: Vec<usize> = Vec::new();

    let starting_faces: Vec<_> = (0..num_faces)
        .collect::<Vec<_>>()
        .choose_multiple(&mut rng, n)
        .cloned()
        .collect();

    for (i, &face_index) in starting_faces.iter().enumerate() {
        if face_color_indices[face_index].is_none() {
            face_color_indices[face_index] = Some(i);
            region_sizes[i] += 1;
            frontier.push(face_index);
            uncolored_count -= 1;
        }
    }

    while uncolored_count > 0 && !frontier.is_empty() {
        let min_region_size = region_sizes
            .iter()
            .filter(|&&s| s > 0)
            .min()
            .unwrap_or(&0)
            .to_owned();
        let max_allowed_size = (min_region_size as f32 * MAX_SIZE_RATIO) as usize;

        let mut valid_frontier_indices: Vec<_> = (0..frontier.len())
            .filter(|&i| {
                let face_idx = frontier[i];
                let color_idx = face_color_indices[face_idx].unwrap();
                region_sizes[color_idx] < max_allowed_size
            })
            .collect();

        if valid_frontier_indices.is_empty() {
            valid_frontier_indices = (0..frontier.len()).collect();
        }

        let frontier_idx_pos = valid_frontier_indices.choose(&mut rng).unwrap().to_owned();
        let face_index = frontier[frontier_idx_pos];
        let color_index = face_color_indices[face_index].unwrap();

        let uncolored_neighbors: Vec<_> = sphere
            .face(face_index)
            .sides()
            .map(|s| s.twin().inside().index())
            .filter(|&neighbor_idx| face_color_indices[neighbor_idx].is_none())
            .collect();

        if uncolored_neighbors.is_empty() {
            frontier.swap_remove(frontier_idx_pos);
        } else if let Some(&neighbor_to_color) = uncolored_neighbors.choose(&mut rng) {
            face_color_indices[neighbor_to_color] = Some(color_index);
            region_sizes[color_index] += 1;
            uncolored_count -= 1;
            frontier.push(neighbor_to_color);
        }
    }

    loop {
        let mut changed = false;
        for i in 0..num_faces {
            if face_color_indices[i].is_none() {
                let neighbor_colors: Vec<_> = sphere
                    .face(i)
                    .sides()
                    .filter_map(|s| face_color_indices[s.twin().inside().index()])
                    .collect();

                if let Some(&neighbor_color_index) = neighbor_colors.choose(&mut rng) {
                    face_color_indices[i] = Some(neighbor_color_index);
                    changed = true;
                }
            }
        }

        if !changed {
            break;
        }
    }

    face_color_indices
        .into_iter()
        .map(|c_idx| c_idx.map(|i| colors[i]).unwrap_or([0.0, 0.0, 0.0, 1.0]))
        .collect()
}

fn create_subsphere_mesh(subdivisions: u32) -> Mesh {
    let sphere = subsphere::HexSphere::from_kis(
        subsphere::icosphere()
            .subdivide_edge(NonZero::new(subdivisions).unwrap())
            .with_projector(subsphere::proj::Fuller),
    )
    .unwrap();

    let face_colors = flood_fill_colors(&sphere, 20);

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
