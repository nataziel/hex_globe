#![warn(clippy::pedantic)]
//! Bevy-independent globe topology and dual-mesh generation.

mod dual;
mod generation;
mod relaxation;
mod topology;

pub use dual::{DualFace, DualFaceId, DualMesh, DualVertex, DualVertexId};
pub use generation::{GlobeBuildError, GlobeConfig, build_globe};
pub use relaxation::{Relaxation, RelaxationError};
pub use topology::{
    ConstructionError, Face, FaceId, FlipError, HalfEdge, HalfEdgeId, TriangleMesh,
    ValidationError, Vertex, VertexId,
};
