use std::ops::Mul;

use indicatif::ProgressBar;

use crate::{
    cu::Single, wavefrontsensor::LensletArray, Builder, Cu, FromBuilder, Gmt, Propagation,
    SegmentWiseSensor, SourceBuilder, WavefrontSensor,
};

use super::{
    data_processing::{Calibration, DataRef, Slopes, SlopesArray},
    GeomShackBuilder,
};

/// Wrapper to CEO geometric ShackHartmann
pub struct GeomShack {
    pub(super) _c_: ffi::geometricShackHartmann,
    pub(super) lenslet_array: LensletArray,
    pub(super) n_gs: usize,
}
impl Drop for GeomShack {
    /// Frees CEO memory before dropping `GeomShack`
    fn drop(&mut self) {
        unsafe {
            self._c_.cleanup();
        }
    }
}
impl FromBuilder for GeomShack {
    type ComponentBuilder = GeomShackBuilder;
}
impl Propagation for GeomShack {
    fn propagate(&mut self, src: &mut crate::Source) {
        unsafe {
            self._c_.propagate(src.as_raw_mut_ptr());
        }
    }

    fn time_propagate(&mut self, _secs: f64, _src: &mut crate::Source) {
        todo!()
    }
}

impl GeomShack {
    pub fn n_total_lenslet(&self) -> usize {
        let LensletArray { n_side_lenslet, .. } = self.lenslet_array;
        n_side_lenslet * n_side_lenslet * self.n_gs
    }
    pub fn data(&self) -> Vec<f32> {
        let mut data = Cu::<Single>::vector(self.n_total_lenslet() * 2);
        data.from_ptr(self._c_.data_proc.d__c);
        data.from_dev()
    }
}

impl SegmentWiseSensor for GeomShack {
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
        // Setting the WFS mask restricted to the segment
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
            src.amplitude().into_iter(),
        );

        let mut data_ref = DataRef::new(pupil);

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

impl From<(&DataRef, &GeomShack)> for Slopes {
    /// Computes the pyramid measurements
    ///
    /// The pyramid detector frame is contained within [Pyramid] and [QuadCell] provides the
    /// optional frame mask  and measurements of reference
    fn from((qc, wfs): (&DataRef, &GeomShack)) -> Self {
        let data = wfs.data();
        let (sx, sy) = data.split_at(wfs.lenslet_array.n_side_lenslet.pow(2));
        let iter = sx.iter().zip(sy);
        let mut sxy: Vec<_> = if let Some(mask) = qc.mask.as_ref() {
            iter.zip(mask)
                .filter(|(_, &m)| m)
                .flat_map(|((sx, sy), _)| vec![*sx, *sy])
                .collect()
        } else {
            iter.flat_map(|(sx, sy)| vec![*sx, *sy]).collect()
        };
        if let Some(Slopes(sxy0)) = qc.sxy0.as_ref() {
            sxy.iter_mut()
                .zip(sxy0)
                .for_each(|(sxy, sxy0)| *sxy -= *sxy0);
        }
        Slopes(sxy)
    }
}

type V = nalgebra::DVector<f32>;

impl Mul<&GeomShack> for &SlopesArray {
    type Output = Option<Vec<f32>>;
    /// Multiplies the pseudo-inverse of the calibration matrix with the [Pyramid] measurements
    fn mul(self, pym: &GeomShack) -> Self::Output {
        let slopes = Slopes::from((&self.data_ref, pym));
        self.inverse
            .as_ref()
            .map(|pinv| pinv * V::from(slopes))
            .map(|x| x.as_slice().to_vec())
    }
}

impl Mul<&GeomShack> for &Calibration {
    type Output = Option<Vec<f32>>;
    /// Multiplies the pseudo-inverse of the calibration matrix with the [Pyramid] measurements
    fn mul(self, wfs: &GeomShack) -> Self::Output {
        Some(self.iter().flat_map(|x| x * wfs).flatten().collect())
    }
}
/* impl<'a, T> Mul<&'a Box<T>> for &'a Calibration
where
    &'a Calibration: Mul<&'a T>,
{
    type Output = Option<Vec<f32>>;

    fn mul(self, rhs: &'a Box<T>) -> Self::Output {
        self * (**rhs)
    }
} */
