use std::ops::Mul;

use crate::{
    wavefrontsensor::{data_processing::DataRef, Calibration, Slopes, SlopesArray},
    Builder, FromBuilder, Gmt, Propagation, SegmentWiseSensor, WavefrontSensor,
};

pub use super::builder::PhaseSensorBuilder;

#[derive(Debug, Default, Clone)]
pub struct PhaseSensor {
    pub(super) data: Vec<f32>,
}
impl PhaseSensor {
    pub fn data(&self) -> Vec<f32> {
        self.data.clone()
    }
}
impl FromBuilder for PhaseSensor {
    type ComponentBuilder = PhaseSensorBuilder;
}
impl Propagation for PhaseSensor {
    fn propagate(&mut self, src: &mut crate::Source) {
        self.data = src.phase().clone()
    }

    fn time_propagate(&mut self, _secs: f64, _src: &mut crate::Source) {
        todo!()
    }
}

impl SegmentWiseSensor for PhaseSensor {
    fn calibrate_segment(
        &mut self,
        src_builder: Option<crate::SourceBuilder>,
        sid: usize,
        n_mode: usize,
        pb: Option<indicatif::ProgressBar>,
    ) -> SlopesArray {
        let data_ref = self.zeroed_segment(sid, src_builder.clone());

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
            let slopes_push = Slopes::from((&data_ref, &*self));
            self.reset();

            gmt.m2_modes_ij(sid - 1, kl_mode, -kl_coef as f64);
            src.through(&mut gmt).xpupil().through(self);
            let slopes_pull = Slopes::from((&data_ref, &*self));
            self.reset();

            slopes.push((slopes_push - slopes_pull) / (2. * kl_coef));
            // slopes.push(slopes_push / kl_coef);
        }
        pb.as_ref().map(|pb| pb.finish());
        (data_ref, slopes).into()
    }

    fn pupil_sampling(&self) -> usize {
        todo!()
    }

    fn zeroed_segment(&mut self, sid: usize, src_builder: Option<crate::SourceBuilder>) -> DataRef {
        let mut gmt = Gmt::builder().build().unwrap();
        gmt.keep(&[sid as i32]);
        let mut src = src_builder.clone().unwrap_or_default().build().unwrap();
        src.through(&mut gmt).xpupil();
        let n = src.pupil_sampling as usize;
        let pupil = nalgebra::DMatrix::<f32>::from_iterator(n, n, src.amplitude().into_iter());

        let mut data_ref = DataRef::new(pupil);

        let mut src = src_builder.clone().unwrap_or_default().build().unwrap();
        self.reset();
        src.through(&mut gmt).xpupil().through(self);
        data_ref.set_ref_with(Slopes::from((&data_ref, &*self)));
        self.reset();
        data_ref
    }
    fn into_slopes(&self, data_ref: &DataRef) -> Slopes {
        Slopes::from((data_ref, self))
    }
}

impl From<(&DataRef, &PhaseSensor)> for Slopes {
    fn from((data_ref, wfs): (&DataRef, &PhaseSensor)) -> Self {
        let mut data = wfs.data();
        if let Some(Slopes(sxy0)) = data_ref.sxy0.as_ref() {
            data.iter_mut()
                .zip(sxy0)
                .for_each(|(sxy, sxy0)| *sxy -= *sxy0);
        }
        Slopes(data)
    }
}

type V = nalgebra::DVector<f32>;

impl Mul<&PhaseSensor> for &SlopesArray {
    type Output = Option<Vec<f32>>;
    /// Multiplies the pseudo-inverse of the calibration matrix with the [Pyramid] measurements
    fn mul(self, wfs: &PhaseSensor) -> Self::Output {
        let slopes = Slopes::from((&self.data_ref, wfs));
        self.inverse
            .as_ref()
            .map(|pinv| pinv * V::from(slopes))
            .map(|x| x.as_slice().to_vec())
    }
}
impl Mul<&PhaseSensor> for &Calibration {
    type Output = Option<Vec<f32>>;
    /// Multiplies the pseudo-inverse of the calibration matrix with the [Pyramid] measurements
    fn mul(self, wfs: &PhaseSensor) -> Self::Output {
        Some(self.iter().flat_map(|x| x * wfs).flatten().collect())
    }
}
