use serde::{Deserialize, Serialize};

use crate::{Builder, SegmentWiseSensorBuilder};

use super::DifferentialPistonSensor;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DifferentialPistonSensorBuilder {
    pupil_sampling: usize,
    wrapping: Option<f64>,
    pub(super) n_gs: i32,
}
impl Default for DifferentialPistonSensorBuilder {
    fn default() -> Self {
        Self {
            pupil_sampling: Default::default(),
            wrapping: Default::default(),
            n_gs: 1,
        }
    }
}
impl DifferentialPistonSensorBuilder {
    pub fn pupil_sampling(mut self, pupil_sampling: usize) -> Self {
        self.pupil_sampling = pupil_sampling;
        self
    }
    pub fn wrapping(mut self, wrapping: f64) -> Self {
        self.wrapping = Some(wrapping);
        self
    }
    pub fn size(mut self, n: usize) -> Self {
        self.n_gs = n as i32;
        self
    }
}

impl SegmentWiseSensorBuilder for DifferentialPistonSensorBuilder {
    fn pupil_sampling(&self) -> usize {
        self.pupil_sampling
    }
}

impl Builder for DifferentialPistonSensorBuilder {
    type Component = DifferentialPistonSensor;

    fn build(self) -> crate::Result<Self::Component> {
        Ok(DifferentialPistonSensor {
            data: vec![0f32; 12 * self.n_gs as usize],
            pupil_sampling: self.pupil_sampling,
            wrapping: self.wrapping,
            n_frame: 0,
        })
    }
}
