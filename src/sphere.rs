use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::PrimitiveTopology;
use rand::seq::SliceRandom;
use std::num::NonZero;
use subsphere::prelude::*;

#[derive(Component, Clone, Copy)]
pub struct Region(pub usize);

#[derive(Component)]
pub struct Face {
    pub neighbours: Vec<Entity>,
}

#[derive(Component)]
pub struct ChangeColour;

#[derive(Component)]
pub struct Frontier;

pub fn create_sphere(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let sphere = subsphere::HexSphere::from_kis(
        subsphere::icosphere()
            .subdivide_edge(NonZero::new(60).unwrap())
            .with_projector(subsphere::proj::Fuller),
    )
    .unwrap();

    let mut rng = rand::thread_rng();
    let mut face_entities = Vec::new();

    // First pass: create entities and store them
    for _ in 0..sphere.num_faces() {
        let entity_id = commands.spawn_empty().id();
        face_entities.push(entity_id);
    }

    // Second pass: populate entities
    for (i, face) in sphere.faces().enumerate() {
        let positions = build_fan_triangulation(face);

        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        );
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.compute_flat_normals();

        let mut neighbours = Vec::new();
        for side in face.sides() {
            let neighbour_index = side.twin().inside().index();
            neighbours.push(face_entities[neighbour_index]);
        }

        commands.entity(face_entities[i]).insert((
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(materials.add(StandardMaterial { ..default() })),
            Face { neighbours },
            Transform::from_xyz(2.0, 2.0, 0.0),
        ));
    }

    // Select starting faces for flood fill
    let n_regions = 20;
    let starting_faces = face_entities
        .choose_multiple(&mut rng, n_regions)
        .cloned()
        .collect::<Vec<_>>();
    for (i, entity) in starting_faces.iter().enumerate() {
        commands
            .entity(*entity)
            .insert((Region(i), ChangeColour, Frontier));
    }
}

fn build_fan_triangulation(face: subsphere::hex::Face<subsphere::proj::Fuller>) -> Vec<[f32; 3]> {
    let face_vertices: Vec<_> = face.vertices().map(|v| v.pos()).collect();

    let mut positions = Vec::new();
    let v0 = face_vertices[0];

    for j in 1..(face_vertices.len() - 1) {
        let v1 = face_vertices[j];
        let v2 = face_vertices[j + 1];

        positions.push([v0[0] as f32, v0[1] as f32, v0[2] as f32]);
        positions.push([v1[0] as f32, v1[1] as f32, v1[2] as f32]);
        positions.push([v2[0] as f32, v2[1] as f32, v2[2] as f32]);
    }
    positions
}

pub fn flood_fill(
    mut commands: Commands,
    q_faces: Query<(Entity, &Face, &Region), With<Frontier>>,
    q_regions: Query<&Region>,
) {
    let mut rng = rand::thread_rng();
    // iterate through the faces that are on the frontier
    for (face_entity_id, face, region) in q_faces.iter() {
        // choose a random neighbour for that face
        if let Some(neighbour_entity) = face.neighbours.choose(&mut rng) {
            // if the chosen neighour has not yet been assigned a region
            if q_regions.get(*neighbour_entity).is_err() {
                // assign it to the current face's region and mark it as on the frontier
                // also mark it to change colour
                commands
                    .entity(*neighbour_entity)
                    .insert((*region, ChangeColour, Frontier));

                // if all neighbours have been assigned a region
                if face
                    .neighbours
                    .iter()
                    .all(|&neighbour_entity_id| q_regions.get(neighbour_entity_id).is_ok())
                {
                    // the current face is no longer on the frontier
                    commands.entity(face_entity_id).remove::<Frontier>();
                }
            }
        }
    }
}

pub fn change_face_color(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<
        (Entity, &MeshMaterial3d<StandardMaterial>, &Region),
        (With<Face>, With<ChangeColour>),
    >,
) {
    let colors = color_palette();
    for (entity_id, material_handle, region) in query.iter() {
        if let Some(material) = materials.get_mut(material_handle) {
            material.base_color = colors[region.0];
        }
        commands.entity(entity_id).remove::<ChangeColour>();
    }
}

fn color_palette() -> Vec<Color> {
    // todo: made this have a nice palette
    vec![
        Color::srgb(0.9, 0.1, 0.1),
        Color::srgb(0.1, 0.9, 0.1),
        Color::srgb(0.1, 0.1, 0.9),
        Color::srgb(0.9, 0.9, 0.1),
        Color::srgb(0.1, 0.9, 0.9),
        Color::srgb(0.9, 0.1, 0.9),
        Color::srgb(0.9, 0.5, 0.1),
        Color::srgb(0.1, 0.9, 0.5),
        Color::srgb(0.5, 0.1, 0.9),
        Color::srgb(0.9, 0.1, 0.5),
        Color::srgb(0.8, 0.1, 0.1),
        Color::srgb(0.1, 0.8, 0.1),
        Color::srgb(0.1, 0.1, 0.8),
        Color::srgb(0.8, 0.8, 0.1),
        Color::srgb(0.1, 0.8, 0.8),
        Color::srgb(0.8, 0.1, 0.8),
        Color::srgb(0.8, 0.5, 0.1),
        Color::srgb(0.1, 0.8, 0.5),
        Color::srgb(0.5, 0.1, 0.8),
        Color::srgb(0.8, 0.1, 0.5),
    ]
}
