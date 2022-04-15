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
    ceo_bindings::{geometricShackHartmann, mask, shackHartmann},
    imaging::NoiseDataSheet,
    Cu, Mask, Single, Source,
};
use std::f32;

pub mod sh;
pub use sh::{ShackHartmann, ShackHartmannBuilder};
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
    fn as_mut_ptr(&mut self) -> *mut f32;
    fn calibrate(&mut self, src: &mut Source, threshold: f64) -> &mut Self;
    fn valid_lenslets(&mut self) -> Vec<f32>;
    fn reset(&mut self) -> &mut Self;
    fn process(&mut self) -> &mut Self;
    /// Returns the centroids corresponding to the valid lenslets
    ///
    /// The first half of the valid lenslet centroids contains all the valid centroids
    /// of all the guide stars along the X–axis direction and the second half contains
    /// all the valid slopes of  all the guide stars along the Y–axis direction
    fn data(&mut self) -> Cu<Single>;
    fn n_valid_lenslet(&mut self) -> usize;
    fn lenslet_mask(&mut self) -> Cu<Single>;
    fn valid_lenslet(&mut self) -> &mut mask;
    fn lenslet_flux(&mut self) -> Cu<Single>;
    fn set_valid_lenslet(&mut self, lenslet_mask: &[i32]);
    fn set_reference_slopes(&mut self, src: &mut Source);
    fn filter(&mut self, lenslet_mask: &mut Mask) -> Cu<Single>;
    fn drop(&mut self);
    fn propagate(&mut self, src: &mut Source) -> &mut Self;
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
    fn as_mut_ptr(&mut self) -> *mut f32 {
        self.data_proc.d__c
    }
    fn reset(&mut self) -> &mut Self {
        unsafe {
            self.reset();
        }
        self
    }
    fn process(&mut self) -> &mut Self {
        unsafe {
            self.process();
        }
        self
    }
    fn data(&mut self) -> Cu<Single> {
        let m = self.valid_lenslet.nnz as usize * 2usize;
        let mut data: Cu<Single> = Cu::vector(m);
        data.malloc();
        unsafe {
            self.get_valid_slopes(data.as_ptr());
        }
        data
    }
    fn n_valid_lenslet(&mut self) -> usize {
        self.valid_lenslet.nnz as usize
    }
    fn drop(&mut self) {
        unsafe { self.cleanup() };
    }
    fn valid_lenslets(&mut self) -> Vec<f32> {
        let n = self.N_SIDE_LENSLET.pow(2) * self.N_WFS;
        let mut mask: Cu<Single> = Cu::vector(n as usize);
        mask.from_ptr(self.valid_lenslet.f);
        mask.into()
    }
    fn lenslet_mask(&mut self) -> Cu<Single> {
        let mut mask: Cu<Single> =
            Cu::vector((self.N_SIDE_LENSLET * self.N_SIDE_LENSLET * self.N_WFS) as usize);
        mask.from_ptr(self.valid_lenslet.f);
        mask
    }
    fn lenslet_flux(&mut self) -> Cu<Single> {
        let mut flux: Cu<Single> =
            Cu::vector((self.N_SIDE_LENSLET * self.N_SIDE_LENSLET * self.N_WFS) as usize);
        flux.from_ptr(self.data_proc.d__mass);
        flux
    }
    fn valid_lenslet(&mut self) -> &mut mask {
        &mut self.valid_lenslet
    }
    fn set_valid_lenslet(&mut self, lenslet_mask: &[i32]) {
        let mut cu_lenslet_mask: Cu<Single> = lenslet_mask
            .iter()
            .map(|x| *x as f32)
            .collect::<Vec<f32>>()
            .into();
        let mut mask = Mask::new();
        mask.build(lenslet_mask.len()).filter(&mut cu_lenslet_mask);
        unsafe {
            self.valid_lenslet.reset();
            self.valid_lenslet.add(mask.as_raw_mut_ptr());
            self.valid_lenslet.set_filter();
            self.valid_lenslet.set_index();
        }
    }
    fn set_reference_slopes(&mut self, src: &mut Source) {
        unsafe { self.set_reference_slopes(src.as_raw_mut_ptr()) }
    }
    fn filter(&mut self, lenslet_mask: &mut Mask) -> Cu<Single> {
        let m = lenslet_mask.nnz() as usize * 2usize;
        let mut data: Cu<Single> = Cu::vector(m);
        data.malloc();
        unsafe {
            self.masked_slopes(data.as_ptr(), lenslet_mask.as_mut_prt());
        }
        data
    }
    /// Calibrates the `ShackHartmann` WFS reference slopes and valid lenslets
    fn calibrate(&mut self, src: &mut Source, threshold: f64) -> &mut Self {
        unsafe {
            self.calibrate(src.as_raw_mut_ptr(), threshold as f32);
        }
        self
    }
    fn propagate(&mut self, src: &mut Source) -> &mut Self {
        unsafe {
            self.propagate(src.as_raw_mut_ptr());
        }
        self
    }
}
impl Model for Diffractive {
    fn new() -> Self {
        Default::default()
    }
    fn reset(&mut self) -> &mut Self {
        unsafe {
            self.camera.reset();
        }
        self
    }
    fn process(&mut self) -> &mut Self {
        unsafe {
            self.process();
        }
        self
    }
    fn data(&mut self) -> Cu<Single> {
        let m = self.valid_lenslet.nnz as usize * 2usize;
        let mut data: Cu<Single> = Cu::vector(m);
        data.malloc();
        unsafe {
            self.get_valid_slopes(data.as_ptr());
        }
        data
    }
    fn n_valid_lenslet(&mut self) -> usize {
        self.valid_lenslet.nnz as usize
    }
    fn lenslet_mask(&mut self) -> Cu<Single> {
        let mut mask: Cu<Single> =
            Cu::vector((self.N_SIDE_LENSLET * self.N_SIDE_LENSLET * self.N_WFS) as usize);
        mask.from_ptr(self.valid_lenslet.f);
        mask
    }
    fn lenslet_flux(&mut self) -> Cu<Single> {
        let mut flux: Cu<Single> =
            Cu::vector((self.N_SIDE_LENSLET * self.N_SIDE_LENSLET * self.N_WFS) as usize);
        flux.from_ptr(self.data_proc.d__mass);
        flux
    }
    fn set_valid_lenslet(&mut self, lenslet_mask: &[i32]) {
        let mut cu_lenslet_mask: Cu<Single> = lenslet_mask
            .iter()
            .map(|x| *x as f32)
            .collect::<Vec<f32>>()
            .into();
        let mut mask = Mask::new();
        mask.build(lenslet_mask.len()).filter(&mut cu_lenslet_mask);
        unsafe {
            self.valid_lenslet.reset();
            self.valid_lenslet.add(mask.as_raw_mut_ptr());
            self.valid_lenslet.set_filter();
            self.valid_lenslet.set_index();
        }
    }
    fn valid_lenslet(&mut self) -> &mut mask {
        &mut self.valid_lenslet
    }
    fn set_reference_slopes(&mut self, src: &mut Source) {
        unsafe { self.set_reference_slopes(src.as_raw_mut_ptr()) }
    }
    fn filter(&mut self, lenslet_mask: &mut Mask) -> Cu<Single> {
        let m = lenslet_mask.nnz() as usize * 2usize;
        let mut data: Cu<Single> = Cu::vector(m);
        data.malloc();
        unsafe {
            self.masked_slopes(data.as_ptr(), lenslet_mask.as_mut_prt());
        }
        data
    }
    fn drop(&mut self) {
        unsafe {
            self.cleanup();
        }
    }
    fn as_mut_ptr(&mut self) -> *mut f32 {
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
    fn valid_lenslets(&mut self) -> Vec<f32> {
        let n = self.N_SIDE_LENSLET.pow(2) * self.N_WFS;
        let mut mask: Cu<Single> = Cu::vector(n as usize);
        mask.from_ptr(self.valid_lenslet.f);
        mask.into()
    }
    fn calibrate(&mut self, src: &mut Source, threshold: f64) -> &mut Self {
        unsafe {
            self.calibrate(src.as_raw_mut_ptr(), threshold as f32);
            self.camera.reset();
        }
        self
    }
    fn propagate(&mut self, src: &mut Source) -> &mut Self {
        unsafe {
            self.propagate(src.as_raw_mut_ptr());
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
