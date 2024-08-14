use crate::Builder;

use super::{Detector, Imaging, LensletArray};

/// Imaging builder
pub struct ImagingBuilder {
    n_sensor: i32,
    lenslet_array: LensletArray,
    detector: Detector,
}
impl Default for ImagingBuilder {
    fn default() -> Self {
        Self {
            n_sensor: 1,
            lenslet_array: Default::default(),
            detector: Default::default(),
        }
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
        let n_px = match n_px_imagelet {
            Some(n_px_imagelet) => n_px_imagelet,
            None => n_px_framelet,
        };
        let binning = n_px / n_px_framelet;

        let mut imgr = Imaging {
            _c_: Default::default(),
            dft_osf: osf,
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
