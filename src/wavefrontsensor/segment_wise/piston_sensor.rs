mod builder;
pub use builder::PistonSensorBuilder;
mod piston_sensor;
pub use piston_sensor::PistonSensor;

pub use super::data_processing;
use crate::{WavefrontSensor, WavefrontSensorBuilder};

impl WavefrontSensorBuilder for PistonSensorBuilder {}

impl WavefrontSensor for PistonSensor {
    fn calibrate(&mut self, _src: &mut crate::Source, _threshold: f64) {
        todo!()
    }

    fn reset(&mut self) {
        self.data.fill(0f32);
        self.n_frame = 0;
    }

    fn process(&mut self) {
        todo!()
    }

    fn readout(&mut self) {
        todo!()
    }

    fn data(&mut self) -> Vec<f64> {
        todo!()
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

#[cfg(test)]
pub mod tests {
    use crate::{wavefrontsensor::SegmentCalibration, Builder, FromBuilder};

    use super::*;

    #[test]
    fn calibrate_kl() {
        let builder = PistonSensor::builder().pupil_sampling(92 * 4);
        let src_builder = builder.guide_stars(None);
        let mut wfs = builder.build().unwrap();
        let seg = SegmentCalibration::modes("Karhunen-Loeve", 0..1, "M2");
        let c = seg.calibrate(1, &mut wfs, src_builder, None);
        dbg!(&c);
    }

    #[test]
    fn calibrate_okl() {
        let builder = PistonSensor::builder().pupil_sampling(92 * 4);
        let src_builder = builder.guide_stars(None);
        let mut wfs = builder.build().unwrap();
        let seg = SegmentCalibration::modes("M2_OrthoNorm_KarhunenLoeveModes", 0..1, "M2");
        let c = seg.calibrate(1, &mut wfs, src_builder, None);
        dbg!(&c);
    }
}
