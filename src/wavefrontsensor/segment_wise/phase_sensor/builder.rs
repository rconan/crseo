use serde::{Deserialize, Serialize};

use crate::{
    wavefrontsensor::{Calibration, GeomShackBuilder, PistonSensor},
    Builder, FromBuilder, SegmentWiseSensorBuilder,
};

use super::PhaseSensor;

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct PhaseSensorBuilder(pub(super) GeomShackBuilder);
impl PhaseSensorBuilder {
    pub fn lenslet(self, n_side_lenslet: usize, n_px_lenslet: usize) -> Self {
        Self(self.0.lenslet(n_side_lenslet, n_px_lenslet))
    }
}
impl SegmentWiseSensorBuilder for PhaseSensorBuilder {
    fn pupil_sampling(&self) -> usize {
        self.0.pupil_sampling()
    }

    fn calibrate(
        self,
        segment: crate::wavefrontsensor::SegmentCalibration,
        src: crate::SourceBuilder,
    ) -> Calibration
    where
        Self::Component: crate::SegmentWiseSensor,
    {
        let (segment_piston, segment_slopes) = segment.slip_at(1).unwrap();
        let piston_sensor = PistonSensor::builder().pupil_sampling(self.0.pupil_sampling());
        let piston_calibration = piston_sensor.calibrate(segment_piston, src.clone());
        let slopes_calibration = self.0.calibrate(segment_slopes, src);
        piston_calibration + slopes_calibration
    }
}

impl Builder for PhaseSensorBuilder {
    type Component = PhaseSensor;

    fn build(self) -> crate::Result<Self::Component> {
        Ok(PhaseSensor {
            geom_shack: self.0.build().unwrap(),
            piston_sensor: PistonSensor::builder()
                .pupil_sampling(self.0.pupil_sampling())
                .build()
                .unwrap(),
        })
    }
}
