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

use crate::Builder;

use self::pyramid::{Calibration, SlopesArray};

use super::{imaging::NoiseDataSheet, Cu, Mask, Single, Source};
use ffi::{geometricShackHartmann, get_device_count, mask, set_device, shackHartmann};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};
use std::{f32, thread};

pub mod shackhartmann;
pub use shackhartmann::{ShackHartmann, ShackHartmannBuilder};
mod sh48;
pub use sh48::SH48;
mod sh24;
pub use sh24::SH24;
mod pyramid;
pub use pyramid::{Pyramid, PyramidBuilder, QuadCell};
mod geom_shack;
pub use geom_shack::{GeomShack, GeomShackBuilder};

pub type Geometric = geometricShackHartmann;
pub type Diffractive = shackHartmann;

/// Shack-Hartmann model type: Geometric or Diffractive
pub trait Model: Clone + Send {
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
    fn readout(
        &mut self,
        exposure_time: f32,
        rms_read_out_noise: f32,
        n_background_photon: f32,
        noise_factor: f32,
    ) -> &mut Self;
    fn frame(&self) -> Option<Vec<f32>>;
    fn n_frame(&self) -> usize;
    fn valid_lenslet_from(&mut self, wfs: &mut mask);
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
    fn readout(
        &mut self,
        _exposure_time: f32,
        _rms_read_out_noise: f32,
        _n_background_photon: f32,
        _noise_factor: f32,
    ) -> &mut Self {
        self
    }
    fn frame(&self) -> Option<Vec<f32>> {
        None
    }
    fn n_frame(&self) -> usize {
        0
    }
    fn valid_lenslet_from(&mut self, wfs: &mut mask) {
        unsafe {
            self.valid_lenslet.reset();
            self.valid_lenslet.add(wfs);
            self.valid_lenslet.set_filter();
            self.valid_lenslet.set_index();
        }
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
    fn readout(
        &mut self,
        exposure_time: f32,
        rms_read_out_noise: f32,
        n_background_photon: f32,
        noise_factor: f32,
    ) -> &mut Self {
        unsafe {
            self.camera.readout1(
                exposure_time,
                rms_read_out_noise,
                n_background_photon,
                noise_factor,
            );
        }
        self
    }
    fn frame(&self) -> Option<Vec<f32>> {
        let n = self.camera.N_PX_CAMERA * self.camera.N_PX_CAMERA * self.camera.N_LENSLET;
        let m = self.camera.N_SOURCE;
        let mut data: Cu<Single> = Cu::array(n as usize, m as usize);
        data.from_ptr(self.camera.d__frame);
        Some(data.into())
    }
    fn n_frame(&self) -> usize {
        self.camera.N_FRAME as usize
    }
    fn valid_lenslet_from(&mut self, wfs: &mut mask) {
        unsafe {
            self.valid_lenslet.reset();
            self.valid_lenslet.add(wfs);
            self.valid_lenslet.set_filter();
            self.valid_lenslet.set_index();
        }
    }
}

/// Lenslet array specifications
/// n_side_lenslet, n_px_lenslet, d
#[doc(hidden)]
#[derive(Debug, Clone, PartialEq, Copy, Serialize, Deserialize)]
pub struct LensletArray {
    pub n_side_lenslet: usize,
    pub n_px_lenslet: usize,
    pub d: f64,
}
impl Default for LensletArray {
    fn default() -> Self {
        LensletArray {
            n_side_lenslet: 1,
            n_px_lenslet: 511,
            d: 25.5,
        }
    }
}
/// Detector noise model specifications
/// n_px_framelet, n_px_imagelet, osf, detector_noise_specs
#[doc(hidden)]
#[derive(Debug, Clone, PartialEq, Copy, Serialize, Deserialize)]
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

pub trait SegmentWiseSensor {
    fn calibrate_segment(
        &mut self,
        sid: usize,
        n_mode: usize,
        pb: Option<ProgressBar>,
    ) -> SlopesArray;
    fn calibrate(&mut self, n_mode: usize) -> Calibration {
        (1..=7)
            .inspect(|i| println!("Calibrating segment # {i}"))
            .fold(Calibration::default(), |mut c, i| {
                c.push(self.calibrate_segment(i, n_mode, None));
                c
            })
    }
}

pub trait SegmentWiseSensorBuilder: Builder + Clone + Copy + Send + Sized + 'static {
    fn calibrate(self, n_mode: usize) -> Calibration
    where
        Self::Component: SegmentWiseSensor,
    {
        let m = MultiProgress::new();
        let mut handle = vec![];
        for sid in 1..=7 {
            let pb = m.add(ProgressBar::new(n_mode as u64 - 1));
            pb.set_style(
                ProgressStyle::with_template(
                    "{msg} [{eta_precise}] {bar:50.cyan/blue} {pos:>7}/{len:7}",
                )
                .unwrap(),
            );
            pb.set_message(format!("Calibrating segment #{sid}"));
            let n = unsafe { get_device_count() };
            let builder = self.clone();
            handle.push(thread::spawn(move || {
                unsafe { set_device((sid - 1) as i32 % n) };
                let mut pym = builder.build().unwrap();
                pym.calibrate_segment(sid, n_mode, Some(pb))
            }));
        }
        let calibration = handle.into_iter().fold(Calibration::default(), |mut c, h| {
            c.push(h.join().unwrap());
            c
        });
        m.clear().unwrap();
        calibration
    }
}
