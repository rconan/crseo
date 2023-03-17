use std::ops::Mul;

use ffi::dev2host_int;

use crate::{Builder, FromBuilder, Gmt, Propagation, SegmentWiseSensor, WavefrontSensor};

use super::{
    data_processing::{Calibration, DataRef, Slopes, SlopesArray},
    PistonSensorBuilder,
};

#[derive(Debug, Default, Clone)]
pub struct PistonSensor {
    pub(super) data: Vec<f32>,
    pub(super) pupil_sampling: usize,
    pub(super) wrapping: Option<f64>,
    pub(super) n_frame: usize,
}
impl PistonSensor {
    pub fn data(&self) -> Vec<f32> {
        self.data.clone()
    }
}
impl FromBuilder for PistonSensor {
    type ComponentBuilder = PistonSensorBuilder;
}
impl Propagation for PistonSensor {
    fn propagate(&mut self, src: &mut crate::Source) {
        let n_ray_total = src.as_raw_mut_ptr().rays.N_RAY_TOTAL as usize;
        let n_ray = n_ray_total / src.size as usize;
        let mut mask = vec![0i32; n_ray_total];
        unsafe {
            dev2host_int(
                mask.as_mut_ptr(),
                src.as_raw_mut_ptr().rays.d__piston_mask,
                n_ray_total as i32,
            );
        }
        src.phase();
        for (mask, phase) in mask.chunks(n_ray).zip(src._phase.chunks(n_ray)) {
            for k in 1..8 {
                let segment_phase = mask
                    .iter()
                    .zip(phase)
                    .filter_map(|(&mask, &phase)| (mask == k).then_some(phase))
                    .collect::<Vec<f32>>();
                let n = segment_phase.len();
                if n > 0 {
                    let mean = segment_phase.iter().sum::<f32>() / n as f32;
                    if let Some(lim) = self.wrapping {
                        self.data[k as usize - 1] += mean % lim as f32;
                    } else {
                        self.data[k as usize - 1] += mean;
                    }
                } /*                 let var = segment_phase
                      .iter()
                      .map(|x| (x - mean).powi(2))
                      .sum::<f32>()
                      / n;
                  segment_wfe.push((mean as f64, var.sqrt() as f64)); */
            }
        }
        self.n_frame += 1;
        // let p7 = self.data[6];
        // self.data.iter_mut().for_each(|p| *p -= p7);
    }

    fn time_propagate(&mut self, _secs: f64, _src: &mut crate::Source) {
        todo!()
    }
}

impl SegmentWiseSensor for PistonSensor {
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

        for kl_mode in 0..n_mode {
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
        self.pupil_sampling
    }

    fn zeroed_segment(&mut self, sid: usize, src_builder: Option<crate::SourceBuilder>) -> DataRef {
        let mut gmt = Gmt::builder().build().unwrap();
        gmt.keep(&[sid as i32]);

        // let mut src = src_builder.clone().unwrap_or_default().build().unwrap();
        // src.through(&mut gmt).xpupil();
        // let n = src.pupil_sampling as usize;

        let mut pupil = nalgebra::DMatrix::<f32>::zeros(7, 1);
        pupil[(sid - 1, 0)] = 1f32;

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
}

impl From<(&DataRef, &PistonSensor)> for Slopes {
    fn from((data_ref, wfs): (&DataRef, &PistonSensor)) -> Self {
        let data = wfs
            .data()
            .into_iter()
            .map(|x| {
                if wfs.n_frame > 0 {
                    x / wfs.n_frame as f32
                } else {
                    x
                }
            })
            .collect::<Vec<f32>>();
        let mut sxy: Vec<_> = if let Some(mask) = data_ref.mask.as_ref() {
            data.into_iter()
                .zip(mask)
                .filter(|(_, &m)| m)
                .map(|(data, _)| data)
                .collect()
        } else {
            data
        };
        if let Some(Slopes(sxy0)) = data_ref.sxy0.as_ref() {
            sxy.iter_mut()
                .zip(sxy0)
                .for_each(|(sxy, sxy0)| *sxy -= *sxy0);
        }
        Slopes(sxy)
    }
}

type V = nalgebra::DVector<f32>;

impl Mul<&PistonSensor> for &SlopesArray {
    type Output = Option<Vec<f32>>;
    /// Multiplies the pseudo-inverse of the calibration matrix with the [PistonSensor] measurements
    fn mul(self, wfs: &PistonSensor) -> Self::Output {
        let slopes = Slopes::from((&self.data_ref, wfs));
        self.inverse
            .as_ref()
            .map(|pinv| pinv * V::from(slopes))
            .map(|x| x.as_slice().to_vec())
    }
}
impl Mul<&PistonSensor> for &Calibration {
    type Output = Option<Vec<f32>>;
    /// Multiplies the pseudo-inverse of the calibration matrix with the [PistonSensor] measurements
    fn mul(self, wfs: &PistonSensor) -> Self::Output {
        Some(self.iter().flat_map(|x| x * wfs).flatten().collect())
    }
}
