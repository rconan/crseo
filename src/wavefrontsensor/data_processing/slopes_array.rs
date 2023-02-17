use std::{error::Error, ops::Mul};

use serde::Serialize;

use super::{DataRef, Slopes};

type Mat = nalgebra::DMatrix<f32>;

/// A collection of pyramid measurements
#[derive(Debug, Clone, Default, Serialize)]
pub struct SlopesArray {
    pub(crate) slopes: Vec<Slopes>,
    pub(crate) quad_cell: DataRef,
    #[serde(skip)]
    pub(crate) inverse: Option<Mat>,
}
impl From<(DataRef, Vec<Slopes>)> for SlopesArray {
    /// Convert a vector of measurements and the associated [QuadCell] into a [SlopesArray]
    fn from((quad_cell, slopes): (DataRef, Vec<Slopes>)) -> Self {
        Self {
            slopes,
            quad_cell,
            inverse: None,
        }
    }
}
impl SlopesArray {
    /// Creates a new [SlopesArray]
    pub fn new(slopes: Vec<Slopes>) -> Self {
        Self {
            slopes,
            ..Default::default()
        }
    }
    /// Returns the number of rows and columns of the [SlopesArray]
    pub fn shape(&self) -> (usize, usize) {
        (self.slopes[0].len(), self.slopes.len())
    }
    /// Returns the number of rows of the [SlopesArray]
    #[inline]
    pub fn nrows(&self) -> usize {
        self.slopes[0].len()
    }
    /// Returns the number of columns of the [SlopesArray]
    #[inline]
    pub fn ncols(&self) -> usize {
        self.slopes.len()
    }
    /// Compute the slopes array pseudo-inverse
    pub fn pseudo_inverse(&mut self) -> Result<&mut Self, Box<dyn Error>> {
        let mat = Mat::from_iterator(
            self.nrows(),
            self.ncols(),
            self.slopes.iter().flat_map(|x| x.0.clone()),
        );
        let mat_svd = mat.svd(true, true);
        // dbg!(&mat_svd.singular_values);
        self.inverse = Some(mat_svd.pseudo_inverse(0.)?);
        Ok(self)
    }
}

impl Mul<Slopes> for &mut SlopesArray {
    type Output = Option<Vec<f32>>;
    /// Multiplies the pseudo-inverse of the calibration matrix with the [Slopes]
    fn mul(self, rhs: Slopes) -> Self::Output {
        <&SlopesArray as Mul<Slopes>>::mul(self, rhs)
    }
}
type V = nalgebra::DVector<f32>;
impl Mul<Slopes> for &SlopesArray {
    type Output = Option<Vec<f32>>;
    /// Multiplies the pseudo-inverse of the calibration matrix with the [Slopes]
    fn mul(self, rhs: Slopes) -> Self::Output {
        self.inverse
            .as_ref()
            .map(|pinv| pinv * V::from(rhs))
            .map(|x| x.as_slice().to_vec())
    }
}
