use crate::{FaceId, TriangleMesh, VertexId};
/// Identifies a dual vertex, corresponding to one primal triangle.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DualVertexId(pub usize);
/// Identifies a dual face, corresponding to one primal vertex.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DualFaceId(pub usize);
/// A dual vertex position on the unit sphere.
#[derive(Clone, Debug)]
pub struct DualVertex {
    pub position: [f64; 3],
}
/// A polygonal dual face with ordered boundary vertices and neighbours.
#[derive(Clone, Debug)]
pub struct DualFace {
    pub centre: [f64; 3],
    pub vertices: Vec<DualVertexId>,
    pub neighbours: Vec<DualFaceId>,
}
/// The final polygonal globe mesh returned by the generator.
#[derive(Clone, Debug)]
pub struct DualMesh {
    vertices: Vec<DualVertex>,
    faces: Vec<DualFace>,
}
impl DualMesh {
    #[must_use]
    pub fn vertices(&self) -> &[DualVertex] {
        &self.vertices
    }
    #[must_use]
    pub fn faces(&self) -> &[DualFace] {
        &self.faces
    }
    #[must_use]
    pub fn face(&self, id: DualFaceId) -> &DualFace {
        &self.faces[id.0]
    }
}
impl TriangleMesh {
    #[must_use]
    pub fn to_dual(&self) -> DualMesh {
        let vertices = (0..self.faces.len())
            .map(|i| {
                let sum = self
                    .face_half_edges(FaceId(i))
                    .into_iter()
                    .fold([0.; 3], |s, h| {
                        let p = self.vertex_position(self.half_edge(h).origin);
                        [s[0] + p[0], s[1] + p[1], s[2] + p[2]]
                    });
                DualVertex {
                    position: normalize(sum),
                }
            })
            .collect();
        let faces = (0..self.vertices.len())
            .map(|i| {
                let v = VertexId(i);
                let ring = self.vertex_outgoing_ring(v);
                DualFace {
                    centre: normalize(self.vertex_position(v)),
                    vertices: ring
                        .iter()
                        .map(|&h| DualVertexId(self.half_edge(h).face.0))
                        .collect(),
                    neighbours: ring
                        .iter()
                        .map(|&h| DualFaceId(self.half_edge_destination(h).0))
                        .collect(),
                }
            })
            .collect();
        DualMesh { vertices, faces }
    }
}
pub(crate) fn normalize(p: [f64; 3]) -> [f64; 3] {
    let n = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt();
    [p[0] / n, p[1] / n, p[2] / n]
}
