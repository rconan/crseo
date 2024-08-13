use std::error::Error;
use std::fmt::Display;
use std::fs::File;
use std::ops::Mul;
use std::path::PathBuf;

use indicatif::ProgressBar;
use nalgebra::{DMatrix, DVector};

use crate::imaging::LensletArray;
use crate::{
    Builder, Frame, FromBuilder, Gmt, Propagation, SegmentWiseSensor, SourceBuilder,
    WavefrontSensor,
};

use super::data_processing::{Calibration, DataRef, Slopes, SlopesArray};
use super::piston_sensor::PistonSensor;
use super::{Modulation, PyramidBuilder};

type Mat = nalgebra::DMatrix<f32>;

/// Wrapper to CEO pyramid
/// # Examples
///
/// ```
/// use crseo::{ceo, Pyramid};
/// // Creates a pyramid instance with default parameters
/// let mut src = ceo!(Pyramid);
/// ```
pub struct Pyramid {
    pub(super) _c_: ffi::pyramid,
    pub lenslet_array: LensletArray,
    pub(super) alpha: f32,
    pub(super) modulation: Option<Modulation>,
    pub(super) piston_sensor: Option<PistonSensor>,
}
impl Drop for Pyramid {
    /// Frees CEO memory before dropping `Pyramid`
    fn drop(&mut self) {
        unsafe {
            self._c_.cleanup();
        }
    }
}
impl FromBuilder for Pyramid {
    type ComponentBuilder = PyramidBuilder;
}
impl Propagation for Pyramid {
    fn propagate(&mut self, src: &mut crate::Source) {
        if let Some(Modulation {
            amplitude,
            sampling,
        }) = self.modulation
        {
            unsafe {
                self._c_.camera.propagateThroughModulatedPyramid(
                    src.as_raw_mut_ptr(),
                    amplitude,
                    sampling,
                    self.alpha,
                )
            }
        } else {
            unsafe {
                self._c_
                    .camera
                    .propagateThroughPyramid(src.as_raw_mut_ptr(), self.alpha)
            }
        }
    }

    fn time_propagate(&mut self, _secs: f64, _src: &mut crate::Source) {
        todo!()
    }
}

impl Pyramid {
    /// Returns the  detector frame
    pub fn frame(&self) -> Vec<f32> {
        let n = self._c_.camera.N_PX_CAMERA.pow(2) * self._c_.camera.N_SOURCE;
        let mut frame = vec![0f32; n as usize];
        unsafe {
            ffi::dev2host(frame.as_mut_ptr(), self._c_.camera.d__frame, n);
        }
        frame
    }
    #[inline]
    /// Returns the detector pixel length
    pub fn n_px_camera(&self) -> usize {
        self._c_.camera.N_PX_CAMERA as usize
    }
    /// Returns the detector resolution
    pub fn camera_resolution(&self) -> (usize, usize) {
        (self.n_px_camera(), self.n_px_camera())
    }
    pub(crate) fn processing(&self) -> (Mat, Mat) {
        let (n, m) = self.camera_resolution();
        let LensletArray { n_side_lenslet, .. } = self.lenslet_array;
        let n0 = n_side_lenslet / 2;
        let n1 = n0 + n / 2;
        let mat: Mat = nalgebra::DMatrix::from_column_slice(n, m, &self.frame());
        let row_diff = mat.rows(n0, n_side_lenslet) - mat.rows(n1, n_side_lenslet);
        let row_col_data =
            row_diff.columns(n0, n_side_lenslet) + row_diff.columns(n1, n_side_lenslet);
        let col_diff = mat.columns(n0, n_side_lenslet) - mat.columns(n1, n_side_lenslet);
        let col_row_data = col_diff.rows(n0, n_side_lenslet) + col_diff.rows(n1, n_side_lenslet);

        let row_sum = mat.rows(n0, n_side_lenslet) + mat.rows(n1, n_side_lenslet);
        let a = row_sum.columns(n0, n_side_lenslet) + row_sum.columns(n1, n_side_lenslet);
        let mut flux = a.as_slice().to_vec();
        flux.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let med_flux = match flux.len() {
            n if n % 2 == 0 => 0.5 * (flux[n / 2] + flux[1 + n / 2]),
            n => flux[(n + 1) / 2],
        };

        (
            row_col_data.map(|v| v / med_flux),
            col_row_data.map(|v| v / med_flux),
        )
    }
    /*     pub fn add_quads(&mut self) -> Mat {
        let (n, m) = self.camera_resolution();
        let LensletArray { n_side_lenslet, .. } = self.lenslet_array;
        let n0 = n_side_lenslet / 2;
        let n1 = n0 + n / 2;
        let mat: Mat = nalgebra::DMatrix::from_column_slice(n, m, &self.frame());
        let row_diff = mat.rows(n0, n_side_lenslet) + mat.rows(n1, n_side_lenslet);
        row_diff.columns(n0, n_side_lenslet) + row_diff.columns(n1, n_side_lenslet)
    } */
    pub fn piston(&self) -> Option<Vec<f32>> {
        if self.piston_sensor.is_none() {
            return None;
        };
        let sxy = self.processing();
        let piston_sensor = self.piston_sensor.as_ref().unwrap();
        let data = sxy
            .0
            .into_iter()
            .zip(&piston_sensor.mask.0)
            .filter_map(|(v, m)| if *m { Some(*v) } else { None })
            .zip(&piston_sensor.sxy0.0)
            .map(|(s, s0)| s - s0)
            .chain(
                sxy.1
                    .into_iter()
                    .zip(&piston_sensor.mask.1)
                    .filter_map(|(v, m)| if *m { Some(*v) } else { None })
                    .zip(&piston_sensor.sxy0.1)
                    .map(|(s, s0)| s - s0),
            );
        let piston = &piston_sensor.pseudo_inverse
            * nalgebra::DVector::from_iterator(piston_sensor.calibration.nrows(), data);
        Some(piston.as_slice().to_vec())
    }
}

