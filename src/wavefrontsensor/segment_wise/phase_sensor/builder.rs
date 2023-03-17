use serde::{Deserialize, Serialize};

use crate::{
    wavefrontsensor::{Calibration, GeomShackBuilder, PistonSensorBuilder},
    Builder, SegmentWiseSensorBuilder,
};

use super::PhaseSensor;

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct PhaseSensorBuilder {
    pub(super) geom_shack_builder: GeomShackBuilder,
    pub(super) piston_sensor_builder: PistonSensorBuilder,
}
impl PhaseSensorBuilder {
    pub fn lenslet(self, n_side_lenslet: usize, n_px_lenslet: usize) -> Self {
        Self {
            geom_shack_builder: self
                .geom_shack_builder
                .lenslet(n_side_lenslet, n_px_lenslet),
            ..self
        }
    }
    pub fn wrapping(self, wrapping: f64) -> Self {
        Self {
            piston_sensor_builder: self.piston_sensor_builder.wrapping(wrapping),
            ..self
        }
    }
}
impl SegmentWiseSensorBuilder for PhaseSensorBuilder {
    fn pupil_sampling(&self) -> usize {
        self.geom_shack_builder.pupil_sampling()
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
        let piston_sensor = self
            .piston_sensor_builder
            .pupil_sampling(self.geom_shack_builder.pupil_sampling());
        let piston_calibration = piston_sensor.calibrate(segment_piston, src.clone());
        let slopes_calibration = self.geom_shack_builder.calibrate(segment_slopes, src);
        piston_calibration + slopes_calibration
    }
}

impl Builder for PhaseSensorBuilder {
    type Component = PhaseSensor;

    fn build(self) -> crate::Result<Self::Component> {
        Ok(PhaseSensor {
            geom_shack: self.geom_shack_builder.build().unwrap(),
            piston_sensor: self
                .piston_sensor_builder
                .pupil_sampling(self.geom_shack_builder.pupil_sampling())
                .build()
                .unwrap(),
        })
    }
}
