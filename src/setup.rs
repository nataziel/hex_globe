use bevy::asset::RenderAssetUsages;
use bevy::math::prelude::*;
use bevy::prelude::*;
use bevy::render::render_resource::PrimitiveTopology;
use noiz::prelude::*;
use rand::{RngExt, rngs::ThreadRng};

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

fn globe_config() -> hex_globe_halfedge::GlobeConfig {
    hex_globe_halfedge::GlobeConfig::default()
        .with_subdivisions(30)
        .with_flip_seed(0x5eed_cafe)
        .with_flip_rate(0.05)
        .with_relaxation(hex_globe_halfedge::Relaxation::Lloyd {
            iterations: 30,
            strength: 0.8,
        })
}
fn create_sphere(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let dual = hex_globe_halfedge::build_globe(globe_config()).expect("valid globe configuration");

    let mut noise = Noise::from(LayeredNoise::new(
        NormedByDerivative::<f32, EuclideanLength, PeakDerivativeContribution>::default()
            .with_falloff(0.3),
        Persistence(0.6),
        FractalLayers {
            layer: Octave::<MixCellGradients<OrthoGrid, Smoothstep, QuickGradients, true>>::default(
            ),
            lacunarity: 1.8,
            amount: 8,
        },
    ));
    noise.set_period(0.001);
    let face_entities: Vec<_> = (0..dual.faces().len())
        .map(|_| commands.spawn_empty().id())
        .collect();
    // One outline material avoids exceeding renderer material-index limits at high subdivisions.
    let outline_material = materials.add(StandardMaterial {
        base_color: Color::BLACK,
        ..default()
    });
    for (i, face) in dual.faces().iter().enumerate() {
        let face_positions: Vec<Vec3> = face
            .vertices
            .iter()
            .map(|&vertex| vec3(dual.vertices()[vertex.0].position))
            .collect();
        let positions = build_fan_triangulation(&face_positions);
        let mut render_mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        );
        render_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        render_mesh.compute_flat_normals();
        let neighbours = face.neighbours.iter().map(|n| face_entities[n.0]).collect();
        commands.entity(face_entities[i]).insert((
            Mesh3d(meshes.add(render_mesh)),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::WHITE,
                ..default()
            })),
            Face {
                centre_pos: vec3(face.centre).normalize(),
            },
            FaceNeighbours(neighbours),
            Transform::default(),
        ));
        commands.entity(face_entities[i]).with_children(|c| {
            c.spawn((
                Mesh3d(
                    meshes.add(build_outline_mesh(
                        &face
                            .vertices
                            .iter()
                            .map(|&vertex| vec3(dual.vertices()[vertex.0].position))
                            .collect::<Vec<_>>(),
                    )),
                ),
                MeshMaterial3d(outline_material.clone()),
            ));
        });
        let height: f32 = noise.sample(vec3(face.centre));
        debug!(height, "dual face height");
    }
}

fn vec3(p: [f64; 3]) -> Vec3 {
    Vec3::new(p[0] as f32, p[1] as f32, p[2] as f32)
}

fn build_fan_triangulation(vertices: &[Vec3]) -> Vec<[f32; 3]> {
    let mut positions = Vec::with_capacity((vertices.len() - 2) * 3);
    for pair in vertices[1..].windows(2) {
        positions.extend([
            vertices[0].to_array(),
            pair[0].to_array(),
            pair[1].to_array(),
        ]);
    }
    positions
}

fn build_outline_mesh(vertices: &[Vec3]) -> Mesh {
    let mut positions: Vec<_> = vertices.iter().map(|p| *p * 1.0001).collect();
    positions.push(positions[0]);
    let mut mesh = Mesh::new(PrimitiveTopology::LineStrip, RenderAssetUsages::default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh
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

fn change_face_color(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<(Entity, &MeshMaterial3d<StandardMaterial>, &ChangeColour), With<Face>>,
) {
    for (entity_id, material_handle, colour) in query.iter() {
        if let Some(mut material) = materials.get_mut(material_handle) {
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fixed_seed_builds_a_deterministic_valence_bounded_dual() {
        let first = hex_globe_halfedge::build_globe(globe_config()).unwrap();
        let second = hex_globe_halfedge::build_globe(globe_config()).unwrap();
        let signature = |dual: &hex_globe_halfedge::DualMesh| {
            dual.faces()
                .iter()
                .map(|face| (face.vertices.clone(), face.neighbours.clone()))
                .collect::<Vec<_>>()
        };
        assert_eq!(signature(&first), signature(&second));
        assert!(
            first
                .faces()
                .iter()
                .all(|face| (5..=7).contains(&face.vertices.len())
                    && face.vertices.len() == face.neighbours.len())
        );
        for (i, face) in first.faces().iter().enumerate() {
            for neighbour in &face.neighbours {
                assert!(
                    first
                        .face(*neighbour)
                        .neighbours
                        .contains(&hex_globe_halfedge::DualFaceId(i))
                );
            }
        }
    }
}