impl SegmentWiseSensor for Pyramid {
    fn pupil_sampling(&self) -> usize {
        let LensletArray {
            n_side_lenslet,
            n_px_lenslet,
            ..
        } = self.lenslet_array;
        n_side_lenslet * n_px_lenslet
    }
    fn zeroed_segment(&mut self, sid: usize, src_builder: Option<SourceBuilder>) -> DataRef {
        let LensletArray { n_side_lenslet, .. } = self.lenslet_array;
        // Setting the pyramid mask restricted to the segment
        let mut gmt = Gmt::builder().build().unwrap();
        gmt.keep(&[sid as i32]);

        let mut src = src_builder
            .clone()
            .unwrap()
            .pupil_sampling(n_side_lenslet)
            .build()
            .unwrap();
        src.through(&mut gmt).xpupil();

        let pupil = nalgebra::DMatrix::<f32>::from_iterator(
            n_side_lenslet,
            n_side_lenslet,
            src.amplitude().into_iter().rev(),
        );

        let mut data_ref = DataRef::new(pupil);

        gmt.keep(&[sid as i32]);
        let mut src = src_builder
            .clone()
            .unwrap()
            .pupil_sampling(self.pupil_sampling())
            .build()
            .unwrap();
        self.reset();

        src.through(&mut gmt).xpupil().through(self);
        data_ref.set_ref_with(Slopes::from((&data_ref, &*self)));
        self.reset();
        data_ref
    }
    fn into_slopes(&self, data_ref: &DataRef) -> Slopes {
        Slopes::from((data_ref, self))
    }
    fn calibrate_segment(
        &mut self,
        _src_builder: Option<SourceBuilder>,
        _sid: usize,
        _n_mode: usize,
        _pb: Option<ProgressBar>,
    ) -> SlopesArray {
        /* let quad_cell = self.zeroed_segment(sid, src_builder.clone());

        let mut gmt = Gmt::builder().m2("Karhunen-Loeve", n_mode).build().unwrap();
        gmt.keep(&[sid as i32]);

        let mut src = src_builder
            .unwrap_or_default()
            .pupil_sampling(self.pupil_sampling())
            .build()
            .unwrap();

        let mut slopes = vec![];
        let o2p = (2. * std::f64::consts::PI / src.wavelength()) as f32;

        for kl_mode in 1..n_mode {
            pb.as_ref().map(|pb| pb.inc(1));
            gmt.reset();
            let kl_a0 = 1e-6;
            gmt.m2_modes_ij(sid - 1, kl_mode, kl_a0);
            src.through(&mut gmt).xpupil();
            let opd = src.phase().clone();
            let opd_minmax =
                opd.iter()
                    .fold((f32::INFINITY, f32::NEG_INFINITY), |(min, max), value| {
                        (
                            if *value < min { *value } else { min },
                            if *value > max { *value } else { max },
                        )
                    });
            let phase_minmax = (o2p * opd_minmax.0, o2p * opd_minmax.1);
            // println!("𝜑 minmax: {:?}", phase_minmax);
            let kl_coef = 1e-2 * kl_a0 as f32 / phase_minmax.0.abs().max(phase_minmax.1);
            // println!("KL coef.:{:e}", kl_coef);

            gmt.m2_modes_ij(sid - 1, kl_mode, kl_coef as f64);
            src.through(&mut gmt).xpupil().through(self);
            let slopes_push = Slopes::from((&quad_cell, &*self));
            self.reset();

            gmt.m2_modes_ij(sid - 1, kl_mode, -kl_coef as f64);
            src.through(&mut gmt).xpupil().through(self);
            let slopes_pull = Slopes::from((&quad_cell, &*self));
            self.reset();

            slopes.push((slopes_push - slopes_pull) / (2. * kl_coef));
            // slopes.push(slopes_push / kl_coef);
        }
        pb.as_ref().map(|pb| pb.finish());
        (quad_cell, slopes).into()*/
        todo!()
    }
    fn frame(&self) -> Frame<f32> {
        Frame {
            resolution: self.camera_resolution(),
            value: self.frame(),
        }
    }
}

