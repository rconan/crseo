use std::{
    error::Error,
    ops::{Add, Deref, DerefMut},
};

use nalgebra::DMatrix;
use serde::Serialize;

use crate::wavefrontsensor::SlopesArray;

/// A collection of [SlopesArray]
#[derive(Clone, Default, Debug, Serialize)]
pub struct Calibration(Vec<SlopesArray>);
impl Deref for Calibration {
    type Target = Vec<SlopesArray>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for Calibration {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl Calibration {
    /// Returns the number of rows and columns of the calibration matrix
    pub fn shape(&self) -> (usize, usize) {
        (self.nrows(), self.ncols())
    }
    /// Returns the number of rows of the calibration matrix
    pub fn nrows(&self) -> usize {
        self.iter().map(|x| x.nrows()).sum()
    }
    /// Returns the number of columns of the calibration matrix
    pub fn ncols(&self) -> usize {
        self.iter().map(|x| x.ncols()).sum()
    }
    /// Compute the pseudo-inverse of each slope array
    pub fn pseudo_inverse(&mut self) -> Result<&mut Self, Box<dyn Error>> {
        self.iter_mut()
            .map(|x| x.pseudo_inverse())
            .collect::<Result<Vec<_>, Box<dyn Error>>>()?;
        Ok(self)
    }
    /// Concatenates the pseudo-inverse of each slope arrays in a [Vec]
    ///
    /// The matrix are flatten column-wise.
    pub fn concat_pinv(&self) -> Vec<f64> {
        self.iter()
            .filter_map(|x| x.inverse.as_ref().map(|x| x.as_slice().to_vec()))
            .flatten()
            .map(|x| x as f64)
            .collect()
    }

    /// Returns the length of the vector of [SlopesArray]
    pub fn len(&self) -> usize {
        self.0.len()
    }
    /// Returns the interaction matrices 
    pub fn interaction_matrices(&self) -> Vec<DMatrix<f32>> {
        self.iter().map(|s| s.interaction_matrix()).collect()
    }
}

impl Add for Calibration {
    type Output = Calibration;

    fn add(self, rhs: Self) -> Self::Output {
        Calibration(self.0.into_iter().chain(rhs.0.into_iter()).collect())
    }
}
