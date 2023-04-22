use std::{
    error::Error,
    fmt::{self, Display, Formatter},
    ops::Mul,
};

use nalgebra::DMatrix;
use serde::Serialize;

use super::{DataRef, Slopes};

type Mat = nalgebra::DMatrix<f32>;

#[derive(Debug)]
#[non_exhaustive]
pub enum SlopesArrayError {
    Nalgebra { kind: NalgebraErrorKind },
}

impl Display for SlopesArrayError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            SlopesArrayError::Nalgebra { .. } => f.write_str("nalgebra error"),
        }
    }
}

impl Error for SlopesArrayError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            SlopesArrayError::Nalgebra { kind } => Some(kind),
            // _ => None,
        }
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum NalgebraErrorKind {
    PseudoInverse(String),
}

impl Display for NalgebraErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            NalgebraErrorKind::PseudoInverse(msg) => write!(f, "pseudo-inverse: {}", msg),
        }
    }
}

impl Error for NalgebraErrorKind {}

/// A collection of pyramid measurements
#[derive(Debug, Clone, Default, Serialize, PartialEq)]
pub struct SlopesArray {
    pub(crate) slopes: Vec<Slopes>,
    pub data_ref: DataRef,
    #[serde(skip)]
    pub(crate) inverse: Option<Mat>,
}

impl From<(DataRef, Vec<Slopes>)> for SlopesArray {
    /// Convert a vector of measurements and the associated [QuadCell] into a [SlopesArray]
    fn from((data_ref, slopes): (DataRef, Vec<Slopes>)) -> Self {
        Self {
            slopes,
            data_ref,
            ..Default::default()
        }
    }
}

impl From<DMatrix<f32>> for SlopesArray {
    fn from(value: DMatrix<f32>) -> Self {
        let slopes: Vec<_> = value
            .column_iter()
            .map(|row| Slopes::from(row.as_slice().to_vec()))
            .collect();
        Self {
            slopes,
            ..Default::default()
        }
    }
}

impl Display for SlopesArray {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "SlopesArray: {:?}", self.shape())
    }
}

#[derive(Clone, Debug)]
pub enum TruncatedPseudoInverse {
    Threshold(f32),
    EigenValues(usize),
}
impl Default for TruncatedPseudoInverse {
    fn default() -> Self {
        TruncatedPseudoInverse::Threshold(0f32)
    }
}
impl From<f32> for TruncatedPseudoInverse {
    fn from(value: f32) -> Self {
        TruncatedPseudoInverse::Threshold(value)
    }
}
impl From<usize> for TruncatedPseudoInverse {
    fn from(value: usize) -> Self {
        TruncatedPseudoInverse::EigenValues(value)
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
    /// Returns the interaction matrix
    pub fn interaction_matrix(&self) -> DMatrix<f32> {
        Mat::from_iterator(
            self.nrows(),
            self.ncols(),
            self.slopes.iter().flat_map(|x| x.0.clone()),
        )
    }
    /// Computes the slopes array pseudo-inverse
    pub fn pseudo_inverse(
        &mut self,
        truncation: Option<TruncatedPseudoInverse>,
    ) -> Result<&mut Self, SlopesArrayError> {
        let mat = self.interaction_matrix();
        let mat_svd = mat.svd(true, true);
        // dbg!(&mat_svd.singular_values);

        if let Some(truncation) = truncation {
            match truncation {
                TruncatedPseudoInverse::Threshold(value) => {
                    self.inverse = Some(mat_svd.pseudo_inverse(value).map_err(|msg| {
                        SlopesArrayError::Nalgebra {
                            kind: NalgebraErrorKind::PseudoInverse(msg.to_string()),
                        }
                    })?);
                }
                TruncatedPseudoInverse::EigenValues(n) => {
                    let value = mat_svd.singular_values[mat_svd.singular_values.len() - 1 - n];
                    self.inverse = Some(mat_svd.pseudo_inverse(value).map_err(|msg| {
                        SlopesArrayError::Nalgebra {
                            kind: NalgebraErrorKind::PseudoInverse(msg.to_string()),
                        }
                    })?);
                }
            }
        } else {
            self.inverse =
                Some(
                    mat_svd
                        .pseudo_inverse(0.)
                        .map_err(|msg| SlopesArrayError::Nalgebra {
                            kind: NalgebraErrorKind::PseudoInverse(msg.to_string()),
                        })?,
                );
        }

        Ok(self)
    }
    /// Removes the [Slopes] at given indices `idxs` in the [Slopes] vector
    pub fn trim(&mut self, idxs: Vec<usize>) {
        let mut count = 0;
        for idx in idxs.into_iter() {
            let i = idx - count;
            self.slopes.remove(i);
            count += 1;
        }
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
