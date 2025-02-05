mod builder;
use crate::builders::source::SourceBuilder;
pub use builder::PhaseSensorBuilder;
mod phase_sensor;
pub use phase_sensor::PhaseSensor;

pub use super::data_processing;
use crate::{
    wavefrontsensor::{GeomShack, PistonSensor},
    SegmentWiseSensorBuilder, WavefrontSensor, WavefrontSensorBuilder,
};

impl WavefrontSensorBuilder for PhaseSensorBuilder {
    fn guide_stars(&self, gs: Option<SourceBuilder>) -> SourceBuilder {
        gs.unwrap_or_default()
            .pupil_sampling(self.geom_shack_builder.pupil_sampling())
    }
}

impl WavefrontSensor for PhaseSensor {
    fn calibrate(&mut self, _src: &mut crate::Source, _threshold: f64) {
        todo!()
    }

    fn reset(&mut self) {
        self.piston_sensor.reset();
        self.geom_shack.reset();
    }

    fn process(&mut self) {
        todo!()
    }

    fn readout(&mut self) {
        todo!()
    }

    fn data(&mut self) -> Vec<f64> {
        PistonSensor::data(&self.piston_sensor)
            .into_iter()
            .chain(GeomShack::data(&self.geom_shack).into_iter())
            .map(|x| x as f64)
            .collect()
    }

    fn frame(&self) -> Option<Vec<f32>> {
        todo!()
    }

    fn n_frame(&self) -> usize {
        todo!()
    }

    fn valid_lenslet_from(&mut self, _wfs: &mut dyn WavefrontSensor) {
        todo!()
    }

    fn valid_lenslet(&mut self) -> &mut ffi::mask {
        todo!()
    }

    fn n_valid_lenslet(&mut self) -> Vec<usize> {
        todo!()
    }

    fn left_multiply(&self, calibration: &super::Calibration) -> Option<Vec<f32>> {
        calibration * self
    }
}
