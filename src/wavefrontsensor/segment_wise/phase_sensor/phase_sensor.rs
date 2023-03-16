use std::ops::Mul;

use indicatif::ProgressBar;

use crate::{
    wavefrontsensor::{GeomShack, PistonSensor},
    FromBuilder, Propagation, SegmentWiseSensor, SourceBuilder,
};

use super::{
    data_processing::{Calibration, DataRef, Slopes, SlopesArray},
    PhaseSensorBuilder,
};

/// Wrapper to CEO geometric ShackHartmann
pub struct PhaseSensor {
    pub(super) geom_shack: GeomShack,
    pub(super) piston_sensor: PistonSensor,
}

impl FromBuilder for PhaseSensor {
    type ComponentBuilder = PhaseSensorBuilder;
}
impl Propagation for PhaseSensor {
    fn propagate(&mut self, src: &mut crate::Source) {
        self.geom_shack.propagate(src);
        self.piston_sensor.propagate(src);
    }

    fn time_propagate(&mut self, _secs: f64, _src: &mut crate::Source) {
        todo!()
    }
}

impl SegmentWiseSensor for PhaseSensor {
    fn pupil_sampling(&self) -> usize {
        self.geom_shack.pupil_sampling()
    }

    fn calibrate_segment(
        &mut self,
        _src: Option<SourceBuilder>,
        _sid: usize,
        _n_mode: usize,
        _pb: Option<ProgressBar>,
    ) -> SlopesArray {
        todo!()
    }

    fn zeroed_segment(&mut self, _sid: usize, _src: Option<SourceBuilder>) -> DataRef {
        todo!()
    }

    fn into_slopes(&self, _data_ref: &DataRef) -> Slopes {
        todo!()
    }

    fn transform(&self, calib: &Calibration) -> Option<Vec<f32>> {
        Some(
            calib
                .iter()
                .take(calib.len() / 2)
                .map(|c| (c * &self.piston_sensor).unwrap())
                .zip(
                    calib
                        .iter()
                        .skip(calib.len() / 2)
                        .map(|c| (c * &self.geom_shack).unwrap()),
                )
                .flat_map(|(p, a)| p.into_iter().chain(a.into_iter()).collect::<Vec<f32>>())
                .collect(),
        )
    }
}

impl Mul<&PhaseSensor> for &Calibration {
    type Output = Option<Vec<f32>>;
    /// Multiplies the pseudo-inverse of the calibration matrix with the [PhaseSensor] measurements
    fn mul(self, wfs: &PhaseSensor) -> Self::Output {
        Some(
            self.iter()
                .take(self.len() / 2)
                .map(|c| (c * &wfs.piston_sensor).unwrap())
                .zip(
                    self.iter()
                        .skip(self.len() / 2)
                        .map(|c| (c * &wfs.geom_shack).unwrap()),
                )
                .flat_map(|(p, a)| p.into_iter().chain(a.into_iter()).collect::<Vec<f32>>())
                .collect(),
        )
    }
}
