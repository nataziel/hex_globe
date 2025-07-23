#![allow(clippy::pedantic)]
//! Generate a sphere of hexagons and pentagons, render it nicely

mod sphere;

use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::PrimitiveTopology;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};

use rand::Rng;
use rand::seq::SliceRandom;

use std::{num::NonZero, time::Duration};
use subsphere::{Face, prelude::*};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PanOrbitCameraPlugin)
        .insert_resource(Time::<Fixed>::from_duration(Duration::from_millis(200)))
        .add_systems(Startup, (setup, sphere::create_sphere))
        .add_systems(FixedUpdate, sphere::flood_fill)
        .add_systems(Update, sphere::change_face_color)
        .run();
}



/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Mesh3d(meshes.add(create_subsphere_mesh(60, 20))),
        // Use a default material, as vertex colors will override it
        MeshMaterial3d(materials.add(StandardMaterial::default())),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    // light
    let light = commands
        .spawn((
            PointLight {
                shadows_enabled: true,
                ..default()
            },
            Transform::default(),
        ))
        .id();

    // camera
    let camera = commands
        .spawn((
            Camera3d::default(),
            Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
            PanOrbitCamera::default(),
        ))
        .id();

    // light follows camera
    commands.entity(camera).add_child(light);
}

fn flood_fill_colors(
    face_region_indices: Vec<Option<usize>>,
    rng: &mut rand::prelude::ThreadRng,
    n_regions: &usize,
) -> Vec<[f32; 4]> {
    let colors = generate_colours(*n_regions, rng);

    face_region_indices
        .into_iter()
        .map(|c_idx| c_idx.map(|i| colors[i]).unwrap())
        .collect()
}

// This function implements a flood fill algorithm to color a hexagonal sphere.
// It divides the sphere into `n` distinct regions with random colors.
// The growth of these regions is controlled to prevent any single region from becoming
// disproportionately large, ensuring a more balanced and visually appealing result.
fn flood_fill_regions(
    sphere: &subsphere::HexSphere<subsphere::proj::Fuller>,
    rng: &mut rand::prelude::ThreadRng,
    n_regions: &usize,
    max_size_ratio: usize,
) -> Vec<Option<usize>> {
    let num_faces = sphere.num_faces();

    // `face_region_mapping` stores the region index for each face.
    // `region_sizes` tracks the number of faces in each region.
    let mut face_region_mapping: Vec<Option<usize>> = vec![None; num_faces];
    let mut region_sizes: Vec<usize> = vec![0; *n_regions];
    let mut unallocated_count = num_faces;

    // The `frontier` holds the indices of allocated faces that are adjacent to unallocated faces.
    let mut frontier: Vec<usize> = Vec::new();

    // Randomly select `n_regions` starting faces to act as seeds for the regions.
    let starting_faces: Vec<_> = (0..num_faces)
        .collect::<Vec<_>>()
        .choose_multiple(rng, *n_regions)
        .cloned()
        .collect();

    // Initialize the starting faces with their respective regions.
    for (region_idx, &face_idx) in starting_faces.iter().enumerate() {
        face_region_mapping[face_idx] = Some(region_idx);
        region_sizes[region_idx] += 1;
        frontier.push(face_idx);
        unallocated_count -= 1;
    }

    // Main expansion loop: continues as long as there are unallocated faces and a frontier to expand.
    while unallocated_count > 0 && !frontier.is_empty() {
        // Determine the size of the smallest active region. At this point it can never be 0.
        let min_region_size = region_sizes.iter().min().unwrap().to_owned();
        // Calculate the maximum allowed size for any region to maintain the size ratio.
        let max_allowed_size = min_region_size * max_size_ratio;

        // Identify frontier faces belonging to regions that are not yet at their size limit.
        let mut valid_frontier_indices: Vec<_> = (0..frontier.len())
            .filter(|&i| {
                let face_idx = frontier[i];
                let region_idx = face_region_mapping[face_idx].unwrap(); // Should not be None at
                // this point unless something weird has happened
                region_sizes[region_idx] < max_allowed_size
            })
            .collect();

        // If all frontier regions are at their size limit, relax the rule to allow any region to grow.
        if valid_frontier_indices.is_empty() {
            valid_frontier_indices = (0..frontier.len()).collect();
        }

        // Randomly select a face from the valid frontier to expand.
        let frontier_idx_pos = valid_frontier_indices.choose(rng).unwrap().to_owned();
        let face_index = frontier[frontier_idx_pos];

        // Find unallocated neighbours of the selected face.
        let unallocated_neighbours: Vec<_> = sphere
            .face(face_index)
            .sides()
            .map(|s| s.twin().inside().index())
            .filter(|&neighbour_idx| face_region_mapping[neighbour_idx].is_none())
            .collect();

        if unallocated_neighbours.is_empty() {
            // If the face has no unallocated neighbours, it's no longer on the frontier.
            frontier.swap_remove(frontier_idx_pos);
        } else if let Some(&neighbour_to_allocate) = unallocated_neighbours.choose(rng) {
            // Allocate a random unallocated neighbour and add it to the frontier.
            let region_index = face_region_mapping[face_index].unwrap();
            face_region_mapping[neighbour_to_allocate] = Some(region_index);
            region_sizes[region_index] += 1;
            frontier.push(neighbour_to_allocate);
            unallocated_count -= 1;
        }
    }

    // Final cleanup loop: ensures any remaining unallocated faces are filled.
    // This loop continues until no more changes can be made.
    loop {
        let mut changed = false;
        for i in 0..num_faces {
            if face_region_mapping[i].is_none() {
                // Find allocated neighbours of the unallocated face.
                let neighbour_regions: Vec<_> = sphere
                    .face(i)
                    .sides()
                    .filter_map(|s| face_region_mapping[s.twin().inside().index()])
                    .collect();

                // Randomly adopt the region of one of its neighbours.
                if let Some(&neighbour_region_index) = neighbour_regions.choose(rng) {
                    face_region_mapping[i] = Some(neighbour_region_index);
                    changed = true;
                }
            }
        }

        if !changed {
            break;
        }
    }
    face_region_mapping
}

// Generate `n` distinct random colors for the regions.
fn generate_colours(n: usize, rng: &mut rand::prelude::ThreadRng) -> Vec<[f32; 4]> {
    let colors: Vec<_> = (0..n)
        .map(|_| Srgba::rgb(rng.r#gen(), rng.r#gen(), rng.r#gen()).to_f32_array())
        .collect();
    colors
}

fn create_subsphere_mesh(subdivisions: u32, n_regions: usize) -> Mesh {
    let sphere = subsphere::HexSphere::from_kis(
        subsphere::icosphere()
            .subdivide_edge(NonZero::new(subdivisions).unwrap())
            .with_projector(subsphere::proj::Fuller),
    )
    .unwrap();

    let mut rng = rand::thread_rng();

    let face_region_indices = flood_fill_regions(&sphere, &mut rng, &n_regions, 3);
    let face_colors = flood_fill_colors(face_region_indices, &mut rng, &n_regions);

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
