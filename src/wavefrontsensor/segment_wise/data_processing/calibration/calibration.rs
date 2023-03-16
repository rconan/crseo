use std::{
    error::Error,
    ops::{Add, Deref, DerefMut},
};

use serde::Serialize;

use crate::wavefrontsensor::SlopesArray;

/// A collection of [SlopesArray]
#[derive(Default, Debug, Serialize)]
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
    /// Returns the length of the vector of [SlopesArray]
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl Add for Calibration {
    type Output = Calibration;

    fn add(self, rhs: Self) -> Self::Output {
        Calibration(self.0.into_iter().chain(rhs.0.into_iter()).collect())
    }
}
