use std::ops::Mul;

use ffi::dev2host_int;

use crate::{Builder, FromBuilder, Gmt, Propagation, SegmentWiseSensor, WavefrontSensor};

use super::{
    data_processing::{Calibration, DataRef, Slopes, SlopesArray},
    DifferentialPistonSensorBuilder,
};

#[derive(Debug, Default, Clone)]
pub struct DifferentialPistonSensor {
    pub(super) data: Vec<f32>,
    pub(super) pupil_sampling: usize,
    #[allow(dead_code)]
    pub(super) wrapping: Option<f64>,
    pub(super) n_frame: usize,
}
impl DifferentialPistonSensor {
    pub fn data(&self) -> Vec<f32> {
        self.data.clone()
    }
}
impl FromBuilder for DifferentialPistonSensor {
    type ComponentBuilder = DifferentialPistonSensorBuilder;
}
impl Propagation for DifferentialPistonSensor {
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
        let mut piston = vec![];
        for (mask, phase) in mask.chunks(n_ray).zip(src._phase.chunks(n_ray)) {
            for k in 1..8 {
                let segment_phase = mask
                    .iter()
                    .zip(phase)
                    .filter_map(|(&mask, &phase)| (mask == k).then_some(phase))
                    .collect::<Vec<f32>>();
                let n = segment_phase.len();
                let mean = segment_phase.iter().sum::<f32>() / n as f32;
                piston.push(mean);
            }
        }

        for (piston, data) in piston.chunks(7).zip(self.data.chunks_mut(12)) {
            let p7 = piston[6];
            data.iter_mut()
                .zip(piston.iter().take(6))
                .for_each(|(d, &p)| *d += p7 - p);
            data.iter_mut()
                .skip(6)
                .take(5)
                .zip(piston.windows(2))
                .for_each(|(d, p)| *d += p[0] - p[1]);
            data[11] += piston[5] - piston[0];
        }

        /*         let p7 = data[6];
        self.data
            .iter_mut()
            .zip(data.iter().take(6))
            .for_each(|(d, &p)| *d = p7 - p);
        self.data
            .iter_mut()
            .skip(6)
            .take(5)
            .zip(data.windows(2))
            .for_each(|(d, p)| *d = p[0] - p[1]);
        self.data[11] = data[5] - data[0]; */
        self.n_frame += 1;
    }

    fn time_propagate(&mut self, _secs: f64, _src: &mut crate::Source) {
        todo!()
    }
}

impl SegmentWiseSensor for DifferentialPistonSensor {
    fn calibrate_segment(
        &mut self,
        _src_builder: Option<crate::source::SourceBuilder>,
        _sid: usize,
        _n_mode: usize,
        _pb: Option<indicatif::ProgressBar>,
    ) -> SlopesArray {
        /*  let data_ref = self.zeroed_segment(sid, src_builder.clone());

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

        (data_ref, slopes).into() */
        todo!()
    }

    fn pupil_sampling(&self) -> usize {
        self.pupil_sampling
    }

