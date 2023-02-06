use ffi::pyramid;
use serde::{Deserialize, Serialize};

use crate::{Builder, FromBuilder, Propagation};

use super::LensletArray;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PyramidBuilder {
    lenslet_array: LensletArray,
    modulation: f32,
    modulation_sampling: i32,
    alpha: f32,
    n_gs: i32,
}
impl Default for PyramidBuilder {
    fn default() -> Self {
        Self {
            lenslet_array: LensletArray(30, 8, 0f64),
            modulation: 0f32,
            modulation_sampling: 0i32,
            alpha: 0.25f32,
            n_gs: 1,
        }
    }
}
impl FromBuilder for Pyramid {
    type ComponentBuilder = PyramidBuilder;
}
impl PyramidBuilder {
    pub fn n_lenslet(mut self, n_lenslet: usize) -> Self {
        self.lenslet_array.0 = n_lenslet;
        self
    }
}

pub struct Pyramid {
    _c_: pyramid,
    lenslet_array: LensletArray,
    alpha: f32,
}
impl Drop for Pyramid {
    /// Frees CEO memory before dropping `Pyramid`
    fn drop(&mut self) {
        unsafe {
            self._c_.cleanup();
        }
    }
}
impl Builder for PyramidBuilder {
    type Component = Pyramid;

    fn build(self) -> crate::Result<Self::Component> {
        let mut pym = Pyramid {
            _c_: pyramid::default(),
            lenslet_array: self.lenslet_array,
            alpha: self.alpha,
        };
        let LensletArray(n_side_lenslet, n_px_lenslet, _) = self.lenslet_array;
        let n_pupil_sampling = n_side_lenslet * n_px_lenslet;
        unsafe {
            pym._c_.setup(
                n_side_lenslet as i32,
                n_pupil_sampling as i32,
                self.modulation,
                self.modulation_sampling,
                self.alpha,
                self.n_gs,
            );
        };

        Ok(pym)
    }
}

impl Propagation for Pyramid {
    fn propagate(&mut self, src: &mut crate::Source) {
        unsafe {
            self._c_
                .camera
                .propagateThroughPyramid(src.as_raw_mut_ptr(), self.alpha)
        }
    }

    fn time_propagate(&mut self, _secs: f64, _src: &mut crate::Source) {
        todo!()
    }
}

impl Pyramid {
    pub fn frame(&self) -> Vec<f32> {
        let n = self._c_.camera.N_PX_CAMERA.pow(2) * self._c_.camera.N_SOURCE;
        let mut frame = vec![0f32; n as usize];
        unsafe {
            ffi::dev2host(frame.as_mut_ptr(), self._c_.camera.d__frame, n);
        }
        frame
    }
    #[inline]
    pub fn n_px_camera(&self) -> usize {
        self._c_.camera.N_PX_CAMERA as usize
    }
    pub fn pupil_sampling(&self) -> usize {
        self.lenslet_array.0 * self.lenslet_array.1
    }
    pub fn camera_resolution(&self) -> (usize, usize) {
        (self.n_px_camera(), self.n_px_camera())
    }
}

#[cfg(test)]
mod tests {
    use crate::{FromBuilder, Gmt, Source};

    use super::*;

    #[test]
    fn propagation() {
        let mut gmt = Gmt::builder().build().unwrap();
        let mut pym = Pyramid::builder().n_lenslet(90).build().unwrap();
        let mut src = Source::builder()
            .pupil_sampling(pym.pupil_sampling())
            .build()
            .unwrap();
        src.through(&mut gmt).xpupil().through(&mut pym);
        let _: complot::Heatmap = ((pym.frame().as_slice(), pym.camera_resolution()), None).into();
    }
}
