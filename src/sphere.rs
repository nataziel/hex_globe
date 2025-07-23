use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::PrimitiveTopology;
use rand::Rng;
use rand::seq::SliceRandom;
use std::num::NonZero;
use subsphere::prelude::*;

#[derive(Component)]
pub struct Face {
    pub neighbors: Vec<Entity>,
}

pub fn create_sphere(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let sphere = subsphere::HexSphere::from_kis(
        subsphere::icosphere()
            .subdivide_edge(NonZero::new(9).unwrap())
            .with_projector(subsphere::proj::Fuller),
    )
    .unwrap();

    let mut rng = rand::thread_rng();
    let mut face_entities = Vec::new();

    // First pass: create entities and store them
    for _ in 0..sphere.num_faces() {
        let entity = commands.spawn_empty().id();
        face_entities.push(entity);
    }

    // Second pass: populate entities
    for (i, face) in sphere.faces().enumerate() {
        let color = Color::srgb(rng.r#gen(), rng.r#gen(), rng.r#gen());

        let positions = build_fan_triangulation(face);

        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        );
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.compute_flat_normals();

        let mut neighbors = Vec::new();
        for side in face.sides() {
            let neighbor_index = side.twin().inside().index();
            neighbors.push(face_entities[neighbor_index]);
        }

        commands.entity(face_entities[i]).insert((
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: color,
                ..default()
            })),
            Face { neighbors },
            Transform::from_xyz(2.0, 2.0, 0.0),
        ));
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

pub fn change_face_color(
    _time: Res<Time>,
    query: Query<&MeshMaterial3d<StandardMaterial>, With<Face>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {

    let mut rng = rand::thread_rng();
    if let Some(material_handle) = query.iter().collect::<Vec<_>>().choose(&mut rng) {
        if let Some(material) = materials.get_mut(*material_handle) {
            material.base_color = Color::srgb(rng.r#gen(), rng.r#gen(), rng.r#gen());
        }
    }
}
