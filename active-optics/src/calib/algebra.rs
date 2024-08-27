use crate::calib::{Calib, CalibPinv};
use crseo::gmt::GmtMx;
use faer::mat::from_column_major_slice;
use faer::Mat;
use std::ops::Mul;

impl<'a> Mul<&'a [f64]> for &'a CalibPinv<f64> {
    type Output = Vec<f64>;
    fn mul(self, rhs: &'a [f64]) -> Self::Output {
        let e = self.0.as_ref() * from_column_major_slice::<f64>(rhs, rhs.len(), 1);
        e.row_iter()
            .flat_map(|r| r.iter().cloned().collect::<Vec<_>>())
            .collect()
    }
}

impl Mul<Vec<f64>> for &CalibPinv<f64> {
    type Output = Vec<f64>;
    fn mul(self, rhs: Vec<f64>) -> Self::Output {
        let e = self.0.as_ref() * from_column_major_slice::<f64>(rhs.as_slice(), rhs.len(), 1);
        e.row_iter()
            .flat_map(|r| r.iter().cloned().collect::<Vec<_>>())
            .collect()
    }
}

impl<M: GmtMx, const SID: u8> Mul<&Calib<M, SID>> for &CalibPinv<f64> {
    type Output = Mat<f64>;
    fn mul(self, rhs: &Calib<M, SID>) -> Self::Output {
        self.0.as_ref() * rhs.mat_ref()
    }
}
