use ffi::pyramid;
use serde::{Deserialize, Serialize};

use crate::{
    wavefrontsensor::{LensletArray, SegmentWiseSensorBuilder},
    Builder,
};

use super::{Modulation, Pyramid};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PyramidBuilder {
    lenslet_array: LensletArray,
    modulation: Option<Modulation>,
    alpha: f32,
    n_gs: i32,
}
impl Default for PyramidBuilder {
    fn default() -> Self {
        Self {
            lenslet_array: LensletArray {
                n_side_lenslet: 30,
                n_px_lenslet: 8,
                d: 0f64,
            },
            modulation: None::<Modulation>,
            alpha: 0.5f32,
            n_gs: 1,
        }
    }
}

impl PyramidBuilder {
    pub fn n_lenslet(mut self, n_lenslet: usize) -> Self {
        self.lenslet_array.n_side_lenslet = n_lenslet;
        self
    }
    pub fn modulation(mut self, amplitude: f32, sampling: i32) -> Self {
        self.modulation = Some(Modulation {
            amplitude,
            sampling,
        });
        self
    }
}

impl SegmentWiseSensorBuilder for PyramidBuilder {
    fn pupil_sampling(&self) -> usize {
        let LensletArray {
            n_side_lenslet,
            n_px_lenslet,
            ..
        } = self.lenslet_array;
        n_side_lenslet * n_px_lenslet
    }
}

impl Builder for PyramidBuilder {
    type Component = Pyramid;

    fn build(self) -> crate::Result<Self::Component> {
        let mut pym = Pyramid {
            _c_: pyramid::default(),
            lenslet_array: self.lenslet_array,
            alpha: self.alpha,
            modulation: self.modulation,
        };
        let LensletArray {
            n_side_lenslet,
            n_px_lenslet,
            ..
        } = self.lenslet_array;
        let n_pupil_sampling = n_side_lenslet * n_px_lenslet;
        let Modulation {
            amplitude,
            sampling,
        } = self.modulation.unwrap_or_default();
        unsafe {
            pym._c_.setup(
                n_side_lenslet as i32,
                n_pupil_sampling as i32,
                amplitude,
                sampling,
                self.alpha,
                self.n_gs,
            );
        };

        Ok(pym)
    }
}
