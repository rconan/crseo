use std::ops::Mul;

use indicatif::ProgressBar;

use crate::wavefrontsensor::LensletArray;
use crate::{
    Builder, FromBuilder, Gmt, Propagation, SegmentWiseSensor, SourceBuilder, WavefrontSensor,
};

use super::data_processing::{Calibration, DataRef, Slopes, SlopesArray};
use super::{Modulation, PyramidBuilder};

type Mat = nalgebra::DMatrix<f32>;

/// Wrapper to CEO pyramid
pub struct Pyramid {
    pub(super) _c_: ffi::pyramid,
    pub(super) lenslet_array: LensletArray,
    pub(super) alpha: f32,
    pub(super) modulation: Option<Modulation>,
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
    pub fn frame(&self) -> Vec<f32> {
        let n = self._c_.camera.N_PX_CAMERA.pow(2) * self._c_.camera.N_SOURCE;
        let mut frame = vec![0f32; n as usize];
        unsafe {
            ffi::dev2host(frame.as_mut_ptr(), self._c_.camera.d__frame, n);
        }
        frame
    }

    #[inline]
    pub fn n_px_camera(&self) -> usize {
        self._c_.camera.N_PX_CAMERA as usize
    }
    pub fn camera_resolution(&self) -> (usize, usize) {
        (self.n_px_camera(), self.n_px_camera())
    }
    pub fn data(&mut self) -> (Mat, Mat) {
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
        (row_col_data, col_row_data)
    }
    pub fn add_quads(&mut self) -> Mat {
        let (n, m) = self.camera_resolution();
        let LensletArray { n_side_lenslet, .. } = self.lenslet_array;
        let n0 = n_side_lenslet / 2;
        let n1 = n0 + n / 2;
        let mat: Mat = nalgebra::DMatrix::from_column_slice(n, m, &self.frame());
        let row_diff = mat.rows(n0, n_side_lenslet) + mat.rows(n1, n_side_lenslet);
        row_diff.columns(n0, n_side_lenslet) + row_diff.columns(n1, n_side_lenslet)
    }
}

impl SegmentWiseSensor for Pyramid {
    fn pupil_sampling(&self) -> usize {
        let LensletArray {
            n_side_lenslet,
            n_px_lenslet,
            ..
        } = self.lenslet_array;
        n_side_lenslet * n_px_lenslet + 1
    }
    fn zeroed_segment(&mut self, sid: usize, src_builder: Option<SourceBuilder>) -> DataRef {
        let LensletArray { n_side_lenslet, .. } = self.lenslet_array;
        // Setting the pyramid mask restricted to the segment
        let mut gmt = Gmt::builder().build().unwrap();
        gmt.keep(&[sid as i32]);
        let mut src = src_builder
            .clone()
            .unwrap_or_default()
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
            .unwrap_or_default()
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
        src_builder: Option<SourceBuilder>,
        sid: usize,
        n_mode: usize,
        pb: Option<ProgressBar>,
    ) -> SlopesArray {
        let quad_cell = self.zeroed_segment(sid, src_builder.clone());

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
        (quad_cell, slopes).into()
    }
}

impl From<(&DataRef, &Pyramid)> for Slopes {
    /// Computes the pyramid measurements
    ///
    /// The pyramid detector frame is contained within [Pyramid] and [QuadCell] provides the
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
        Some(self.iter().flat_map(|x| x * wfs).flatten().collect())
    }
}
