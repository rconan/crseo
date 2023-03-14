use serde::{Deserialize, Serialize};

use crate::{Builder, SegmentWiseSensorBuilder};

use super::PhaseSensor;

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct PhaseSensorBuilder;

impl SegmentWiseSensorBuilder for PhaseSensorBuilder {}

impl Builder for PhaseSensorBuilder {
    type Component = PhaseSensor;

    fn build(self) -> crate::Result<Self::Component> {
        Ok(Default::default())
    }
}
