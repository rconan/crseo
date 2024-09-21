use crate::Builder;

use super::{Detector, Imaging, LensletArray};

/// Imaging builder
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ImagingBuilder {
    pub n_sensor: i32,
    pub lenslet_array: LensletArray,
    detector: Detector,
    fluxlet_threshold: f64,
}
impl Default for ImagingBuilder {
    fn default() -> Self {
        Self {
            n_sensor: 1,
            lenslet_array: Default::default(),
            detector: Default::default(),
            fluxlet_threshold: Default::default(),
        }
    }
}

impl ImagingBuilder {
    /// Sets the # of sensors
    pub fn n_sensor(mut self, n_sensor: usize) -> Self {
        self.n_sensor = n_sensor as i32;
        self
    }
    /// Sets the lenslet array property
    pub fn lenslet_array(mut self, lenslet_array: LensletArray) -> Self {
        self.lenslet_array = lenslet_array;
        self.detector.n_px_framelet = lenslet_array.n_px_lenslet;
        self
    }
    /// Sets the detector property
    pub fn detector(mut self, detector: Detector) -> Self {
        self.detector = detector;
        self
    }
    /// Lenslet selection based on lenslet flux threshold
    pub fn lenslet_flux(mut self, threshold: f64) -> Self {
        self.fluxlet_threshold = threshold;
        self
    }
}

impl Builder for ImagingBuilder {
    type Component = Imaging;

    fn build(self) -> crate::Result<Self::Component> {
        let LensletArray {
            n_side_lenslet,
            n_px_lenslet,
            ..
        } = self.lenslet_array;
        let Detector {
            n_px_framelet,
            n_px_imagelet,
            osf,
            ..
        } = self.detector;
        let n_px = n_px_imagelet.unwrap_or_else(|| n_px_lenslet);
        let binning = (n_px / n_px_framelet).max(1);

        let mut imgr = Imaging {
            _c_: Default::default(),
            dft_osf: osf,
            fluxlet_threshold: self.fluxlet_threshold,
        };

        unsafe {
            imgr._c_.setup3(
                n_px_lenslet as i32,
                n_side_lenslet as i32,
                osf as i32,
                n_px as i32,
                binning as i32,
                self.n_sensor as i32,
            );
        }
        Ok(imgr)
    }
}
