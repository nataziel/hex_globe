use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::PrimitiveTopology;
use rand::{Rng, rngs::ThreadRng, seq::IndexedRandom};
use std::num::NonZero;
use subsphere::prelude::*;

use crate::states::WorldGenState;

const N_PLATES: usize = 40;

#[derive(Component, Clone, Copy, PartialEq)]
pub struct Plate(pub usize);

#[derive(Resource)]
struct PlatePalette(Vec<Color>);

#[derive(Component)]
pub struct Face {
    pub centre_pos: Vec3,
}

#[derive(Component)]
pub struct FaceNeighbours(Vec<Entity>);

#[derive(Component)]
pub struct ChangeColour {
    pub colour: Color,
}

#[derive(Component)]
pub struct PlateBoundary;

#[derive(Component)]
pub struct PlateGenFrontier;

#[derive(Component)]
pub struct Land;

#[derive(Component)]
pub struct Sea;

#[derive(Component)]
pub struct FacePlateVelocity {
    pub velocity: Vec3,
}

fn create_sphere(
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

    let mut rng = rand::rng();
    let mut face_entities = Vec::new();

    // First pass: create entities and store them
    for _ in 0..sphere.num_faces() {
        let entity_id = commands.spawn_empty().id();
        face_entities.push(entity_id);
    }

    // Second pass: populate entities
    for (i, face) in sphere.faces().enumerate() {
        let centre_pos = get_centre_vec(face);

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
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, 1.0, 1.0),
                ..default()
            })),
            Face { centre_pos },
            FaceNeighbours(neighbours),
            Transform::from_xyz(0.0, 0.0, 0.0),
        ));
    }

    // Create plates colour palette
    let colour_palette = gen_colour_palette(N_PLATES, &mut rng);
    commands.insert_resource(PlatePalette(colour_palette.clone()));
}

fn get_centre_vec(face: subsphere::hex::Face<subsphere::proj::Fuller>) -> Vec3 {
    let centre_position_array = face.center().pos();
    Vec3::new(
        centre_position_array[0] as f32,
        centre_position_array[1] as f32,
        centre_position_array[2] as f32,
    )
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

// Select starting faces for flood fill
fn seed_flood_fill(
    mut commands: Commands,
    face_query: Query<Entity, With<Face>>,
    palette: Res<PlatePalette>,
) {
    let face_entities: Vec<Entity> = face_query.into_iter().collect();

    let mut rng = rand::rng();

    let starting_faces = face_entities
        .choose_multiple(&mut rng, N_PLATES)
        .copied()
        .collect::<Vec<_>>();
    for (i, entity) in starting_faces.iter().enumerate() {
        commands.entity(*entity).insert((
            Plate(i),
            ChangeColour {
                colour: palette.0[i],
            },
            PlateGenFrontier,
        ));
    }
}

fn flood_fill(
    mut commands: Commands,
    palette: Res<PlatePalette>,
    q_faces: Query<(Entity, &FaceNeighbours, &Plate), With<PlateGenFrontier>>,
    q_regions: Query<&Plate>,
) {
    let mut rng = rand::rng();
    // iterate through the faces that are on the frontier
    for (face_entity_id, face_neighbours, region) in q_faces.iter() {
        // choose a random neighbour for that face
        if let Some(neighbour_entity) = face_neighbours.0.choose(&mut rng) {
            // if the chosen neighour has not yet been assigned a region
            if q_regions.get(*neighbour_entity).is_err() {
                // assign it to the current face's region and mark it as on the frontier
                // also mark it to change colour
                commands.entity(*neighbour_entity).insert((
                    *region,
                    ChangeColour {
                        colour: palette.0[region.0],
                    },
                    PlateGenFrontier,
                ));

                // if all neighbours have been assigned a region
                if face_neighbours
                    .0
                    .iter()
                    .all(|&neighbour_entity_id| q_regions.get(neighbour_entity_id).is_ok())
                {
                    // the current face is no longer on the frontier
                    commands.entity(face_entity_id).remove::<PlateGenFrontier>();
                }
            }
        }
    }
}

fn change_face_color(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<(Entity, &MeshMaterial3d<StandardMaterial>, &ChangeColour), With<FaceNeighbours>>,
) {
    for (entity_id, material_handle, colour) in query.iter() {
        if let Some(material) = materials.get_mut(material_handle) {
            material.base_color = colour.colour;
        }
        commands.entity(entity_id).remove::<ChangeColour>();
    }
}

fn gen_colour_palette(n: usize, rng: &mut ThreadRng) -> Vec<Color> {
    (0..n)
        .map(|_| {
            Color::srgb(
                rng.random_range(0.0..1.0),
                rng.random_range(0.0..1.0),
                rng.random_range(0.0..1.0),
            )
        })
        .collect()
}

fn check_if_finished_plates(
    mut state: ResMut<NextState<WorldGenState>>,
    query_unassigned_faces: Query<Entity, (With<FaceNeighbours>, Without<Plate>)>,
) {
    if query_unassigned_faces.iter().len() == 0 {
        state.set(WorldGenState::FinishedPlates);
    }
}

fn assign_continental_plates(
    mut commands: Commands,
    mut state: ResMut<NextState<WorldGenState>>,
    query_faces: Query<(Entity, &Plate), With<FaceNeighbours>>,
) {
    let mut rng = rand::rng();

    let ocean_plates = (0..N_PLATES)
        .collect::<Vec<_>>()
        .choose_multiple(&mut rng, N_PLATES / 3)
        .copied()
        .collect::<Vec<_>>();

    for (entity_id, plate) in query_faces.iter() {
        if ocean_plates.contains(&plate.0) {
            commands.entity(entity_id).insert((
                Land,
                ChangeColour {
                    colour: Color::srgb(0.565, 0.933, 0.565),
                },
            ));
        } else {
            commands.entity(entity_id).insert((
                Sea,
                ChangeColour {
                    colour: Color::srgb(0.0, 0.412, 0.58),
                },
            ));
        }
    }

    state.set(WorldGenState::FinishedContinents);
}

fn handle_finished_plates(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<NextState<WorldGenState>>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        state.set(WorldGenState::GenContinents);
    }
    if keyboard_input.just_pressed(KeyCode::KeyR) {}
}

