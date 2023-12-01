use std::{
    error::Error,
    fmt::Display,
    ops::{Add, Deref, DerefMut, Div, Mul, SubAssign},
};

use nalgebra::DMatrix;
use serde::{Deserialize, Serialize};
use slopes::Slopes;

use crate::{
    wavefrontsensor::{
        segment_wise::data_processing::{
            slopes, slopes_array::SlopesArrayError, TruncatedPseudoInverse,
        },
        SlopesArray,
    },
    SourceBuilder,
};

#[derive(Debug)]
#[non_exhaustive]
pub enum CalibrationError {
    SlopesArray(SlopesArrayError),
    Collect,
}
impl Display for CalibrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CalibrationError::SlopesArray(_) => f.write_str("error in SlopesArray"),
            CalibrationError::Collect => {
                f.write_str("failed to flatten Calibration because of DataRef mismatch")
            }
        }
    }
}
impl Error for CalibrationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            CalibrationError::SlopesArray(e) => Some(e),
            _ => None,
        }
    }
}
impl From<SlopesArrayError> for CalibrationError {
    fn from(value: SlopesArrayError) -> Self {
        Self::SlopesArray(value)
    }
}

/// A collection of [SlopesArray]
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct Calibration {
    pub(crate) data: Vec<SlopesArray>,
    pub(crate) src: SourceBuilder,
}
impl Deref for Calibration {
    type Target = Vec<SlopesArray>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}
impl DerefMut for Calibration {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}
impl Display for Calibration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        println!("Calibration:");
        for s in self.iter() {
            s.fmt(f)?;
        }
        Ok(())
    }
}
impl From<(DMatrix<f32>, Calibration)> for Calibration {
    fn from((value, mut cal): (DMatrix<f32>, Calibration)) -> Self {
        assert_eq!(cal.data.len(), 1);
        let sa = cal.data.pop().unwrap();
        Self {
            data: vec![(value, sa).into()],
            ..Default::default()
        }
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
    /// Returns the number of sub-matrices
    pub fn size(&self) -> usize {
        self.data.len()
    }
    /// Returns the slope masks
    pub fn masks<'a>(&'a self) -> impl Iterator<Item = Option<&'a nalgebra::DMatrix<bool>>> {
        self.data.iter().map(|s| s.data_ref.mask())
    }
    /// Returns the reference slopes
    pub fn reference_slopes(&self) -> Vec<Option<&Vec<f32>>> {
        self.data.iter().map(|sa| sa.reference_slopes()).collect()
    }
    /// Return the condition number of the interaction matrices
    pub fn condition_number(&self, lasts: Option<Vec<Option<usize>>>) -> Vec<f32> {
        match lasts {
            Some(lasts) => self
                .iter()
                .zip(lasts.into_iter())
                .map(|(x, last)| x.condition_number(last))
                .collect(),
            None => self.iter().map(|x| x.condition_number(None)).collect(),
        }
    }
    /// Compute the pseudo-inverse of each slope array
    pub fn pseudo_inverse(
        &mut self,
        truncation: Option<Vec<Option<TruncatedPseudoInverse>>>,
    ) -> Result<&mut Self, CalibrationError> {
        let n = self.size();
        self.iter_mut()
            .zip(truncation.unwrap_or(vec![None; n]).into_iter())
            .map(|(x, t)| x.pseudo_inverse(t))
            .collect::<Result<Vec<_>, SlopesArrayError>>()?;
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
        self.data.len()
    }
    /// Returns the interaction matrices
    pub fn interaction_matrices(&self) -> Vec<DMatrix<f32>> {
        self.iter().map(|s| s.interaction_matrix()).collect()
    }
    /// Removes the [SlopesArray] at given indices in the [SlopesArray] vector
    ///
    /// If some other indices are given as well, keep the [SlopesArray]
    /// but removes the [Slopes] at the other indices in the [Slopes] vector
    pub fn trim(&mut self, indices: Vec<(usize, Option<Vec<usize>>)>) -> &mut Self {
        let mut count = 0;
        for (idx, maybe_idx) in indices.into_iter() {
            let i = idx - count;
            if let Some(idxs) = maybe_idx {
                self.data.get_mut(i).map(|sa| sa.trim(idxs));
            } else {
                self.data.remove(i);
                count += 1;
            }
        }
        self
    }
    /// Concatenates all the slopes array into a single one
    ///
    /// Failed if all the [DataRef] do not match
    pub fn flatten(self) -> Result<Self, CalibrationError> {
        let mut sa_iter = self.data.into_iter();
        let SlopesArray {
            mut slopes,
            data_ref,
            ..
        } = sa_iter.next().unwrap();
        for mut sa in sa_iter {
            if sa.data_ref == data_ref {
                slopes.append(&mut sa.slopes);
            } else {
                return Err(CalibrationError::Collect);
            }
        }
        Ok(Self {
            data: vec![SlopesArray {
                slopes,
                data_ref,
                inverse: None,
            }],
            ..Default::default()
        })
    }
    pub fn insert_rows(&mut self, indices: Vec<(usize, Vec<usize>)>) {
        for (sa_idx, rows) in indices.into_iter() {
            self.data.get_mut(sa_idx).map(|sa| sa.insert_rows(rows));
        }
    }
}

impl Add for Calibration {
    type Output = Calibration;

    fn add(self, rhs: Self) -> Self::Output {
        Calibration {
            data: self.data.into_iter().chain(rhs.data.into_iter()).collect(),
            ..Default::default()
        }
    }
}

impl Mul for Calibration {
    type Output = Calibration;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            data: self
                .interaction_matrices()
                .into_iter()
                .zip(rhs.interaction_matrices())
                .map(|(a, b)| a * b)
                .map(|c| SlopesArray::from(c))
                .collect(),
            ..Default::default()
        }
    }
}

impl Div for Calibration {
    type Output = Result<Calibration, Box<dyn Error>>;

    fn div(self, rhs: Self) -> Self::Output {
        let mut slopes_array: Vec<SlopesArray> = vec![];
        for (a, b) in self
            .interaction_matrices()
            .into_iter()
            .zip(rhs.interaction_matrices())
        {
            let c = a * b.pseudo_inverse(0.)?;
            slopes_array.push(SlopesArray::from(c));
        }
        Ok(Self {
            data: slopes_array,
            ..Default::default()
        })
    }
}

impl SubAssign for Calibration {
    fn sub_assign(&mut self, rhs: Self) {
        self.data
            .iter_mut()
            .zip(rhs.interaction_matrices())
            .for_each(|(sa, b)| {
                let a = sa.interaction_matrix();
                let c = a - b;
                let slopes: Vec<_> = c
                    .column_iter()
                    .map(|row| Slopes::from(row.as_slice().to_vec()))
                    .collect();
                sa.slopes = slopes;
            });
    }
}
