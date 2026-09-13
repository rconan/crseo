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

#[derive(Debug, Default, Clone)]
pub struct Set {
    xy: Option<Vec<[f64; 2]>>,
    data: Vec<Vec<f64>>,
}
impl Set {
    pub fn new(data: &[Vec<f64>]) -> Self {
        Self {
            data: data.to_vec(),
            ..Default::default()
        }
    }
    pub fn xy(mut self, xy: impl IntoIterator<Item = [f64; 2]>) -> Self {
        self.xy = Some(xy.into_iter().collect());
        self
    }
    pub fn len(&self) -> usize {
        self.data.len()
    }
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
    pub fn push(&mut self, value: impl Into<Vec<f64>>) {
        self.data.push(value.into());
    }
    pub fn iter(&self) -> impl Iterator<Item = &[f64]> {
        self.data.iter().map(|x| x.as_slice())
    }
    pub fn into_iter(self) -> impl Iterator<Item = Vec<f64>> {
        self.data.into_iter()
    }
}

impl<A> FromIterator<A> for Set
where
    Vec<f64>: From<A>,
{
    fn from_iter<T: IntoIterator<Item = A>>(iter: T) -> Self {
        Self {
            data: iter.into_iter().map(|x| x.into()).collect(),
            ..Default::default()
        }
    }
}

impl<T> From<T> for Set
where
    Vec<f64>: From<T>,
{
    fn from(value: T) -> Self {
        Self {
            data: vec![value.into()],
            ..Default::default()
        }
    }
}

/// Sets of mirror modes
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ModesSets {
    pub n_sample: usize,
    pub width: f64,
    pub sets: HashMap<usize, Set>,
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
    pub fn get(&self, idx: usize) -> ModesSetsResult<&Set> {
        self.sets
            .get(&idx)
            .ok_or_else(|| ModesSetError::MissingSet(idx))
    }
    pub fn get_mut(&mut self, idx: usize) -> ModesSetsResult<&mut Set> {
        self.sets
            .get_mut(&idx)
            .ok_or_else(|| ModesSetError::MissingSet(idx))
    }
    /// Inserts a mode into a particular set
    ///
    /// `idx` is the set index, returns an error if it is not found into `segment2set`
    pub fn insert(&mut self, idx: usize, set: impl Into<Set>) -> ModesSetsResult<&mut Self> {
        self.check_set_index(idx)?;
        // self.sets.entry(idx).or_insert(vec![]).push(mode.into());
        let _ = self.sets.insert(idx, set.into());
        Ok(self)
    }
    /* /// Inserts several modes into a particular set
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
    } */
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
