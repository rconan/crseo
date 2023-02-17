mod builder;
pub use builder::GeomShackBuilder;
use indicatif::ProgressBar;

use crate::{cu::Single, Builder, Cu, FromBuilder, Gmt, Propagation, SegmentWiseSensor, Source};

use super::{
    pyramid::{Slopes, SlopesArray},
    LensletArray, QuadCell,
};

/// Wrapper to CEO geometric ShackHartmann
pub struct GeomShack {
    _c_: ffi::geometricShackHartmann,
    lenslet_array: LensletArray,
    n_gs: usize,
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
    pub fn reset(&mut self) {
        unsafe {
            self._c_.reset();
        }
    }
    pub fn n_total_lenslet(&self) -> usize {
        let LensletArray { n_side_lenslet, .. } = self.lenslet_array;
        n_side_lenslet * n_side_lenslet * self.n_gs
    }
    pub fn data(&self) -> Vec<f32> {
        let mut data = Cu::<Single>::vector(self.n_total_lenslet() * 2);
        data.from_ptr(self._c_.data_proc.d__c);
        data.from_dev()
    }
    pub fn pupil_sampling(&self) -> usize {
        let LensletArray {
            n_side_lenslet,
            n_px_lenslet,
            ..
        } = self.lenslet_array;
        n_side_lenslet * n_px_lenslet + 1
    }
}
impl SegmentWiseSensor for GeomShack {
    fn calibrate_segment(
        &mut self,
        sid: usize,
        n_mode: usize,
        pb: Option<ProgressBar>,
    ) -> SlopesArray {
        let LensletArray { n_side_lenslet, .. } = self.lenslet_array;

        // Setting the WFS mask restricted to the segment
        let mut gmt = Gmt::builder().build().unwrap();
        gmt.keep(&[sid as i32]);
        let mut src = Source::builder()
            .pupil_sampling(n_side_lenslet)
            .build()
            .unwrap();
        src.through(&mut gmt).xpupil();

        let pupil = nalgebra::DMatrix::<f32>::from_iterator(
            n_side_lenslet,
            n_side_lenslet,
            src.amplitude().into_iter(),
        );

        let mut quad_cell = QuadCell::new(pupil);

        let mut gmt = Gmt::builder().m2("Karhunen-Loeve", n_mode).build().unwrap();
        gmt.keep(&[sid as i32]);
        // let mut pym = Pyramid::builder().n_lenslet(n_lenslet).build().unwrap();
        let mut src = Source::builder()
            .pupil_sampling(self.pupil_sampling())
            .build()
            .unwrap();
        self.reset();
        src.through(&mut gmt).xpupil().through(self);
        quad_cell.set_ref_with(Slopes::from((&quad_cell, &*self)));
        self.reset();

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

impl From<(&QuadCell, &GeomShack)> for Slopes {
    /// Computes the pyramid measurements
    ///
    /// The pyramid detector frame is contained within [Pyramid] and [QuadCell] provides the
    /// optional frame mask  and measurements of reference
    fn from((qc, wfs): (&QuadCell, &GeomShack)) -> Self {
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

#[cfg(test)]
mod tests {
    use std::fs::File;

    use super::*;
    use crate::{FromBuilder, Gmt, Source};

    #[test]
    fn geom_shack() {
        let n_side_lenslet = 50;
        let mut gmt = Gmt::builder().build().unwrap();
        let mut wfs = GeomShack::builder()
            .lenslet(n_side_lenslet, 16)
            .build()
            .unwrap();
        let mut src = Source::builder()
            .pupil_sampling(wfs.pupil_sampling())
            .build()
            .unwrap();
        src.through(&mut gmt).xpupil().through(&mut wfs);

        let _: complot::Heatmap = (
            (
                src.phase().as_slice(),
                (wfs.pupil_sampling(), wfs.pupil_sampling()),
            ),
            Some(complot::Config::new().filename("phase.png")),
        )
            .into();

        let data = wfs.data();
        dbg!(data.len());
        serde_pickle::to_writer(
            &mut File::create("geom_shack_data.pkl").unwrap(),
            &data,
            Default::default(),
        )
        .unwrap();

        let calib = wfs.calibrate_segment(1, 15, None);
        dbg!(calib.shape());
        serde_pickle::to_writer(
            &mut File::create("geom_shack_calibration.pkl").unwrap(),
            &calib,
            Default::default(),
        )
        .unwrap();
    }
}