fn handle_finished_continents(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<NextState<WorldGenState>>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        state.set(WorldGenState::JustChill);
    }
}

fn do_plate_velocities(
    mut commands: Commands,
    query_faces: Query<(Entity, &Face, &Plate)>,
    mut state: ResMut<NextState<WorldGenState>>,
) {
    let plate_rotation_vectors: Vec<Vec3> = (0..N_PLATES)
        .collect::<Vec<_>>()
        .iter()
        .map(|_| random_rotation_vector())
        .collect();

    for (entity_id, face, plate) in query_faces.iter() {
        let face_velocity = plate_rotation_vectors[plate.0].cross(face.centre_pos);
        commands.entity(entity_id).insert(FacePlateVelocity {
            velocity: face_velocity,
        });
    }

    state.set(WorldGenState::JustChill);
}

/// Generates a random angular velocity vector with length <= 1
fn random_rotation_vector() -> Vec3 {
    let mut rng = rand::rng();

    // Random unit direction
    let dir = random_unit_vector(&mut rng);

    // Random speed in [0.0, 1.0]
    let speed = rng.random_range(0.0..=1.0);

    dir * speed
}

/// Uniformly samples a random unit vector on the sphere
fn random_unit_vector(rng: &mut impl Rng) -> Vec3 {
    let u: f32 = rng.random_range(-1.0..=1.0);
    let theta: f32 = rng.random_range(0.0..=std::f32::consts::TAU);

    let sqrt_term = (1.0 - u * u).sqrt();
    let x = sqrt_term * theta.cos();
    let y = sqrt_term * theta.sin();
    let z = u;

    Vec3::new(x, y, z)
}

pub struct SpherePlugin;

impl Plugin for SpherePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (create_sphere, seed_flood_fill).chain())
            .add_systems(
                FixedUpdate,
                ((flood_fill, check_if_finished_plates).chain())
                    .run_if(in_state(WorldGenState::GenPlates)),
            )
            .add_systems(
                Update,
                (handle_finished_plates).run_if(in_state(WorldGenState::FinishedPlates)),
            )
            .add_systems(
                FixedUpdate,
                (assign_continental_plates).run_if(in_state(WorldGenState::GenContinents)),
            )
            .add_systems(
                Update,
                (handle_finished_continents).run_if(in_state(WorldGenState::FinishedContinents)),
            )
            .add_systems(
                FixedUpdate,
                (do_plate_velocities).run_if(in_state(WorldGenState::GenPlateVelocities)),
            )
            .add_systems(Update, change_face_color);
    }
}
