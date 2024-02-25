use ffi::pyramid;
use serde::{Deserialize, Serialize};

use crate::{
    wavefrontsensor::{LensletArray, SegmentWiseSensorBuilder},
    Builder,
};

use super::{piston_sensor::PistonSensor, Modulation, Pyramid};

/// [Pyramid] builder
///
/// Default properties:
///   - n_lenslet: 30
///   - n_px_lenslet: 8px
///   - lenslet_pitch: 0
///   - no modulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PyramidBuilder {
    pub lenslet_array: LensletArray,
    modulation: Option<Modulation>,
    alpha: f32,
    n_gs: i32,
    pub piston_sensor: Option<PistonSensor>,
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
            piston_sensor: None,
        }
    }
}

impl PyramidBuilder {
    /// Sets the number of equivalent lenslets
    pub fn n_lenslet(mut self, n_lenslet: usize) -> Self {
        self.lenslet_array.n_side_lenslet = n_lenslet;
        self
    }
    /// Sets the equivalent lenslet array
    pub fn lenslet_array(mut self, lenslet_array: LensletArray) -> Self {
        self.lenslet_array = lenslet_array;
        self
    }
    /// Sets the modulation amplitude and sampling
    ///
    /// The amplitude is given in units of lambda/d
    pub fn modulation(mut self, amplitude: f32, sampling: i32) -> Self {
        self.modulation = Some(Modulation {
            amplitude,
            sampling,
        });
        self
    }
    /*     pub fn piston_sensor<G: Into<GmtSegmentation>>(
        &mut self,
        calibration: &Calibration,
        gmt_segmentation: G,
    ) -> Result<(), CrseoError> {
        self.piston_sensor = Some(PistonSensor::new(
            self,
            calibration.masks(),
            gmt_segmentation.into(),
            calibration.src.clone(),
        )?);
        Ok(())
    } */

    /*     pub fn piston_mask<'a>(
        &self,
        masks: impl Iterator<Item = Option<&'a nalgebra::DMatrix<bool>>>,
        segmentation: GmtSegmentation,
        gs: SourceBuilder,
    ) -> Result<(Vec<bool>, Vec<bool>), CrseoError> {
        let piston_sensor = PistonSensor::new(&self, masks, segmentation, gs)?;
        Ok(piston_sensor.mask)
    } */
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
            piston_sensor: self.piston_sensor,
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
