use serde::{Deserialize, Serialize};

use crate::{Builder, SegmentWiseSensorBuilder};

use super::PistonSensor;

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct PistonSensorBuilder {
    pupil_sampling: usize,
    wrapping: Option<f64>,
}
impl PistonSensorBuilder {
    pub fn pupil_sampling(mut self, pupil_sampling: usize) -> Self {
        self.pupil_sampling = pupil_sampling;
        self
    }
    pub fn wrapping(mut self, wrapping: f64) -> Self {
        self.wrapping = Some(wrapping);
        self
    }
}

impl SegmentWiseSensorBuilder for PistonSensorBuilder {
    fn pupil_sampling(&self) -> usize {
        self.pupil_sampling
    }
}

impl Builder for PistonSensorBuilder {
    type Component = PistonSensor;

    fn build(self) -> crate::Result<Self::Component> {
        Ok(PistonSensor {
            data: vec![0f32; 7],
            pupil_sampling: self.pupil_sampling,
            wrapping: self.wrapping,
            n_frame: 0,
        })
    }
}
