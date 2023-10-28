use std::fmt::Display;

use serde::{Deserialize, Serialize};

use super::Slopes;

type Mat = nalgebra::DMatrix<f32>;

/// Quad cell data
///
/// Holds the mask applied to the detector frame and
/// the reference slopes
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DataRef {
    pub(crate) mask: Option<nalgebra::DMatrix<bool>>,
    pub(crate) sxy0: Option<Slopes>,
}
impl DataRef {
    pub fn new(mask: nalgebra::DMatrix<f32>) -> Self {
        Self {
            mask: Some(nalgebra::DMatrix::<bool>::from_iterator(
                mask.nrows(),
                mask.ncols(),
                mask.into_iter()
                    .map(|p| if *p > 0f32 { true } else { false }),
            )),
            sxy0: None,
        }
    }
    pub fn set_ref_with(&mut self, slopes: Slopes) {
        self.sxy0 = Some(slopes);
    }
    pub fn sx(&self, slopes: &Slopes) -> Option<Mat> {
        let Some(mask) = self.mask.as_ref() else {
            return None;
        };
        let (nrows, ncols) = mask.shape();
        let mut slopes_iter = slopes.0.iter().step_by(2);
        Some(Mat::from_iterator(
            nrows,
            ncols,
            mask.iter().map(|m| {
                if *m {
                    *slopes_iter.next().unwrap()
                } else {
                    0f32
                }
            }),
        ))
    }
    pub fn sy(&self, slopes: &Slopes) -> Option<Mat> {
        let Some(mask) = self.mask.as_ref() else {
            return None;
        };
        let (nrows, ncols) = mask.shape();
        let mut slopes_iter = slopes.0.iter().skip(1).step_by(2);
        Some(Mat::from_iterator(
            nrows,
            ncols,
            mask.iter().map(|m| {
                if *m {
                    *slopes_iter.next().unwrap()
                } else {
                    0f32
                }
            }),
        ))
    }
    pub fn mask(&self) -> Option<&nalgebra::DMatrix<bool>> {
        self.mask.as_ref()
    }
}

impl Display for DataRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (self.mask.as_ref(), self.sxy0.as_ref()) {
            (None, None) => write!(f, "Empty DataRef"),
            (None, Some(sxy0)) => write!(f, "DataRef with no mask and {} slopes", sxy0.len()),
            (Some(mask), None) => {
                write!(
                    f,
                    "DataRef with no slopes and a {:?} mask(nnz={})",
                    mask.shape(),
                    mask.iter()
                        .filter(|&&x| x)
                        .enumerate()
                        .last()
                        .map(|(l, _)| l + 1)
                        .unwrap()
                )
            }
            (Some(mask), Some(sxy0)) => write!(
                f,
                "DataRef with {} slopes and a {:?} mask(nnz={})",
                sxy0.len(),
                mask.shape(),
                mask.iter()
                    .filter(|&&x| x)
                    .enumerate()
                    .last()
                    .map(|(l, _)| l + 1)
                    .unwrap()
            ),
        }
    }
}
