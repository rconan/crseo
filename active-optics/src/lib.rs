use faer::{mat::from_column_major_slice, Mat};
use serde::{Deserialize, Serialize};
use std::{fmt::Display, fs::File, ops::Mul, path::Path};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Calib<const SID: u8> {
    n_mode: usize,
    c: Vec<f64>,
    mask: Vec<bool>,
}

pub struct CalibPinv<T: faer::Entity>(Mat<T>);

impl<const SID: u8> Calib<SID> {
    pub fn new(n_mode: usize, c: Vec<f64>, mask: Vec<bool>) -> Self {
        Self { n_mode, c, mask }
    }
    pub fn area(&self) -> usize {
        self.mask.iter().filter(|x| **x).count()
    }
    pub fn dump<P: AsRef<Path> + Display>(&self, path: P) -> Result<()> {
        let mut file = File::create(&path)?;
        serde_pickle::to_writer(&mut file, self, Default::default())?;
        println!("calib written to {:}", path);
        Ok(())
    }
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::open(path)?;
        Ok(serde_pickle::from_reader(file, Default::default())?)
    }
    #[inline]
    pub fn nrows(&self) -> usize {
        self.c.len() / self.n_mode
    }
    #[inline]
    pub fn ncols(&self) -> usize {
        self.n_mode
    }
    pub fn pseudoinverse(&self) -> CalibPinv<f64> {
        let c_mat = from_column_major_slice::<f64>(&self.c, self.nrows(), self.ncols());
        let svd = c_mat.svd();
        CalibPinv(svd.pseudoinverse())
    }
    pub fn apply_mask(&self, data: &[f64]) -> Vec<f64> {
        assert_eq!(data.len(), self.mask.len());
        data.iter()
            .zip(self.mask.iter())
            .filter_map(|(x, b)| if *b { Some(*x) } else { None })
            .collect()
    }
}

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
