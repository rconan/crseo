use std::{
    mem::take,
    ops::{Deref, DerefMut},
};

use super::{Model, ShackHartmann};
use crate::{
    Builder, FromBuilder, Result, ShackHartmannBuilder, SourceBuilder, WavefrontSensorBuilder,
};

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
/// use ceo::{Builder, SH24, Geometric};
/// let mut wfs = SH24::<Geometric>::new().build();
/// ```
#[derive(Debug, Clone)]
pub struct SH24<T: Model>(ShackHartmannBuilder<T>);

impl<T: Model> Deref for SH24<T> {
    type Target = ShackHartmannBuilder<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T: Model> DerefMut for SH24<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: Model> Default for SH24<T> {
    fn default() -> Self {
        SH24(
            ShackHartmannBuilder::new()
                .n_sensor(1)
                .lenslet_array(24, 32, 25.5 / 24.0)
                .detector(12, Some(60), Some(2), None),
        )
    }
}
impl<T: Model> SH24<T> {}
impl<M, T> WavefrontSensorBuilder for T
where
    M: Model,
    T: Deref<Target = ShackHartmannBuilder<M>>,
{
    fn guide_stars(&self, template: Option<SourceBuilder>) -> SourceBuilder {
        self.deref().guide_stars(template)
    }
    /*
        fn detector_noise_specs(self, noise_specs: NoiseDataSheet) -> Self {
            Self(self.detector_noise_specs(noise_specs))
        }
    */
}
impl<M, T> Builder for T
where
    M: Model,
    T: DerefMut<Target = ShackHartmannBuilder<M>> + Default,
{
    type Component = ShackHartmann<M>;
    fn build(mut self) -> Result<ShackHartmann<M>> {
        take(self.deref_mut()).build()
    }
}

pub struct ShackHartmann24x24<T: Model>(ShackHartmann<T>);
impl<T: Model> Deref for ShackHartmann24x24<T> {
    type Target = ShackHartmann<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T: Model> DerefMut for ShackHartmann24x24<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<M, T> FromBuilder for T
where
    M: Model,
    T: DerefMut<Target = ShackHartmann<M>>,
{
    type ComponentBuilder = ShackHartmannBuilder<M>;
}
