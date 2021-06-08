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
///
/// # Examples
///
/// ```
/// use ceo::{Builder, SH48, Geometric};
/// let mut wfs = SH48::<Geometric>::new().build();
/// ```
use super::{Model, ShackHartmann, WavefrontSensorBuilder};
use crate::{imaging::NoiseDataSheet, Builder, Result, SHACKHARTMANN, SOURCE};

#[derive(Debug, Clone)]
pub struct SH48<T: Model>(SHACKHARTMANN<T>);

impl<T: Model> Default for SH48<T> {
    fn default() -> Self {
        SH48(
            SHACKHARTMANN::new()
                .n_sensor(4)
                .lenslet_array(48, 16, 25.5 / 48.0)
                .detector(8, Some(24), Some(2), None),
        )
    }
}
impl<T: Model> SH48<T> {
    pub fn n_sensor(self, n_sensor: usize) -> Self {
        Self(self.0.n_sensor(n_sensor))
    }
}
impl<T: Model> WavefrontSensorBuilder for SH48<T> {
    fn guide_stars(&self, template: Option<SOURCE>) -> SOURCE {
        self.0.guide_stars(template)
    }

    fn detector_noise_specs(self, noise_specs: NoiseDataSheet) -> Self {
        Self(self.0.detector_noise_specs(noise_specs))
    }
}
impl<T: Model> Builder for SH48<T> {
    type Component = ShackHartmann<T>;
    fn build(self) -> Result<ShackHartmann<T>> {
        self.0.build()
    }
}
