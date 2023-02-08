use std::{
    error::Error,
    ops::{Deref, DerefMut, Div, DivAssign, Mul, Sub},
};

use serde::{Deserialize, Serialize};

use crate::wavefrontsensor::LensletArray;

use super::{Mat, Pyramid, QuadCell};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Slopes(pub(super) Vec<f32>);
impl Slopes {
    pub fn len(&self) -> usize {
        self.0.len()
    }
}
type V = nalgebra::DVector<f32>;
impl From<Slopes> for V {
    fn from(value: Slopes) -> Self {
        V::from_column_slice(&value.0)
    }
}

/* impl From<(&QuadCell, &mut Pyramid)> for Slopes {
    fn from((qc, pym): (&QuadCell, &mut Pyramid)) -> Self {
        <Slopes as From<(&QuadCell, &Pyramid)>>::from((qc, pym))
    }
} */
impl From<(&QuadCell, &Pyramid)> for Slopes {
    fn from((qc, pym): (&QuadCell, &Pyramid)) -> Self {
        let (sx, sy, a) = {
            let (n, m) = pym.camera_resolution();
            let LensletArray(n_lenslet, _, _) = pym.lenslet_array;
            let n0 = n_lenslet / 2;
            let n1 = n0 + n / 2;
            let mat: Mat = nalgebra::DMatrix::from_column_slice(n, m, &pym.frame());
            let row_diff = mat.rows(n0, n_lenslet) - mat.rows(n1, n_lenslet);
            let sx = row_diff.columns(n0, n_lenslet) + row_diff.columns(n1, n_lenslet);
            let col_diff = mat.columns(n0, n_lenslet) - mat.columns(n1, n_lenslet);
            let sy = col_diff.rows(n0, n_lenslet) + col_diff.rows(n1, n_lenslet);

            let row_sum = mat.rows(n0, n_lenslet) + mat.rows(n1, n_lenslet);
            let a = row_sum.columns(n0, n_lenslet) + row_sum.columns(n1, n_lenslet);
            (sx, sy, a)
        };

        let iter = sx.into_iter().zip(sy.into_iter()).zip(&a);
        let mut sxy: Vec<_> = if let Some(mask) = qc.mask.as_ref() {
            iter.zip(mask)
                .filter(|(_, &m)| m)
                .flat_map(|(((sx, sy), a), _)| vec![sx / a, sy / a])
                .collect()
        } else {
            iter.flat_map(|((sx, sy), a)| vec![sx / a, sy / a])
                .collect()
        };
        if let Some(Slopes(sxy0)) = qc.sxy0.as_ref() {
            sxy.iter_mut()
                .zip(sxy0)
                .for_each(|(sxy, sxy0)| *sxy -= sxy0);
        }
        Slopes(sxy)
    }
}

impl Div<f32> for Slopes {
    type Output = Slopes;

    fn div(self, rhs: f32) -> Self::Output {
        Slopes(self.0.into_iter().map(|x| x / rhs).collect())
    }
}

impl DivAssign<f32> for Slopes {
    fn div_assign(&mut self, rhs: f32) {
        self.0.iter_mut().for_each(|x| *x /= rhs);
    }
}

impl Sub for Slopes {
    type Output = Slopes;

    fn sub(self, rhs: Self) -> Self::Output {
        Slopes(
            self.0
                .into_iter()
                .zip(rhs.0.into_iter())
                .map(|(x, y)| x - y)
                .collect(),
        )
    }
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct SlopesArray {
    pub(super) slopes: Vec<Slopes>,
    pub(super) quad_cell: QuadCell,
    #[serde(skip)]
    inverse: Option<Mat>,
}
impl From<(QuadCell, Vec<Slopes>)> for SlopesArray {
    fn from((quad_cell, slopes): (QuadCell, Vec<Slopes>)) -> Self {
        Self {
            slopes,
            quad_cell,
            inverse: None,
        }
    }
}
impl SlopesArray {
    pub fn new(slopes: Vec<Slopes>) -> Self {
        Self {
            slopes,
            ..Default::default()
        }
    }
    pub fn shape(&self) -> (usize, usize) {
        (self.slopes[0].len(), self.slopes.len())
    }
    pub fn nrows(&self) -> usize {
        self.slopes[0].len()
    }
    pub fn ncols(&self) -> usize {
        self.slopes.len()
    }
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

/* #[derive(Debug,thiserror::Error)]
pub enum SlopesError {
    #[error("")]
    PseudoInverse(#[from] nalgebra::e)
} */

impl Mul<Slopes> for &mut SlopesArray {
    type Output = Option<Vec<f32>>;

    fn mul(self, rhs: Slopes) -> Self::Output {
        self.inverse
            .as_ref()
            .map(|pinv| pinv * V::from(rhs))
            .map(|x| x.as_slice().to_vec())
    }
}
impl Mul<Slopes> for &SlopesArray {
    type Output = Option<Vec<f32>>;

    fn mul(self, rhs: Slopes) -> Self::Output {
        self.inverse
            .as_ref()
            .map(|pinv| pinv * V::from(rhs))
            .map(|x| x.as_slice().to_vec())
    }
}
impl Mul<&Pyramid> for &SlopesArray {
    type Output = Option<Vec<f32>>;

    fn mul(self, pym: &Pyramid) -> Self::Output {
        let slopes = Slopes::from((&self.quad_cell, pym));
        self.inverse
            .as_ref()
            .map(|pinv| pinv * V::from(slopes))
            .map(|x| x.as_slice().to_vec())
    }
}

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
    pub fn shape(&self) -> (usize, usize) {
        (self.nrows(), self.ncols())
    }
    pub fn nrows(&self) -> usize {
        self.iter().map(|x| x.nrows()).sum()
    }
    pub fn ncols(&self) -> usize {
        self.iter().map(|x| x.ncols()).sum()
    }
    pub fn pseudo_inverse(&mut self) -> Result<&mut Self, Box<dyn Error>> {
        self.iter_mut()
            .map(|x| x.pseudo_inverse())
            .collect::<Result<Vec<_>, Box<dyn Error>>>()?;
        Ok(self)
    }
}
impl Mul<&Pyramid> for &Calibration {
    type Output = Option<Vec<f32>>;

    fn mul(self, pym: &Pyramid) -> Self::Output {
        Some(self.iter().flat_map(|x| x * pym).flatten().collect())
    }
}
