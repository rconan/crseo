use crate::{Builder, Propagation, WavefrontSensor, WavefrontSensorBuilder};

#[derive(Default, Clone)]
pub struct PistonSensorBuilder {}

impl Builder for PistonSensorBuilder {
    type Component = PistonSensor;

    fn build(self) -> crate::Result<Self::Component> {
        Ok(Default::default())
    }
}

#[derive(Default)]
pub struct PistonSensor {
    piston_refs: Vec<f64>,
    pistons: Vec<f64>,
}

impl WavefrontSensorBuilder for PistonSensor {}

impl WavefrontSensor for PistonSensor {
    fn calibrate(&mut self, src: &mut crate::Source, _threshold: f64) {
        self.piston_refs = src.segment_piston();
    }
    fn data(&mut self) -> Vec<f64> {
        self.pistons
            .iter()
            .zip(&self.piston_refs)
            .map(|(x, x0)| (*x - *x0))
            .collect()
    }

    fn reset(&mut self) {
        todo!()
    }

    fn process(&mut self) {
        todo!()
    }

    fn readout(&mut self) {
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

    fn left_multiply(
        &self,
        _calibration: &crate::wavefrontsensor::Calibration,
    ) -> Option<Vec<f32>> {
        todo!()
    }
}

impl Propagation for PistonSensor {
    fn propagate(&mut self, src: &mut crate::Source) {
        self.pistons = src.segment_piston();
    }

    fn time_propagate(&mut self, _secs: f64, src: &mut crate::Source) {
        self.propagate(src)
    }
}
