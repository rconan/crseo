use crseo::{
    gmt::{GmtBuilder, GmtMirror, GmtMirrorBuilder, GmtMx, MirrorGetSet},
    Builder, FromBuilder, Gmt, Source,
};
use faer::{mat::from_column_major_slice, Mat};
use serde::{Deserialize, Serialize};
use std::{fmt::Display, fs::File, marker::PhantomData, ops::Mul, path::Path, time::Instant};

include!(concat!(env!("CARGO_MANIFEST_DIR"), "/config.rs"));

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Calib<M: GmtMx, const SID: u8> {
    n_mode: usize,
    c: Vec<f64>,
    mask: Vec<bool>,
    mirror: PhantomData<M>,
}

pub struct CalibPinv<T: faer::Entity>(Mat<T>);

impl<M, const SID: u8> Calib<M, SID>
where
    Gmt: GmtMirror<M>,
    GmtBuilder: GmtMirrorBuilder<M>,
    M: Default + GmtMx,
{
    pub fn new(n_mode: usize) -> Self {
        Self {
            n_mode,
            ..Default::default()
        }
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
    pub fn calibrate_segment_modes(&mut self, stroke: f64) -> Result<&mut Self> {
        let mut gmt = Gmt::builder().n_mode::<M>(self.n_mode).build()?;
        gmt.keep(&[SID as i32]);
        let mut src = Source::builder().build()?;

        src.through(&mut gmt).xpupil();
        // let phase0 = src.phase().clone();
        let amplitude0 = src.amplitude();
        let mask: Vec<_> = amplitude0.iter().map(|x| *x > 0.).collect();
        let area0 = mask.iter().filter(|&&x| x).count();
        dbg!(area0);

        // let stroke = 1e-6;
        let mut a = vec![0f64; self.n_mode];
        let mut calib: Vec<f64> = Vec::new();
        let now = Instant::now();
        for i in 0..self.n_mode {
            a[i] = stroke;
            <Gmt as GmtMirror<M>>::as_mut(&mut gmt).set_segment_modes(SID, &a);
            src.through(&mut gmt).xpupil();
            let area = src.amplitude().iter().filter(|&&x| x > 0.).count();
            if area != area0 {
                panic!("Expected area={}, found {}", area0, area);
            }
            let push = src.phase().clone();

            a[i] *= -1.;
            <Gmt as GmtMirror<M>>::as_mut(&mut gmt).set_segment_modes(SID, &a);
            src.through(&mut gmt).xpupil();
            let area = src.amplitude().iter().filter(|&&x| x > 0.).count();
            if area != area0 {
                panic!("Expected area={}, found {}", area0, area);
            }

            let pushpull = push
                .iter()
                .zip(src.phase().iter())
                .zip(&mask)
                .filter(|&(_, &m)| m)
                .map(|((x, y), _)| 0.5 * (x - y) as f64 / stroke);
            calib.extend(pushpull);

            a[i] = 0.;
        }
        println!("Elapsed: {:?}", now.elapsed());
        self.mask = mask;
        self.c = calib;
        Ok(self)
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
