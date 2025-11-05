use bevy::prelude::*;
use rand::{Rng, seq::IndexedRandom};

use crate::setup::{ChangeColour, Face, FaceNeighbours, N_PLATES, PlatePalette};
use crate::states::{GameState, WorldGenState};

#[derive(Component, Clone, Copy, PartialEq)]
pub struct Plate(pub usize);

#[derive(Component)]
pub struct PlateBoundary;

#[derive(Component)]
pub struct PlateGenFrontier;

#[derive(Component)]
pub struct Land;

#[derive(Component)]
pub struct Sea;

#[derive(Event)]
struct ResetContinents;

#[derive(Component)]
pub struct FacePlateVelocity {
    pub velocity: Vec3,
}
/// Select starting faces for flood fill
fn seed_flood_fill(
    mut commands: Commands,
    face_query: Query<Entity, With<Face>>,
    palette: Res<PlatePalette>,
    mut gen_state: ResMut<NextState<WorldGenState>>,
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
            ChangeColour { colour: palette[i] },
            PlateGenFrontier,
        ));
    }

    gen_state.set(WorldGenState::GenPlates);
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
        if let Some(neighbour_entity) = face_neighbours.choose(&mut rng) {
            // if the chosen neighour has not yet been assigned a region
            if q_regions.get(*neighbour_entity).is_err() {
                // assign it to the current face's region and mark it as on the frontier
                // also mark it to change colour
                commands.entity(*neighbour_entity).insert((
                    *region,
                    ChangeColour {
                        colour: palette[region.0],
                    },
                    PlateGenFrontier,
                ));

                // if all neighbours have been assigned a region
                if face_neighbours
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

fn check_if_finished_plates(
    mut state: ResMut<NextState<WorldGenState>>,
    query_unassigned_faces: Query<Entity, (With<Face>, Without<Plate>)>,
) {
    if query_unassigned_faces.iter().len() == 0 {
        state.set(WorldGenState::FinishedPlates);
    }
}

fn assign_plate_boundaries(
    q_faces: Query<(Entity, &FaceNeighbours, &Plate)>,
    q_regions: Query<&Plate>,
    mut commands: Commands,
    mut state: ResMut<NextState<WorldGenState>>,
) {
    for (face_entity_id, face_neighbours, plate) in q_faces.iter() {
        // if any neighbours are from a different region
        if face_neighbours.iter().any(|&neighbour_entity_id| {
            match q_regions.get(neighbour_entity_id) {
                Ok(neighbour_plate) => neighbour_plate != plate,
                Err(_) => unreachable!(), // this should be impossible, at this point every face should be assigned a plate
            }
        }) {
            // this face is on a plate boundary
            commands.entity(face_entity_id).insert((
                PlateBoundary,
                ChangeColour {
                    colour: Color::BLACK,
                },
            ));
        }
    }

    state.set(WorldGenState::FinishedPlateBoundaries)
}

fn assign_continental_plates(
    mut commands: Commands,
    mut state: ResMut<NextState<WorldGenState>>,
    query_faces: Query<(Entity, &Plate), With<Face>>,
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
        state.set(WorldGenState::AssignPlateBoundaries);
    }
    if keyboard_input.just_pressed(KeyCode::KeyR) {
        // remove the colour from every face
        // remove the `Plate` component from every face
        // seed the flood fill
        // change state to GenPlates
    }
}

fn handle_finished_plate_boundaries(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<NextState<WorldGenState>>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        state.set(WorldGenState::GenContinents);
    }
}

fn handle_finished_continents(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<NextState<WorldGenState>>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        state.set(WorldGenState::GenPlateVelocities);
    }
    if keyboard_input.just_pressed(KeyCode::KeyR) {
        commands.trigger(ResetContinents);
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

fn handle_just_chill(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut game_state: ResMut<NextState<GameState>>,
    mut gen_state: ResMut<NextState<WorldGenState>>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        gen_state.set(WorldGenState::Finished);
        game_state.set(GameState::Simulation);
    }
}

fn reset_continents(
    _: On<ResetContinents>,
    mut commands: Commands,
    mut state: ResMut<NextState<WorldGenState>>,
    query_faces: Query<Entity, Or<(With<Land>, With<Sea>)>>,
) {
    for entity in query_faces.iter() {
        commands.entity(entity).remove::<(Land, Sea)>();
    }
    state.set(WorldGenState::GenContinents);
}

pub struct WorldGenPlugin;

impl Plugin for WorldGenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            seed_flood_fill.run_if(in_state(WorldGenState::SeedPlates)),
        )
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
            (assign_plate_boundaries).run_if(in_state(WorldGenState::AssignPlateBoundaries)),
        )
        .add_systems(
            Update,
            (handle_finished_plate_boundaries)
                .run_if(in_state(WorldGenState::FinishedPlateBoundaries)),
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
        .add_systems(
            Update,
            (handle_just_chill).run_if(in_state(WorldGenState::JustChill)),
        )
        .add_observer(reset_continents);
    }
}
