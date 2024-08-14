use std::ops::{Deref, DerefMut};

use super::Model;
use crate::{Builder, wavefrontsensor::ShackHartmannBuilder};

/// `ShackHartmann` "SH48" builder for GMT AGWS model
///
/// Default properties:
///  - n_sensor: 4
///  - lenslet_array:
///    - n_lenslet: 48
///    - n_px_lenslet: 16px
///    - lenslet_pitch: 25.5m/48
///  - detector:
///    - n_px_framelet: 8px
///    - n_px_imagelet: Some(24px)
///    - osf: Some(2)
#[derive(Debug, Clone)]
pub struct SH48<T: Model>(ShackHartmannBuilder<T>);

impl<T: Model> Deref for SH48<T> {
    type Target = ShackHartmannBuilder<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T: Model> DerefMut for SH48<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: Model> Default for SH48<T> {
    fn default() -> Self {
        SH48(
            ShackHartmannBuilder::new()
                .n_sensor(4)
                .lenslet_array(48, 16, 25.5 / 48.0)
                .detector(8, Some(24), 2, None),
        )
    }
}

impl<T: Model> SH48<T> {
    pub fn n_sensor(self, n_sensor: usize) -> Self {
        Self(self.0.n_sensor(n_sensor))
    }
}
