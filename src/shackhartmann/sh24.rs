use super::{Model, ShackHartmann, WavefrontSensorBuilder};
use crate::{imaging::NoiseDataSheet, Builder, Result, SHACKHARTMANN, SOURCE};

/// `ShackHartmann` "SH24" builder for GMT AGWS model
///
/// Default properties:
///  - n_sensor: 1
///  - lenslet_array:
///    - n_lenslet: 24
///    - n_px_lenslet: 32px
///    - lenslet_pitch: 25.5m/24
///  - detector:
///    - n_px_framelet: 12px
///    - n_px_imagelet: Some(60px)
///    - osf: Some(2)
///
/// # Examples
///
/// ```
/// use ceo::{Builder, SH48, Geometric};
/// let mut wfs = SH48::<Geometric>::new().build();
/// ```
#[derive(Debug, Clone)]
pub struct SH24<T: Model>(SHACKHARTMANN<T>);

impl<T: Model> Default for SH24<T> {
    fn default() -> Self {
        SH24(
            SHACKHARTMANN::new()
                .n_sensor(1)
                .lenslet_array(24, 32, 25.5 / 24.0)
                .detector(12, Some(60), Some(2), None),
        )
    }
}
impl<T: Model> SH24<T> {}
impl<T: Model> WavefrontSensorBuilder for SH24<T> {
    fn guide_stars(&self, template: Option<SOURCE>) -> SOURCE {
        self.0.guide_stars(template)
    }

    fn detector_noise_specs(self, noise_specs: NoiseDataSheet) -> Self {
        Self(self.0.detector_noise_specs(noise_specs))
    }
}
impl<T: Model> Builder for SH24<T> {
    type Component = ShackHartmann<T>;
    fn build(self) -> Result<ShackHartmann<T>> {
        self.0.build()
    }
}
