use crseo::{
    gmt::{GmtBuilder, GmtM1, GmtM2, GmtMirror, GmtMirrorBuilder, GmtMx, MirrorGetSet},
    source::SourceBuilder,
    Builder, FromBuilder, Gmt,
};
use faer::{mat::from_column_major_slice, Mat, MatRef};
use serde::{Deserialize, Serialize};
use std::{fmt::Display, fs::File, marker::PhantomData, ops::Mul, path::Path, time::Instant};

include!(concat!(env!("CARGO_MANIFEST_DIR"), "/config.rs"));

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Calib<M: GmtMx, const SID: u8> {
    n_mode: usize,
    c: Vec<f64>,
    mask: Vec<bool>,
    src_builder: SourceBuilder,
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

pub struct CalibPinv<T: faer::Entity>(Mat<T>);

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
}
impl<M, const SID: u8> Calib<M, SID>
where
    Gmt: GmtMirror<M>,
    GmtBuilder: GmtMirrorBuilder<M>,
    M: Default + GmtMx,
{
    pub fn calibrate_segment_modes(&mut self, stroke: f64) -> Result<&mut Self> {
        println!("Calibrating segment modes ...");
        let mut gmt = Gmt::builder().n_mode::<M>(self.n_mode).build()?;
        gmt.keep(&[SID as i32]);
        let mut src = self.src_builder.clone().build()?;

        src.through(&mut gmt).xpupil();
        // let phase0 = src.phase().clone();
        let amplitude0 = src.amplitude();
        let mask: Vec<_> = amplitude0.iter().map(|x| *x > 0.).collect();
        let area0 = mask.iter().filter(|&&x| x).count();

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

    pub fn calibrate_rigid_body_motions(&mut self, stroke: [Option<f64>; 6]) -> Result<&mut Self> {
        println!("Calibrating rigid body motions ...");
        let mut gmt = Gmt::builder().build()?;
        gmt.keep(&[SID as i32]);
        let mut src = self.src_builder.clone().build()?;

        src.through(&mut gmt).xpupil();
        // let phase0 = src.phase().clone();
        let amplitude0 = src.amplitude();
        let mut mask: Vec<_> = amplitude0.iter().map(|x| *x > 0.).collect();
        let area0 = mask.iter().filter(|&&x| x).count();

        let mut tr_xyz = [0f64; 6];
        let mut calib = vec![];
        let now = Instant::now();
        for i in 0..6 {
            let Some(s) = stroke[i] else {
                continue;
            };
            tr_xyz[i] = s;
            <Gmt as GmtMirror<M>>::as_mut(&mut gmt).set_rigid_body_motions(SID, &tr_xyz);
            src.through(&mut gmt).xpupil();
            let amplitude = src.amplitude();
            let area = amplitude.iter().filter(|&&x| x > 0.).count();
            if area != area0 {
                mask.iter_mut()
                    .zip(amplitude.iter())
                    .for_each(|(m, &a)| *m &= a > 0.);
            }
            let push = src.phase().clone();

            tr_xyz[i] *= -1.;
            <Gmt as GmtMirror<M>>::as_mut(&mut gmt).set_rigid_body_motions(SID, &tr_xyz);
            src.through(&mut gmt).xpupil();
            let amplitude = src.amplitude();
            let area = amplitude.iter().filter(|&&x| x > 0.).count();
            if area != area0 {
                mask.iter_mut()
                    .zip(amplitude.iter())
                    .for_each(|(m, &a)| *m &= a > 0.);
            }

            let pushpull: Vec<_> = push
                .iter()
                .zip(src.phase().iter())
                .zip(&mask)
                .map(|((x, y), &m)| if m { 0.5 * (x - y) as f64 / s } else { 0. })
                .collect();
            calib.push(pushpull);

            tr_xyz[i] = 0.;
        }
        calib.iter_mut().for_each(|x| {
            let mut iter = mask.iter();
            x.retain(|_| *iter.next().unwrap())
        });
        println!("Elapsed: {:?}", now.elapsed());
        self.mask = mask;
        self.c = calib.into_iter().flatten().collect();
        Ok(self)
    }

    pub fn match_areas<T: GmtMx>(&mut self, other: &mut Calib<T, SID>) {
        assert_eq!(self.mask.len(), other.mask.len());
        let area_a = self.area();
        let area_b = other.area();
        if area_a > area_b {
            let c_to_area: Vec<_> = self
                .c
                .chunks(area_a)
                .flat_map(|c| {
                    self.mask
                        .iter()
                        .zip(&other.mask)
                        .filter(|&(&ma, _)| ma)
                        .zip(c)
                        .filter(|&((_, &mb), _)| mb)
                        .map(|(_, c)| *c)
                })
                .collect();
            self.c = c_to_area;
            self.mask = other.mask.clone();
        } else {
            let c_to_area: Vec<_> = other
                .c
                .chunks(area_b)
                .flat_map(|c| {
                    other
                        .mask
                        .iter()
                        .zip(&self.mask)
                        .filter(|&(&ma, _)| ma)
                        .zip(c)
                        .filter(|&((_, &mb), _)| mb)
                        .map(|(_, c)| *c)
                })
                .collect();
            other.c = c_to_area;
            other.mask = self.mask.clone();
        }
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

impl<M: GmtMx, const SID: u8> Mul<&Calib<M, SID>> for &CalibPinv<f64> {
    type Output = Mat<f64>;
    fn mul(self, rhs: &Calib<M, SID>) -> Self::Output {
        self.0.as_ref() * rhs.mat_ref()
    }
}
