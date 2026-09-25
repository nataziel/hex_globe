use crate::{FaceId, TriangleMesh, VertexId, dual::normalize};
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Relaxation {
    None,
    Lloyd { iterations: usize, strength: f64 },
}

#[derive(Clone, Debug, PartialEq)]
pub enum RelaxationError {
    InvalidStrength(f64),
}

impl fmt::Display for RelaxationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid relaxation: {self:?}")
    }
}

impl std::error::Error for RelaxationError {}

impl TriangleMesh {
    /// Performs snapshot-based spherical Lloyd relaxation.
    ///
    /// # Errors
    /// Returns an error when `strength` is outside 0..=1.
    #[allow(clippy::cast_precision_loss)] // Vertex rings remain small in this triangular globe.
    pub fn relax_on_sphere(
        &mut self,
        iterations: usize,
        strength: f64,
    ) -> Result<(), RelaxationError> {
        if !(0.0..=1.0).contains(&strength) {
            return Err(RelaxationError::InvalidStrength(strength));
        }
        for _ in 0..iterations {
            let snapshot: Vec<_> = self.vertices.iter().map(|v| v.position).collect();
            let dual_vertices: Vec<_> = (0..self.faces.len())
                .map(|i| {
                    normalize(self.face_half_edges(FaceId(i)).into_iter().fold(
                        [0.; 3],
                        |sum, h| {
                            let p = snapshot[self.half_edge(h).origin.0];
                            [sum[0] + p[0], sum[1] + p[1], sum[2] + p[2]]
                        },
                    ))
                })
                .collect();
            let next: Vec<_> = (0..self.vertices.len())
                .map(|i| {
                    let ring = self.vertex_outgoing_ring(VertexId(i));
                    let centre = snapshot[i];
                    let dual_face: Vec<_> = ring
                        .iter()
                        .map(|&edge| dual_vertices[self.half_edge(edge).face.0])
                        .collect();
                    let weighted_centroid = dual_face
                        .iter()
                        .copied()
                        .zip(dual_face.iter().copied().cycle().skip(1))
                        .take(dual_face.len())
                        .fold([0.0; 3], |sum, (first, second)| {
                            let area = spherical_triangle_area(centre, first, second);
                            let triangle_centroid = normalize(add(add(centre, first), second));
                            add(sum, scale(triangle_centroid, area))
                        });
                    let centroid = normalize(weighted_centroid);
                    normalize(add(
                        scale(centre, 1.0 - strength),
                        scale(centroid, strength),
                    ))
                })
                .collect();
            for (vertex, position) in self.vertices.iter_mut().zip(next) {
                vertex.position = position;
            }
        }
        Ok(())
    }
}

fn add(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn scale(p: [f64; 3], factor: f64) -> [f64; 3] {
    [p[0] * factor, p[1] * factor, p[2] * factor]
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn determinant(a: [f64; 3], b: [f64; 3], c: [f64; 3]) -> f64 {
    dot(
        a,
        [
            b[1] * c[2] - b[2] * c[1],
            b[2] * c[0] - b[0] * c[2],
            b[0] * c[1] - b[1] * c[0],
        ],
    )
}

fn spherical_triangle_area(a: [f64; 3], b: [f64; 3], c: [f64; 3]) -> f64 {
    2.0 * determinant(a, b, c)
        .abs()
        .atan2(1.0 + dot(a, b) + dot(b, c) + dot(c, a))
}
