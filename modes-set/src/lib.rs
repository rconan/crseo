//! A container for sets of modes for the GMT segments

#[cfg(feature = "delaunay")]
mod delaunay;

use std::{collections::HashMap, fmt::Display};

#[derive(Debug, thiserror::Error)]
pub enum ModesSetError {
    #[error("index {0} is missing in `segment2set`")]
    MissingSet(usize),
    #[error("index {0} is empty")]
    EmptySet(usize),
    #[cfg(feature = "delaunay")]
    #[error("Delaunay triangulation failed")]
    Delaunay(#[from] delaunay::TriangulationError),
}
pub(crate) type ModesSetsResult<T> = Result<T, ModesSetError>;

#[derive(Debug, Default, Clone)]
pub enum InterpolationMethod {
    Barycentric,
    #[default]
    NaturalNeighbor,
    NaturalNeighborWithGradients,
}

/// Sets of mirror modes
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ModesSets {
    pub n_sample: usize,
    pub width: f64,
    pub sets: HashMap<usize, Vec<Vec<f64>>>,
    pub segment2set: [i32; 7],
    interpolation_method: Option<InterpolationMethod>,
}
impl Display for ModesSets {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "GMT Modes Sets (N={}px, L={}m): {:?} -> {:?}",
            self.n_sample,
            self.width,
            self.n_mode(),
            self.segment2set
        )
    }
}
impl ModesSets {
    /// Creates a new [ModesSets] object
    ///
    /// A mode is sampled on a `n_sample X n_sample` regular grid of length `width` in meters.
    /// A set of modes consists of `n_mode` matched to one or several GMT segments.
    /// The 7 elements array `segment2set` assigns a set index to each segment.
    pub fn new(n_sample: usize, width: f64, segment2set: impl Into<[i32; 7]>) -> Self {
        Self {
            n_sample,
            width,
            segment2set: segment2set.into(),
            ..Default::default()
        }
    }
    /// Uses the barycentric interpolation method to interpolate the mode on a regular grid
    pub fn barycentric_interpolation(mut self) -> Self {
        self.interpolation_method = Some(InterpolationMethod::Barycentric);
        self
    }
    /// Uses the natural neighbor interpolation method to interpolate the mode on a regular grid
    ///
    /// This is the default interpolation method if not set explicitely
    pub fn natural_neighbor_interpolation(mut self) -> Self {
        self.interpolation_method = Some(InterpolationMethod::NaturalNeighbor);
        self
    }
    /// Uses the natural neighbor interpolation method with gradients estimation to interpolate the mode on a regular grid
    pub fn natural_neighbor_interpolation_with_gradients(mut self) -> Self {
        self.interpolation_method = Some(InterpolationMethod::NaturalNeighborWithGradients);
        self
    }
    fn check_set_index(&self, idx: usize) -> ModesSetsResult<()> {
        self.segment2set
            .contains(&(idx as i32))
            .ok_or(ModesSetError::MissingSet(idx))
    }
    /// Inserts a mode into a particular set
    ///
    /// `idx` is the set index, returns an error if it is not found into `segment2set`
    pub fn insert(&mut self, idx: usize, mode: impl Into<Vec<f64>>) -> ModesSetsResult<&mut Self> {
        self.check_set_index(idx)?;
        self.sets.entry(idx).or_insert(vec![]).push(mode.into());
        Ok(self)
    }
    /// Inserts several modes into a particular set
    ///
    /// `idx` is the set index, returns an error if it is not found into `segment2set`
    pub fn inserts<T: Into<Vec<f64>>>(
        &mut self,
        idx: usize,
        modes: impl Iterator<Item = T>,
    ) -> ModesSetsResult<&mut Self> {
        self.check_set_index(idx)?;
        for mode in modes {
            self.sets.entry(idx).or_insert(vec![]).push(mode.into());
        }
        Ok(self)
    }
    /// Returns the number of modes in each set in ascending order
    pub fn n_mode(&self) -> Vec<usize> {
        let mut idxs: Vec<_> = self.sets.keys().collect();
        idxs.sort();
        let mut n_mode = vec![];
        for i in idxs {
            n_mode.push(self.sets.get(&i).map_or_else(|| 0, |modes| modes.len()));
        }
        n_mode
    }
    /// Returns the largest number of modes off all sets
    pub fn max_n_mode(&self) -> Option<usize> {
        self.n_mode()
            .iter()
            .max_by(|x, y| x.partial_cmp(y).unwrap())
            .map(|n| *n)
    }
}
