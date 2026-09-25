use std::{collections::HashMap, fmt};
/// Identifies a primal mesh vertex.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VertexId(pub usize);
/// Identifies one directed half-edge.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HalfEdgeId(pub usize);
/// Identifies a triangular primal face.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FaceId(pub usize);
/// A primal vertex with a spherical position and an outgoing half-edge representative.
#[derive(Clone, Debug)]
pub struct Vertex {
    pub position: [f64; 3],
    pub half_edge: HalfEdgeId,
}
/// A directed edge record linking its origin, twin, incident face, and face cycle.
#[derive(Clone, Debug)]
pub struct HalfEdge {
    pub origin: VertexId,
    pub twin: HalfEdgeId,
    pub face: FaceId,
    pub next: HalfEdgeId,
    pub previous: HalfEdgeId,
}
/// A triangular primal face represented by one boundary half-edge.
#[derive(Clone, Debug)]
pub struct Face {
    pub half_edge: HalfEdgeId,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConstructionError {
    Empty,
    InvalidSubdivision,
    InvalidVertex(VertexId),
    RepeatedVertex(usize),
    DuplicateDirectedEdge(usize),
    BoundaryEdge,
    InconsistentWinding,
}
impl fmt::Display for ConstructionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid triangle mesh: {self:?}")
    }
}
impl std::error::Error for ConstructionError {}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FlipError {
    ValenceOutOfRange,
    Degenerate,
}
impl fmt::Display for FlipError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "cannot flip edge: {self:?}")
    }
}
impl std::error::Error for FlipError {}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ValidationError {
    InvalidReference,
    TwinMismatch,
    FaceCycle,
    VertexRing,
    EulerCharacteristic,
}
impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid half-edge mesh: {self:?}")
    }
}
impl std::error::Error for ValidationError {}
/// A closed mutable triangular manifold stored as dense half-edge arrays.
#[derive(Clone, Debug)]
pub struct TriangleMesh {
    pub(crate) vertices: Vec<Vertex>,
    pub(crate) half_edges: Vec<HalfEdge>,
    pub(crate) faces: Vec<Face>,
}
impl TriangleMesh {
    /// Constructs a closed, consistently wound triangle mesh.
    ///
    /// # Errors
    /// Returns an error when the input is malformed or not a closed manifold.
    pub fn from_triangles(
        positions: Vec<[f64; 3]>,
        triangles: Vec<[usize; 3]>,
    ) -> Result<Self, ConstructionError> {
        if positions.is_empty() || triangles.is_empty() {
            return Err(ConstructionError::Empty);
        }
        let mut vertices: Vec<_> = positions
            .into_iter()
            .map(|position| Vertex {
                position,
                half_edge: HalfEdgeId(usize::MAX),
            })
            .collect();
        let mut half_edges = Vec::new();
        let mut faces = Vec::new();
        let mut edges = HashMap::new();
        for (fi, t) in triangles.into_iter().enumerate() {
            if let Some(&v) = t.iter().find(|&&v| v >= vertices.len()) {
                return Err(ConstructionError::InvalidVertex(VertexId(v)));
            }
            if t[0] == t[1] || t[1] == t[2] || t[2] == t[0] {
                return Err(ConstructionError::RepeatedVertex(fi));
            }
            let ids = [
                HalfEdgeId(half_edges.len()),
                HalfEdgeId(half_edges.len() + 1),
                HalfEdgeId(half_edges.len() + 2),
            ];
            for i in 0..3 {
                let (a, b) = (t[i], t[(i + 1) % 3]);
                if edges.insert((a, b), ids[i]).is_some() {
                    return Err(ConstructionError::DuplicateDirectedEdge(fi));
                }
                half_edges.push(HalfEdge {
                    origin: VertexId(a),
                    twin: HalfEdgeId(usize::MAX),
                    face: FaceId(fi),
                    next: ids[(i + 1) % 3],
                    previous: ids[(i + 2) % 3],
                });
                if vertices[a].half_edge.0 == usize::MAX {
                    vertices[a].half_edge = ids[i];
                }
            }
            faces.push(Face { half_edge: ids[0] });
        }
        for i in 0..half_edges.len() {
            let a = half_edges[i].origin.0;
            let b = half_edges[half_edges[i].next.0].origin.0;
            half_edges[i].twin = *edges.get(&(b, a)).ok_or(ConstructionError::BoundaryEdge)?;
        }
        let mesh = Self {
            vertices,
            half_edges,
            faces,
        };
        mesh.validate()
            .map_err(|_| ConstructionError::InconsistentWinding)?;
        Ok(mesh)
    }
    #[must_use]
    pub fn vertices(&self) -> &[Vertex] {
        &self.vertices
    }
    #[must_use]
    pub fn half_edges(&self) -> &[HalfEdge] {
        &self.half_edges
    }
    #[must_use]
    pub fn faces(&self) -> &[Face] {
        &self.faces
    }
    #[must_use]
    pub fn half_edge(&self, id: HalfEdgeId) -> &HalfEdge {
        &self.half_edges[id.0]
    }
    #[must_use]
    pub fn vertex_position(&self, id: VertexId) -> [f64; 3] {
        self.vertices[id.0].position
    }
    pub fn set_vertex_position(&mut self, id: VertexId, p: [f64; 3]) {
        self.vertices[id.0].position = p;
    }
    #[must_use]
    pub fn half_edge_destination(&self, id: HalfEdgeId) -> VertexId {
        self.half_edges[self.half_edges[id.0].next.0].origin
    }
    #[must_use]
    pub fn face_half_edges(&self, f: FaceId) -> [HalfEdgeId; 3] {
        let a = self.faces[f.0].half_edge;
        let b = self.half_edges[a.0].next;
        [a, b, self.half_edges[b.0].next]
    }
    #[must_use]
    pub fn vertex_outgoing_ring(&self, v: VertexId) -> Vec<HalfEdgeId> {
        let s = self.vertices[v.0].half_edge;
        let mut r = vec![s];
        let mut h = self.half_edges[self.half_edges[s.0].previous.0].twin;
        while h != s {
            r.push(h);
            h = self.half_edges[self.half_edges[h.0].previous.0].twin;
        }
        r
    }
    #[must_use]
    pub fn vertex_degree(&self, v: VertexId) -> usize {
        self.vertex_outgoing_ring(v).len()
    }
    /// Flips a valence-eligible interior edge.
    ///
    /// # Errors
    /// Returns an error when the flip would violate the 5–7 valence bound.
    #[allow(clippy::many_single_char_names)] // Local edge labels mirror the half-edge flip diagram.
    pub fn flip(&mut self, h: HalfEdgeId) -> Result<(), FlipError> {
        let t = self.half_edges[h.0].twin;
        let (h2, h3, h4, h5) = (
            self.half_edges[h.0].next,
            self.half_edges[h.0].previous,
            self.half_edges[t.0].next,
            self.half_edges[t.0].previous,
        );
        let (a, b, c, d) = (
            self.half_edges[h.0].origin,
            self.half_edges[t.0].origin,
            self.half_edges[h3.0].origin,
            self.half_edges[h5.0].origin,
        );
        if a == b || c == d || a == c || a == d || b == c || b == d {
            return Err(FlipError::Degenerate);
        }
        if self.vertex_degree(a) <= 5
            || self.vertex_degree(b) <= 5
            || self.vertex_degree(c) >= 7
            || self.vertex_degree(d) >= 7
        {
            return Err(FlipError::ValenceOutOfRange);
        }
        let (f0, f1) = (self.half_edges[h.0].face, self.half_edges[t.0].face);
        if self.vertices[a.0].half_edge == h {
            self.vertices[a.0].half_edge = h4;
        }
        if self.vertices[b.0].half_edge == t {
            self.vertices[b.0].half_edge = h2;
        }
        self.half_edges[h.0] = HalfEdge {
            origin: c,
            twin: t,
            face: f0,
            next: h5,
            previous: h2,
        };
        self.half_edges[t.0] = HalfEdge {
            origin: d,
            twin: h,
            face: f1,
            next: h3,
            previous: h4,
        };
        self.half_edges[h2.0].next = h;
        self.half_edges[h2.0].previous = h5;
        self.half_edges[h5.0].next = h2;
        self.half_edges[h5.0].previous = h;
        self.half_edges[h5.0].face = f0;
        self.half_edges[h3.0].next = h4;
        self.half_edges[h3.0].previous = t;
        self.half_edges[h3.0].face = f1;
        self.half_edges[h4.0].next = t;
        self.half_edges[h4.0].previous = h3;
        self.half_edges[h4.0].face = f1;
        self.faces[f0.0].half_edge = h;
        self.faces[f1.0].half_edge = t;
        Ok(())
    }
    /// Checks closed-manifold half-edge invariants.
    ///
    /// # Errors
    /// Returns the first invariant violation found.
    pub fn validate(&self) -> Result<(), ValidationError> {
        for (i, h) in self.half_edges.iter().enumerate() {
            if h.origin.0 >= self.vertices.len()
                || h.twin.0 >= self.half_edges.len()
                || h.next.0 >= self.half_edges.len()
                || h.previous.0 >= self.half_edges.len()
            {
                return Err(ValidationError::InvalidReference);
            }
            if self.half_edges[h.twin.0].twin != HalfEdgeId(i) {
                return Err(ValidationError::TwinMismatch);
            }
        }
        for (i, _f) in self.faces.iter().enumerate() {
            let c = self.face_half_edges(FaceId(i));
            if self.half_edges[c[2].0].next != c[0]
                || c.iter().any(|h| self.half_edges[h.0].face != FaceId(i))
            {
                return Err(ValidationError::FaceCycle);
            }
        }
        if self.vertices.len().saturating_add(self.faces.len()) != self.half_edges.len() / 2 + 2 {
            return Err(ValidationError::EulerCharacteristic);
        }
        Ok(())
    }
}
