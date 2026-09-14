//! A container for sets of modes for the GMT segments

#[cfg(feature = "delaunay")]
mod delaunay;
mod filing;

use std::{collections::HashMap, fmt::Display, marker::PhantomData};

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

/// Interpolation method
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum InterpolationMethod {
    Barycentric,
    #[default]
    NaturalNeighbor,
    NaturalNeighborWithGradients,
}

/// A set of modes
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Set {
    /// modes `x,y` mesh vertices
    pub xy: Option<Vec<[f64; 2]>>,
    /// modes
    pub data: Vec<Vec<f64>>,
}
impl Set {
    /// Creates a new [Set] object from a vector of modes
    pub fn new(data: impl Into<Vec<Vec<f64>>>) -> Self {
        Self {
            data: data.into(),
            ..Default::default()
        }
    }
    /// Sets the `x,y` coordinates of the mesh where the modes are defined
    pub fn xy(mut self, xy: impl IntoIterator<Item = [f64; 2]>) -> Self {
        self.xy = Some(xy.into_iter().collect());
        self
    }
    /// Returns the number of modes
    pub fn len(&self) -> usize {
        self.data.len()
    }
    /// Checks if the set is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
    /// Pushes a mode into the set
    pub fn push(&mut self, value: impl Into<Vec<f64>>) {
        self.data.push(value.into());
    }
    /// Returns a borrowing iterator over the modes
    pub fn iter(&self) -> impl Iterator<Item = &[f64]> {
        self.data.iter().map(|x| x.as_slice())
    }
    /// Returns a consuming iterator over the modes
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

#[derive(Default)]
pub struct Native {}
#[derive(Default)]
pub struct Regular {}
pub trait Mesh: Default {}
impl Mesh for Native {}
impl Mesh for Regular {}

/// Sets of mirror modes
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ModesSets<M: Mesh> {
    pub n_sample: usize,
    pub width: f64,
    pub sets: HashMap<usize, Set>,
    pub segment2set: [i32; 7],
    interpolation_method: Option<InterpolationMethod>,
    mesh: PhantomData<M>,
}
impl<M: Mesh> Display for ModesSets<M> {
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
impl<M: Mesh> ModesSets<M> {
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
    /// Inserts a set into the collection
    ///
    /// `idx` is the set index, returns an error if it is not found into `segment2set`
    pub fn insert(&mut self, idx: usize, set: impl Into<Set>) -> ModesSetsResult<&mut Self> {
        self.check_set_index(idx)?;
        let _ = self.sets.insert(idx, set.into());
        Ok(self)
    }
    /// Returns the number of modes in each set in ascending order
    pub fn n_mode(&self) -> ModesSetsResult<Vec<usize>> {
        let mut idxs: Vec<_> = self.sets.keys().collect();
        idxs.sort();
        let mut n_mode = vec![];
        for i in idxs {
            n_mode.push(self.get(*i)?.len());
        }
        Ok(n_mode)
    }
    /// Returns the largest number of modes off all sets
    pub fn max_n_mode(&self) -> ModesSetsResult<Option<usize>> {
        Ok(self
            .n_mode()?
            .iter()
            .max_by(|x, y| x.partial_cmp(y).unwrap())
            .map(|n| *n))
    }
}
