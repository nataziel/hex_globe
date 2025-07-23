use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::PrimitiveTopology;
use rand::Rng;
use rand::seq::SliceRandom;
use std::num::NonZero;
use subsphere::prelude::*;

#[derive(Component)]
pub struct Face;

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

    for face in sphere.faces() {
        let face_vertices: Vec<_> = face.vertices().map(|v| v.pos()).collect();
        let color = Color::srgb(rng.r#gen(), rng.r#gen(), rng.r#gen());

        let mut positions = Vec::new();
        let v0 = face_vertices[0];

        for i in 1..(face_vertices.len() - 1) {
            let v1 = face_vertices[i];
            let v2 = face_vertices[i + 1];

            positions.push([v0[0] as f32, v0[1] as f32, v0[2] as f32]);
            positions.push([v1[0] as f32, v1[1] as f32, v1[2] as f32]);
            positions.push([v2[0] as f32, v2[1] as f32, v2[2] as f32]);
        }

        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        );
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.compute_flat_normals();

        commands.spawn((
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: color,
                ..default()
            })),
            Face,
            Transform::from_xyz(2.0, 2.0, 0.0),
        ));
    }
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
