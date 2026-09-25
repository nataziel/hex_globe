use crate::{ConstructionError, DualMesh, HalfEdgeId, Relaxation, RelaxationError, TriangleMesh};
use rand::{SeedableRng, rngs::StdRng, seq::SliceRandom};
use std::{fmt, num::NonZero};
use subsphere::prelude::*;
/// Parameters controlling deterministic globe construction, flipping, and relaxation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GlobeConfig {
    pub subdivisions: u32,
    pub flip_seed: u64,
    pub flip_rate: f64,
    pub relaxation: Relaxation,
}
impl Default for GlobeConfig {
    fn default() -> Self {
        Self {
            subdivisions: 60,
            flip_seed: 0x5eed_cafe,
            flip_rate: 0.20,
            relaxation: Relaxation::Lloyd {
                iterations: 30,
                strength: 0.8,
            },
        }
    }
}
impl GlobeConfig {
    #[must_use]
    pub fn with_subdivisions(mut self, value: u32) -> Self {
        self.subdivisions = value;
        self
    }
    #[must_use]
    pub fn with_flip_seed(mut self, value: u64) -> Self {
        self.flip_seed = value;
        self
    }
    #[must_use]
    pub fn with_flip_rate(mut self, value: f64) -> Self {
        self.flip_rate = value;
        self
    }
    #[must_use]
    pub fn with_relaxation(mut self, value: Relaxation) -> Self {
        self.relaxation = value;
        self
    }
}
#[derive(Clone, Debug, PartialEq)]
pub enum GlobeBuildError {
    Construction(ConstructionError),
    InvalidFlipRate(f64),
    Relaxation(RelaxationError),
}
impl fmt::Display for GlobeBuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "could not build globe: {self:?}")
    }
}
impl std::error::Error for GlobeBuildError {}
/// Builds a valence-bounded, optionally relaxed dual globe.
///
/// # Errors
/// Returns an error for an invalid rate, source mesh, or relaxation setting.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss
)] // The rate is validated in 0..=1 and practical meshes are far below f64 precision limits.
pub fn build_globe(config: GlobeConfig) -> Result<DualMesh, GlobeBuildError> {
    if !(0.0..=1.0).contains(&config.flip_rate) {
        return Err(GlobeBuildError::InvalidFlipRate(config.flip_rate));
    }
    let mut mesh = TriangleMesh::fuller_icosphere(config.subdivisions)
        .map_err(GlobeBuildError::Construction)?;
    let mut candidates: Vec<_> = mesh
        .half_edges()
        .iter()
        .enumerate()
        .filter_map(|(i, e)| (i < e.twin.0).then_some(HalfEdgeId(i)))
        .collect();
    let target = (candidates.len() as f64 * config.flip_rate).round() as usize;
    candidates.shuffle(&mut StdRng::seed_from_u64(config.flip_seed));
    let mut accepted = 0;
    for h in candidates {
        if accepted == target {
            break;
        }
        if mesh.flip(h).is_ok() {
            accepted += 1;
        }
    }
    if let Relaxation::Lloyd {
        iterations,
        strength,
    } = config.relaxation
    {
        mesh.relax_on_sphere(iterations, strength)
            .map_err(GlobeBuildError::Relaxation)?;
    }
    Ok(mesh.to_dual())
}
impl TriangleMesh {
    pub(crate) fn fuller_icosphere(segments: u32) -> Result<Self, ConstructionError> {
        let sphere = subsphere::icosphere()
            .subdivide_edge(NonZero::new(segments).ok_or(ConstructionError::InvalidSubdivision)?)
            .with_projector(subsphere::proj::Fuller);
        let mut p = vec![[0.; 3]; sphere.num_vertices()];
        for v in sphere.vertices() {
            p[v.index()] = v.pos();
        }
        let t = sphere
            .faces()
            .map(|f| {
                let v: Vec<_> = f.vertices().map(|v| v.index()).collect();
                [v[0], v[1], v[2]]
            })
            .collect();
        Self::from_triangles(p, t)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn builder_is_deterministic_and_bounded() {
        let config = GlobeConfig::default()
            .with_subdivisions(3)
            .with_flip_seed(7)
            .with_relaxation(Relaxation::None);
        let a = build_globe(config).unwrap();
        let b = build_globe(config).unwrap();
        assert_eq!(a.faces().len(), b.faces().len());
        assert!(
            a.faces()
                .iter()
                .all(|f| (5..=7).contains(&f.vertices.len()))
        );
    }
    #[test]
    fn rejects_invalid_flip_rate() {
        assert!(matches!(
            build_globe(GlobeConfig::default().with_flip_rate(1.1)),
            Err(GlobeBuildError::InvalidFlipRate(_))
        ));
    }
}
