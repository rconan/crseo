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
    fn data(&mut self) -> Vec<f32> {
        self.pistons
            .iter()
            .zip(&self.piston_refs)
            .map(|(x, x0)| (*x - *x0) as f32)
            .collect()
    }
}

impl Propagation for PistonSensor {
    fn propagate(&mut self, src: &mut crate::Source) -> &mut Self {
        self.pistons = src.segment_piston();
        self
    }

    fn time_propagate(&mut self, _secs: f64, src: &mut crate::Source) -> &mut Self {
        self.propagate(src)
    }
}
