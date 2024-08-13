use serde::{Deserialize, Serialize};

use crate::{imaging::LensletArray, wavefrontsensor::SegmentWiseSensorBuilder, Builder};

use super::GeomShack;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct GeomShackBuilder {
    lenslet_array: LensletArray,
    pub(super) n_gs: i32,
}
impl Default for GeomShackBuilder {
    fn default() -> Self {
        Self {
            lenslet_array: LensletArray::default(),
            n_gs: 1,
        }
    }
}
impl GeomShackBuilder {
    pub fn lenslet(mut self, n_side_lenslet: usize, n_px_lenslet: usize) -> Self {
        self.lenslet_array = LensletArray {
            n_side_lenslet,
            n_px_lenslet,
            d: 25.5 / n_side_lenslet as f64,
        };
        self
    }
    pub fn size(mut self, n: usize) -> Self {
        self.n_gs = n as i32;
        self
    }
}

impl SegmentWiseSensorBuilder for GeomShackBuilder {
    fn pupil_sampling(&self) -> usize {
        let LensletArray {
            n_side_lenslet,
            n_px_lenslet,
            ..
        } = self.lenslet_array;
        n_side_lenslet * n_px_lenslet + 1
    }
}

impl Builder for GeomShackBuilder {
    type Component = GeomShack;

    fn build(self) -> crate::Result<Self::Component> {
        let mut wfs = GeomShack {
            _c_: ffi::geometricShackHartmann::default(),
            lenslet_array: self.lenslet_array,
            n_gs: self.n_gs as usize,
        };
        let LensletArray {
            n_side_lenslet, d, ..
        } = self.lenslet_array;
        unsafe {
            wfs._c_.setup(n_side_lenslet as i32, d as f32, self.n_gs);
        };

        Ok(wfs)
    }
}
