//!
//! # CEO shackhartmann wrapper
//!
//! Provides a structure `ShackHartmann` that is a wrapper for [CEO](https://github.com/rconan/CEO) shackhartmann C++ structure.
//! `ShackHartmann<M: Model>` is instantiated and initialized with the `SHACKHARTMANN<M: Model>` builder where `Model` is either type `Geometric` of `Diffractive`
//!
//! # Examples
//!
//! ```
//! use ceo::ceo;
//! // Creates a gmt instance with default parameters
//! let mut wfs = ceo!(SHACKHARTMANN:Geometric);
//! ```

use super::{
    ceo_bindings::{geometricShackHartmann, shackHartmann},
    imaging::NoiseDataSheet,
    Source,
};
use std::f32;

pub mod sh;
pub use sh::{ShackHartmann, SHACKHARTMANN};
mod sh48;
pub use sh48::SH48;
mod sh24;
pub use sh24::SH24;

pub type Geometric = geometricShackHartmann;
pub type Diffractive = shackHartmann;

/// Shack-Hartmann model type: Geometric or Diffractive
pub trait Model: Clone {
    fn new() -> Self;
    fn build(
        &mut self,
        n_side_lenslet: i32,
        d: f32,
        n_sensor: i32,
        n_px_lenslet: i32,
        osf: i32,
        n_px: i32,
        b: i32,
    );
    fn get_c_as_mut_ptr(&mut self) -> *mut f32;
    fn calibrate(&mut self, src: &mut Source, threshold: f64) -> &mut Self;
    fn drop(&mut self);
}
impl Model for Geometric {
    fn new() -> Self {
        Default::default()
    }
    fn build(
        &mut self,
        n_side_lenslet: i32,
        d: f32,
        n_sensor: i32,
        _n_px_lenslet: i32,
        _osf: i32,
        _n_px: i32,
        _b: i32,
    ) {
        unsafe {
            self.setup(n_side_lenslet, d, n_sensor);
        }
    }
    fn get_c_as_mut_ptr(&mut self) -> *mut f32 {
        self.data_proc.d__c
    }
    fn drop(&mut self) {
        unsafe { self.cleanup() };
    }
    /// Calibrates the `ShackHartmann` WFS reference slopes and valid lenslets
    fn calibrate(&mut self, src: &mut Source, threshold: f64) -> &mut Self {
        unsafe {
            self.calibrate(src.as_raw_mut_ptr(), threshold as f32);
        }
        self
    }
}
impl Model for Diffractive {
    fn new() -> Self {
        Default::default()
    }
    fn drop(&mut self) {
        unsafe {
            self.cleanup();
        }
    }
    fn get_c_as_mut_ptr(&mut self) -> *mut f32 {
        self.data_proc.d__c
    }
    fn build(
        &mut self,
        n_side_lenslet: i32,
        d: f32,
        n_sensor: i32,
        n_px_lenslet: i32,
        osf: i32,
        n_px: i32,
        b: i32,
    ) {
        unsafe {
            self.setup(n_side_lenslet, n_px_lenslet, d, osf, n_px, b, n_sensor);
        }
    }
    fn calibrate(&mut self, src: &mut Source, threshold: f64) -> &mut Self {
        unsafe {
            self.calibrate(src.as_raw_mut_ptr(), threshold as f32);
            self.camera.reset();
        }
        self
    }
}

/// Lenslet array specifications
/// n_side_lenslet, n_px_lenslet, d
#[doc(hidden)]
#[derive(Debug, Clone)]
pub struct LensletArray(pub usize, pub usize, pub f64);
impl Default for LensletArray {
    fn default() -> Self {
        LensletArray(1, 511, 25.5)
    }
}
/// Detector noise model specifications
/// n_px_framelet, n_px_imagelet, osf, detector_noise_specs
#[doc(hidden)]
#[derive(Debug, Clone)]
pub struct Detector(
    pub usize,
    pub Option<usize>,
    pub Option<usize>,
    pub Option<NoiseDataSheet>,
);
impl Default for Detector {
    fn default() -> Self {
        Detector(512, None, None, None)
    }
}
