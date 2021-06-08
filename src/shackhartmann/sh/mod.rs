/// `ShackHartmann` builder
///
/// Default properties:
///  - n_sensor: 1
///  - lenslet_array:
///    - n_lenslet: 1
///    - n_px_lenslet: 511px
///    - lenslet_pitch: 25.5m
///  - detector:
///    - n_px_framelet: 512px
///    - n_px_imagelet: None[512px]
///    - osf: None[2]
///
/// # Examples
///
/// ```
/// use ceo::{Builder,  SHACKHARTMANN, Geometric};
/// let mut wfs = SHACKHARTMANN::<Geometric>::new().build();
/// ```
use super::{
    Detector, Diffractive, Geometric, LensletArray, Model, WavefrontSensor, WavefrontSensorBuilder,
};
use crate::{imaging::NoiseDataSheet, Builder, Cu, Result, SOURCE};
pub mod sensor;
pub use sensor::ShackHartmann;

#[derive(Debug, Clone)]
pub struct SHACKHARTMANN<T: Model> {
    pub n_sensor: usize,
    pub lenslet_array: LensletArray,
    pub detector: Detector,
    marker: std::marker::PhantomData<T>,
}
impl<T: Model> Default for SHACKHARTMANN<T> {
    fn default() -> Self {
        SHACKHARTMANN {
            n_sensor: 1,
            lenslet_array: LensletArray::default(),
            detector: Detector::default(),
            marker: std::marker::PhantomData,
        }
    }
}
impl<T: Model> SHACKHARTMANN<T> {
    pub fn n_sensor(self, n_sensor: usize) -> Self {
        Self { n_sensor, ..self }
    }
    pub fn lenslet_array(self, n_side_lenslet: usize, n_px_lenslet: usize, d: f64) -> Self {
        Self {
            lenslet_array: LensletArray(n_side_lenslet, n_px_lenslet, d),
            ..self
        }
    }
    pub fn detector(
        self,
        n_px_framelet: usize,
        n_px_imagelet: Option<usize>,
        osf: Option<usize>,
        detector_noise_specs: Option<NoiseDataSheet>,
    ) -> Self {
        Self {
            detector: Detector(n_px_framelet, n_px_imagelet, osf, detector_noise_specs),
            ..self
        }
    }
}
impl<T: Model> WavefrontSensorBuilder for SHACKHARTMANN<T> {
    fn guide_stars(&self, template: Option<SOURCE>) -> SOURCE {
        let LensletArray(n_side_lenslet, n_px_lenslet, d) = self.lenslet_array;
        match template {
            Some(src) => src,
            None => SOURCE::new(),
        }
        .size(self.n_sensor)
        .pupil_size(d * n_side_lenslet as f64)
        .pupil_sampling(n_px_lenslet * n_side_lenslet + 1)
    }

    fn detector_noise_specs(self, noise_specs: NoiseDataSheet) -> Self {
        let mut detector = self.detector;
        detector.3 = Some(noise_specs);
        Self { detector, ..self }
    }
}
impl<T: Model> Builder for SHACKHARTMANN<T> {
    type Component = ShackHartmann<T>;
    fn build(self) -> Result<ShackHartmann<T>> {
        let LensletArray(n_side_lenslet, n_px_lenslet, d) = self.lenslet_array;
        let Detector(n_px_framelet, n_px_imagelet, osf, detector_noise_model) = self.detector;
        let mut wfs = ShackHartmann::<T> {
            _c_: Model::new(),
            n_side_lenslet: n_side_lenslet as i32,
            n_px_lenslet: n_px_lenslet as i32,
            d,
            n_sensor: self.n_sensor as i32,
            n_centroids: 0,
            centroids: Cu::vector((n_side_lenslet * n_side_lenslet * 2 * self.n_sensor) as usize),
            detector_noise_model,
        };
        let n_px = match n_px_imagelet {
            Some(n_px_imagelet) => n_px_imagelet,
            None => n_px_framelet,
        };
        let b = n_px / n_px_framelet;
        let o = osf.unwrap_or(2);
        wfs.n_centroids = wfs.n_side_lenslet * wfs.n_side_lenslet * 2 * wfs.n_sensor;
        wfs._c_.build(
            wfs.n_side_lenslet,
            wfs.d as f32,
            wfs.n_sensor,
            wfs.n_px_lenslet,
            o as i32,
            n_px as i32,
            b as i32,
        );
        wfs.centroids.from_ptr(wfs._c_.get_c_as_mut_ptr());
        Ok(wfs)
    }
}