    fn zeroed_segment(
        &mut self,
        _sid: usize,
        src_builder: Option<crate::source::SourceBuilder>,
    ) -> DataRef {
        let mut gmt = Gmt::builder().build().unwrap();
        // gmt.keep(&[sid as i32]);

        // let mut src = src_builder.clone().unwrap_or_default().build().unwrap();
        // src.through(&mut gmt).xpupil();
        // let n = src.pupil_sampling as usize;

        let mut pupil = nalgebra::DMatrix::<f32>::zeros(7, 1);
        pupil.fill(1f32);
        // pupil[(sid - 1, 0)] = 1f32;

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

impl From<(&DataRef, &DifferentialPistonSensor)> for Slopes {
    fn from((data_ref, wfs): (&DataRef, &DifferentialPistonSensor)) -> Self {
        let mut sxy = wfs
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
        // dbg!(&data);
        /*         let mut sxy: Vec<_> = if let Some(mask) = data_ref.mask.as_ref() {
            data.into_iter()
                .zip(mask)
                .filter(|(_, &m)| m)
                .map(|(data, _)| data)
                .collect()
        } else {
            data
        }; */
        if let Some(Slopes(sxy0)) = data_ref.sxy0.as_ref() {
            sxy.iter_mut()
                .zip(sxy0)
                .for_each(|(sxy, sxy0)| *sxy -= *sxy0);
        }
        Slopes(sxy)
    }
}

type V = nalgebra::DVector<f32>;

impl Mul<&DifferentialPistonSensor> for &SlopesArray {
    type Output = Option<Vec<f32>>;
    /// Multiplies the pseudo-inverse of the calibration matrix with the [DifferentialPistonSensor] measurements
    fn mul(self, wfs: &DifferentialPistonSensor) -> Self::Output {
        let slopes = Slopes::from((&self.data_ref, wfs));
        self.inverse
            .as_ref()
            .map(|pinv| pinv * V::from(slopes))
            .map(|x| x.as_slice().to_vec())
    }
}
impl Mul<&DifferentialPistonSensor> for &Calibration {
    type Output = Option<Vec<f32>>;
    /// Multiplies the pseudo-inverse of the calibration matrix with the [DifferentialPistonSensor] measurements
    fn mul(self, wfs: &DifferentialPistonSensor) -> Self::Output {
        Some(self.iter().flat_map(|x| x * wfs).flatten().collect())
    }
}

/* #[cfg(test)]
mod tests {
    use crate::{
        wavefrontsensor::SegmentCalibration, SegmentWiseSensorBuilder, Source,
        WavefrontSensorBuilder,
    };

    use super::*;

    #[test]
    fn piston() {
        let mut gmt = Gmt::builder().build().unwrap();
        let n_gs = 3;
        let mut sensor = DifferentialPistonSensor::builder()
            .size(n_gs)
            .pupil_sampling(401)
            .build()
            .unwrap();
        let mut src = Source::builder()
            .pupil_sampling(401)
            .size(n_gs)
            .build()
            .unwrap();

        let sensor_zero =
            sensor.zeroed_segment(0, Some(Source::builder().pupil_sampling(401).size(n_gs)));

        dbg!(&sensor_zero);

        src.through(&mut gmt).xpupil();
        let piston: Vec<_> = (1..=7).map(|i| (i as f32) * 1e-7).collect();
        src.add_piston(&piston);

        let pe = src.segment_piston();
        dbg!(pe);

        src.through(&mut sensor);
        dbg!(sensor.data());

        let data: Vec<_> = Slopes::from((&sensor_zero, &sensor)).into();
        dbg!(&data);
    }

    #[test]
    fn calibrate_segment_rbm() {
        let builder = DifferentialPistonSensor::builder().pupil_sampling(48 * 8);
        let src_builder = builder.guide_stars(Some(Source::builder()));
        let mut wfs = builder.build().unwrap();
        let seg = SegmentCalibration::rbm("TRxyz", "M1").keep_all();
        let mut c = seg.calibrate(3, &mut wfs, src_builder, None);
        dbg!(&c);
        println!("{:.6}", c.interaction_matrix());
        c.pseudo_inverse(None).unwrap();
    }

    #[test]
    fn calibrate_rbm() {
        let builder = DifferentialPistonSensor::builder().pupil_sampling(48 * 8);
        let src_builder = builder.guide_stars(Some(Source::builder()));
        let mut dfs_calibration = builder
            .calibrate(
                SegmentCalibration::rbm("TRxyz", "M1").keep_all(),
                src_builder,
            )
            .flatten()
            .unwrap();
        println!("{:.1}", dfs_calibration.interaction_matrices()[0]);

        // dbg!(&dfs_calibration);
        /*         dfs_calibration
        .interaction_matrices()
        .iter()
        .enumerate()
        .for_each(|(k, c)| println!("Segment #{:}{:.6}", k + 1, c)); */
    }

    #[test]
    fn wrapping() {
        let mut gmt = Gmt::builder().build().unwrap();
        gmt.keep(&[1i32]);
        let mut src = Source::builder().pupil_sampling(401).build().unwrap();
        let w = src.wavelength() as f32;

        let mut sensor = DifferentialPistonSensor::builder()
            .pupil_sampling(401)
            .wrapping(0.5 * w as f64)
            .build()
            .unwrap();
        let sensor_zero = sensor.zeroed_segment(1, None);

        let mut piston = vec![0f32; 7];
        let mut ramp = vec![];
        for k in 0..10 {
            src.through(&mut gmt).xpupil();
            piston[0] = 0.1 * w * k as f32;
            src.add_piston(&piston);
            sensor.reset();
            src.through(&mut sensor);
            let data: Vec<f32> = Slopes::from((&sensor_zero, &sensor)).into();
            ramp.push(data[0] / w);
        }
        dbg!(ramp);
    }
}
 */