use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::PrimitiveTopology;
use rand::{Rng, rngs::ThreadRng};
use std::num::NonZero;
use subsphere::prelude::*;

pub const N_PLATES: usize = 40;

#[derive(Resource, Deref)]
pub struct PlatePalette(Vec<Color>);

#[derive(Component)]
pub struct Face {
    pub centre_pos: Vec3,
}

#[derive(Component, Deref)]
pub struct FaceNeighbours(Vec<Entity>);

#[derive(Component)]
pub struct ChangeColour {
    pub colour: Color,
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
}

// Create plates colour palette
fn create_palette(mut commands: Commands) {
    let mut rng = rand::rng();
    let colour_palette = gen_colour_palette(N_PLATES, &mut rng);
    commands.insert_resource(PlatePalette(colour_palette));
}

// TODO: merge this into create_palette?
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

fn change_face_color(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<(Entity, &MeshMaterial3d<StandardMaterial>, &ChangeColour), With<Face>>,
) {
    for (entity_id, material_handle, colour) in query.iter() {
        if let Some(material) = materials.get_mut(material_handle) {
            material.base_color = colour.colour;
        }
        commands.entity(entity_id).remove::<ChangeColour>();
    }
}

pub struct SetupPlugin;

impl Plugin for SetupPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (create_sphere, create_palette))
            .add_systems(Update, change_face_color);
    }
}