impl From<(&DataRef, &Pyramid)> for Slopes {
    /// Computes the pyramid measurements
    ///
    /// The pyramid detector frame is contained within [Pyramid] and [DataRef] provides the
    /// optional frame mask  and measurements of reference
    fn from((qc, pym): (&DataRef, &Pyramid)) -> Self {
        let (sx, sy, a) = {
            let (n, m) = pym.camera_resolution();
            let LensletArray { n_side_lenslet, .. } = pym.lenslet_array;
            let n0 = n_side_lenslet / 2;
            let n1 = n0 + n / 2;
            let mat: Mat = nalgebra::DMatrix::from_column_slice(n, m, &pym.frame());
            let row_diff = mat.rows(n0, n_side_lenslet) - mat.rows(n1, n_side_lenslet);
            let sx = row_diff.columns(n0, n_side_lenslet) + row_diff.columns(n1, n_side_lenslet);
            let col_diff = mat.columns(n0, n_side_lenslet) - mat.columns(n1, n_side_lenslet);
            let sy = col_diff.rows(n0, n_side_lenslet) + col_diff.rows(n1, n_side_lenslet);

            let row_sum = mat.rows(n0, n_side_lenslet) + mat.rows(n1, n_side_lenslet);
            let a = row_sum.columns(n0, n_side_lenslet) + row_sum.columns(n1, n_side_lenslet);
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

type V = nalgebra::DVector<f32>;

impl Mul<&Pyramid> for &SlopesArray {
    type Output = Option<Vec<f32>>;
    /// Multiplies the pseudo-inverse of the calibration matrix with the [Pyramid] measurements
    fn mul(self, pym: &Pyramid) -> Self::Output {
        let slopes = Slopes::from((&self.data_ref, pym));
        self.inverse
            .as_ref()
            .map(|pinv| pinv * V::from(slopes))
            .map(|x| x.as_slice().to_vec())
    }
}
impl Mul<&Pyramid> for &Calibration {
    type Output = Option<Vec<f32>>;
    /// Multiplies the pseudo-inverse of the calibration matrix with the [Pyramid] measurements
    fn mul(self, wfs: &Pyramid) -> Self::Output {
        // let modes: Vec<_> = self.iter().flat_map(|x| x * wfs).flatten().collect();
        let modes = wfs
            .piston_sensor
            .as_ref()
            .map(|piston_sensor| piston_sensor.piston(wfs))
            .map_or_else(
                || {
                    self.iter()
                        .flat_map(|x| x * wfs)
                        .flat_map(|mut x| {
                            x.insert(0, 0f32);
                            x
                        })
                        .collect()
                },
                |piston| {
                    piston
                        .into_iter()
                        .chain(Some(0f32))
                        .zip(self.iter().flat_map(|x| x * wfs))
                        .flat_map(|(piston, modes)| {
                            let mut piston_modes = vec![piston];
                            piston_modes.extend_from_slice(&modes);
                            piston_modes
                        })
                        .collect::<Vec<f32>>()
                },
            );
        Some(modes)
    }
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Segment {
    sid: u8,
    n_mode: usize,
    mask: DMatrix<bool>,
    calibration: DMatrix<f32>,
}
impl Display for Segment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Segment #{} with {} modes, calibration: {:?}, mask: {:?}",
            self.sid,
            self.n_mode,
            self.calibration.shape(),
            self.mask.shape(),
        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Mirror {
    segments: Vec<Segment>,
    piston_mask: (Vec<bool>, Vec<bool>),
}
impl Display for Mirror {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for segment in &self.segments {
            writeln!(f, "{segment}")?;
        }
        writeln!(
            f,
            "Piston mask: [{},{}]",
            self.piston_mask.0.len(),
            self.piston_mask.1.len()
        )
    }
}

pub struct PyramidCalibration {
    h_filter: Vec<bool>,
    p_filter: Vec<bool>,
    offset: Vec<f32>,
    estimator: DMatrix<f32>,
}
impl PyramidCalibration {
    pub fn new(
        (sx0, sy0): (DMatrix<f32>, DMatrix<f32>),
        calibration: PathBuf,
        estimation: PathBuf,
    ) -> Result<Self, Box<dyn Error>> {
        let mirror: Mirror =
            serde_pickle::from_reader(&File::open(calibration)?, Default::default())?;

        let cum_mask = mirror.segments.iter().skip(1).fold(
            mirror.segments[0].mask.clone_owned(),
            |mut mask, segment| {
                mask.iter_mut()
                    .zip(segment.mask.iter())
                    .for_each(|(m1, mi)| *m1 = *m1 || *mi);
                mask
            },
        );
        let h_filter: Vec<_> = cum_mask.into_iter().cloned().collect();
        let p_filter: Vec<_> = mirror
            .piston_mask
            .0
            .iter()
            .chain(mirror.piston_mask.1.iter())
            .cloned()
            .collect();

        let offset: Vec<_> = sx0
            .iter()
            .chain(sy0.iter())
            .zip(h_filter.iter().cycle())
            .filter_map(|(s, f)| f.then_some(*s))
            .chain(
                sx0.iter()
                    .chain(sy0.iter())
                    .zip(p_filter.iter())
                    .filter_map(|(s, f)| f.then_some(*s)),
            )
            .collect();

        let estimator: DMatrix<f32> =
            serde_pickle::from_reader(&File::open(estimation)?, Default::default())?;

        Ok(Self {
            h_filter,
            p_filter,
            offset,
            estimator,
        })
    }
}

impl Mul<&Pyramid> for &PyramidCalibration {
    type Output = Option<Vec<f32>>;

    fn mul(self, pym: &Pyramid) -> Self::Output {
        let (sx, sy) = pym.processing();
        let sxy: Vec<_> = sx
            .iter()
            .chain(sy.iter())
            .zip(self.h_filter.iter().cycle())
            .filter_map(|(s, f)| f.then_some(*s))
            .chain(
                sx.iter()
                    .chain(sy.iter())
                    .zip(&self.p_filter)
                    .filter_map(|(s, f)| f.then_some(*s)),
            )
            .zip(&self.offset)
            .map(|(s, s0)| s - *s0)
            .collect();
        let c = (&self.estimator * DVector::from_column_slice(&sxy))
            .as_slice()
            .to_vec();
        Some(c)
    }
}
