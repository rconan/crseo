mod algebra;
mod calibration;

use crseo::gmt::{GmtM1, GmtM2, GmtMx};
use crseo::source::SourceBuilder;
use faer::mat::from_column_major_slice;
use faer::{Mat, MatRef};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::fs::File;
use std::marker::PhantomData;
use std::path::Path;

pub struct CalibPinv<T: faer::Entity>(Mat<T>);

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Calib<M: GmtMx, const SID: u8> {
    n_mode: usize,
    pub(crate) c: Vec<f64>,
    pub(crate) mask: Vec<bool>,
    pub(crate) src_builder: SourceBuilder,
    mirror: PhantomData<M>,
}

impl<const SID: u8> Display for Calib<GmtM1, SID> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Calib M1S{} ({}, {}); area = {}",
            SID,
            self.c.len() / self.n_mode,
            self.n_mode,
            self.area()
        )
    }
}

impl<const SID: u8> Display for Calib<GmtM2, SID> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Calib M2S{} ({}, {}); area = {}",
            SID,
            self.c.len() / self.n_mode,
            self.n_mode,
            self.area()
        )
    }
}

impl<M, const SID: u8> Calib<M, SID>
where
    M: Default + GmtMx,
{
    pub fn new(n_mode: usize) -> Self {
        Self {
            n_mode,
            ..Default::default()
        }
    }
    pub fn guide_star(mut self, src_builder: SourceBuilder) -> Self {
        self.src_builder = src_builder;
        self
    }
}
impl<M, const SID: u8> Calib<M, SID>
where
    M: GmtMx,
{
    pub fn area(&self) -> usize {
        self.mask.iter().filter(|x| **x).count()
    }
    pub fn dump<P: AsRef<Path> + Display>(&self, path: P) -> crate::Result<()> {
        let mut file = File::create(&path)?;
        serde_pickle::to_writer(&mut file, self, Default::default())?;
        println!("calib written to {:}", path);
        Ok(())
    }
    pub fn load<P: AsRef<Path>>(path: P) -> crate::Result<Self> {
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
    pub fn mat_ref(&self) -> MatRef<'_, f64> {
        from_column_major_slice::<f64>(&self.c, self.nrows(), self.ncols())
    }
    pub fn pseudoinverse(&self) -> CalibPinv<f64> {
        let svd = self.mat_ref().svd();
        CalibPinv(svd.pseudoinverse())
    }
    pub fn apply_mask(&self, data: &[f64]) -> Vec<f64> {
        assert_eq!(data.len(), self.mask.len());
        data.iter()
            .zip(self.mask.iter())
            .filter_map(|(x, b)| if *b { Some(*x) } else { None })
            .collect()
    }
    pub fn unmask<'a>(&'a self, mut data: impl Iterator<Item = &'a f64>) -> Vec<f64> {
        self.mask
            .iter()
            .map(|b| if *b { *data.next().unwrap() } else { 0. })
            .collect()
    }
    #[inline]
    pub fn mask_len(&self) -> usize {
        self.mask.len()
    }
    #[inline]
    pub fn src_mask_len(&self) -> usize {
        self.mask.len() / self.src_builder.size
    }
    #[inline]
    pub fn src_mask_square_len(&self) -> usize {
        (self.src_mask_len() as f64).sqrt() as usize
    }
}
