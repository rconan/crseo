//!
//! # CEO source wrapper
//!
//! Provides a structure `Source` that is a wrapper for [CEO](https://github.com/rconan/CEO) source C++ structure.
//! `Source` is instantiated and initialized with the `SOURCE` builder
//!
//! # Examples
//!
//! - on-axis source with default parameters
//!
//! ```
//! use ceo::ceo;
//! // Creates a source with default parameters
//! let mut src = ceo!(SOURCE);
//! ```
//!
//! - 3 sources evenly spread on a ring with a 8 arcminute radius
//!
//! ```
//! use ceo::{ceo, Conversion};
//! let mut src = ceo!(SOURCE, size = [3] , on_ring = [8f32.from_arcmin()]);
//! ```

use super::{cu::Double, cu::Single, Builder, Centroiding, Cu, Result};
use ffi::{bundle, dev2host, dev2host_int, source, vector};

use std::{
    f32,
    ffi::{CStr, CString},
    usize,
};

/// A system that mutates `Source` arguments should implement the `Propagation` trait
pub trait Propagation {
    fn propagate(&mut self, src: &mut Source);
    fn time_propagate(&mut self, secs: f64, src: &mut Source);
}

#[derive(Clone, Debug, PartialEq)]
pub enum PupilSampling {
    SquareGrid {
        size: Option<f64>,
        resolution: usize,
    },
    UserSet(usize),
}
impl PupilSampling {
    pub fn total(&self) -> usize {
        match &self {
            PupilSampling::SquareGrid { resolution, .. } => resolution * resolution,
            PupilSampling::UserSet(n) => *n,
        }
    }
    pub fn side(&self) -> usize {
        match &self {
            PupilSampling::SquareGrid { resolution, .. } => *resolution,
            PupilSampling::UserSet(n) => *n,
        }
    }
    pub fn size(&self) -> Option<f64> {
        match &self {
            PupilSampling::SquareGrid { size, .. } => *size,
            _ => None,
        }
    }
}

/// `Source` builder
///
/// Default properties:
///  - size             : 1
///  - pupil size       : 25.5m
///  - pupil sampling   : 512px
///  - photometric band : Vs (500nm)
///  - zenith           : 0degree
///  - azimuth          : 0degree
///  - magnitude        : 0
///
/// # Examples
///
/// - on-axis source with default parameters
///
/// ```
/// use ceo::{Builder, SOURCE};
/// let mut src = SOURCE::new().build();
/// ```
///
/// - 3 sources evenly spread on a ring with a 8 arcminute radius
///
/// ```
/// use ceo::{Builder, SOURCE, Conversion};
/// let mut src = SOURCE::new().size(3).on_ring(8f32.from_arcmin()).build();
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct SourceBuilder {
    pub size: usize,
    pub pupil_size: f64,
    pub pupil_sampling: PupilSampling,
    pub band: String,
    pub zenith: Vec<f32>,
    pub azimuth: Vec<f32>,
    pub magnitude: Vec<f32>,
    pub rays_coordinates: Option<(Vec<f64>, Vec<f64>)>,
    pub fwhm: Option<f64>,
}
impl Default for SourceBuilder {
    fn default() -> Self {
        SourceBuilder {
            size: 1,
            pupil_size: 25.5,
            pupil_sampling: PupilSampling::SquareGrid {
                size: Some(25.5),
                resolution: 512,
            },
            band: "Vs".into(),
            zenith: vec![0f32],
            azimuth: vec![0f32],
            magnitude: vec![0f32],
            rays_coordinates: None,
            fwhm: None,
        }
    }
}
impl SourceBuilder {
    /// Set the number of sources
    pub fn size(self, size: usize) -> Self {
        Self { size, ..self }
    }
    /// Set the sampling of the pupil in pixels
    pub fn pupil_sampling(self, resolution: usize) -> Self {
        Self {
            pupil_sampling: PupilSampling::SquareGrid {
                size: self.pupil_sampling.size().or(Some(25.5)),
                resolution,
            },
            ..self
        }
    }
    /// Set the pupil size in meters
    pub fn pupil_size(self, pupil_size: f64) -> Self {
        Self { pupil_size, ..self }
    }
    /// Set the photometric band
    pub fn band(self, band: &str) -> Self {
        Self {
            band: band.to_owned(),
            ..self
        }
    }
    /// Set the source zenith and azimuth angles
    pub fn zenith_azimuth(self, zenith: Vec<f32>, azimuth: Vec<f32>) -> Self {
        assert_eq!(
            self.size,
            zenith.len(),
            "zenith vector must be of length {}",
            self.size
        );
        assert_eq!(
            self.size,
            azimuth.len(),
            "azimuth vector must be of length {}",
            self.size
        );
        Self {
            zenith,
            azimuth,
            ..self
        }
    }
    /// Set n sources at zenith angle evenly spread of a ring
    pub fn on_ring(self, zenith: f32) -> Self {
        Self {
            zenith: vec![zenith; self.size],
            azimuth: (0..self.size)
                .map(|x| 2. * std::f32::consts::PI * x as f32 / self.size as f32)
                .collect::<Vec<f32>>(),
            ..self
        }
    }
    /// Set the source magnitude
    pub fn magnitude(self, magnitude: Vec<f32>) -> Self {
        assert_eq!(
            self.size,
            magnitude.len(),
            "azimuth vector must be of length {}",
            self.size
        );
        Self { magnitude, ..self }
    }
    ///  Builds a star field made of 21 sources located at the vertices of a Delaunay mesh sampling a 10 arcminute field of view
    pub fn field_delaunay21(self) -> Self {
        use super::Conversion;
        use serde::Deserialize;
        use serde_pickle as pickle;
        #[derive(Deserialize)]
        struct Field {
            pub zenith_arcmin: Vec<f32>,
            pub azimuth_degree: Vec<f32>,
        }
        let Field {
            zenith_arcmin,
            azimuth_degree,
        } = pickle::from_slice(include_bytes!("fielddelaunay21.pkl"))
            .expect("fielddelaunay21.pkl loading failed!");
        let n_src = zenith_arcmin.len();
        Self {
            size: n_src,
            zenith: zenith_arcmin
                .iter()
                .map(|x| x.from_arcmin())
                .collect::<Vec<f32>>(),
            azimuth: azimuth_degree
                .iter()
                .map(|x| x.to_radians())
                .collect::<Vec<f32>>(),
            magnitude: vec![0f32; n_src],
            ..self
        }
    }
    /// Set the \[x,y\] coordinates of the bundle of rays in the entrance pupil
    pub fn rays_coordinates(self, rays_x: Vec<f64>, rays_y: Vec<f64>) -> Self {
        assert_eq!(
            rays_x.len(),
            rays_y.len(),
            "x and y rays coordinates vector must have the same lenght"
        );
        Self {
            pupil_sampling: PupilSampling::UserSet(rays_x.len()),
            rays_coordinates: Some((rays_x, rays_y)),
            ..self
        }
    }
    /// Sets the `Source` full width at half maximum in un-binned detector pixel
    pub fn fwhm(self, value: f64) -> Self {
        Self {
            fwhm: Some(value),
            ..self
        }
    }
}
impl Builder for SourceBuilder {
    type Component = Source;
    /// Build the `Source`
    fn build(self) -> Result<Source> {
        let mut src = Source {
            _c_: Default::default(),
            size: self.size as i32,
            pupil_size: self.pupil_size,
            pupil_sampling: self.pupil_sampling.side() as i32,
            _wfe_rms: vec![0.0; self.size],
            _phase: vec![0.0; self.pupil_sampling.total() * self.size],
            zenith: self.zenith.clone(),
            azimuth: self.azimuth.clone(),
            magnitude: self.magnitude,
        };

        let origin = vector {
            x: 0.0,
            y: 0.0,
            z: 25.0,
        };
        let src_band = CString::new(self.band.into_bytes()).unwrap();
        if let Some((mut rays_x, mut rays_y)) = self.rays_coordinates {
            let mut zenith: Vec<_> = self.zenith.iter().map(|&x| x as f64).collect();
            let mut azimuth: Vec<_> = self.azimuth.iter().map(|&x| x as f64).collect();
            unsafe {
                src._c_.setup9(
                    src_band.into_raw(),
                    src.magnitude.as_mut_ptr(),
                    zenith.as_mut_ptr(),
                    azimuth.as_mut_ptr(),
                    f32::INFINITY,
                    self.size as i32,
                    rays_x.len() as i32,
                    rays_x.as_mut_ptr(),
                    rays_y.as_mut_ptr(),
                    origin,
                );
            }
        } else {
            unsafe {
                src._c_.setup7(
                    src_band.into_raw(),
                    src.magnitude.as_mut_ptr(),
                    src.zenith.as_mut_ptr(),
                    src.azimuth.as_mut_ptr(),
                    f32::INFINITY,
                    self.size as i32,
                    self.pupil_size,
                    self.pupil_sampling.side() as i32,
                    origin,
                );
            }
        }
        if let Some(fwhm) = self.fwhm {
            src._c_.fwhm = fwhm as f32;
        }
        Ok(src)
    }
}

impl From<&Source> for SourceBuilder {
    fn from(src: &Source) -> Self {
        Self {
            size: src.size as usize,
            pupil_size: src.pupil_size,
            pupil_sampling: PupilSampling::SquareGrid {
                size: Some(src.pupil_size),
                resolution: src.pupil_sampling as usize,
            },
            band: src.get_photometric_band(),
            zenith: src.zenith.clone(),
            azimuth: src.azimuth.clone(),
            magnitude: src.magnitude.clone(),
            rays_coordinates: None,
            fwhm: Some(src._c_.fwhm as f64),
        }
    }
}

/// source wrapper
pub struct Source {
    _c_: source,
    /// The number of sources
    pub size: i32,
    /// The diameter of the entrance pupil \[m\]
    pub pupil_size: f64,
    /// The sampling of the entrance pupil \[px\]
    pub pupil_sampling: i32,
    pub _wfe_rms: Vec<f32>,
    pub _phase: Vec<f32>,
    pub zenith: Vec<f32>,
    pub azimuth: Vec<f32>,
    pub magnitude: Vec<f32>,
}
impl PartialEq for Source {
    fn eq(&self, other: &Self) -> bool {
        Into::<SourceBuilder>::into(self) == Into::<SourceBuilder>::into(other)
    }
}
impl Source {
    /// Creates and empty `Source`
    pub fn empty() -> Source {
        Source {
            _c_: Default::default(),
            size: 0,
            pupil_size: 0.0,
            pupil_sampling: 0,
            _wfe_rms: vec![],
            _phase: vec![],
            zenith: vec![],
            azimuth: vec![],
            magnitude: vec![],
        }
    }
    /// Creates a new `Source` with the arguments:
    ///
    /// * `pupil_size` - the diameter of the entrance pupil \[m\]
    /// * `pupil_sampling` - the sampling of the entrance pupil \[px\]
    pub fn new(size: i32, pupil_size: f64, pupil_sampling: i32) -> Source {
        Source {
            _c_: Default::default(),
            size,
            pupil_size,
            pupil_sampling,
            _wfe_rms: vec![0.0; size as usize],
            _phase: vec![0.0; (pupil_sampling * pupil_sampling * size) as usize],
            zenith: vec![0.0; size as usize],
            azimuth: vec![0.0; size as usize],
            magnitude: vec![0.0; size as usize],
        }
    }
    pub fn from(args: (i32, f64, i32)) -> Source {
        Source::new(args.0, args.1, args.2)
    }
    /// Sets the `Source` parameters:
    ///
    /// * `band` - the photometric band: Vs, V, R, I, J, H, K or R+I
    /// * `zenith` - the zenith angle \[rd\]
    /// * `azimuth` - the azimuth angle \[rd\]
    /// * `magnitude` - the magnitude at the specified photometric band
    pub fn build(
        &mut self,
        band: &str,
        mut zenith: Vec<f32>,
        mut azimuth: Vec<f32>,
        mut magnitude: Vec<f32>,
    ) -> &mut Self {
        assert_eq!(zenith.len(), azimuth.len());
        assert_eq!(zenith.len(), magnitude.len());
        let band = CString::new(band).unwrap();
        unsafe {
            let origin = vector {
                x: 0.0,
                y: 0.0,
                z: 25.0,
            };
            self.magnitude.copy_from_slice(magnitude.as_slice());
            self._c_.setup7(
                band.into_raw(),
                magnitude.as_mut_ptr(),
                zenith.as_mut_ptr(),
                azimuth.as_mut_ptr(),
                f32::INFINITY,
                self.size,
                self.pupil_size,
                self.pupil_sampling,
                origin,
            );
        }
        self
    }
    pub fn as_raw_mut_ptr(&mut self) -> &mut source {
        &mut self._c_
    }
    /// Returns the `Source` photometric band
    pub fn get_photometric_band(&self) -> String {
        unsafe {
            String::from(
                CStr::from_ptr(self._c_.photometric_band)
                    .to_str()
                    .expect("CStr::to_str failed"),
            )
        }
    }
    /// Returns the `Source` wavelength \[m\]
    pub fn wavelength(&mut self) -> f64 {
        unsafe { self._c_.wavelength() as f64 }
    }
    /// Sets the `Source` full width at half maximum in un-binned detector pixel
    pub fn fwhm(&mut self, value: f64) {
        self._c_.fwhm = value as f32;
    }
    /// Set the pupil rotation angle \[degree\]
    pub fn rotate_rays(&mut self, angle: f64) {
        self._c_.rays.rot_angle = angle;
    }
    /// Copies the optical path difference from ray tracing into the wavefront phase argument, this usually takes place after ray tracing to the exit pupil
    pub fn xpupil(&mut self) -> &mut Self {
        unsafe {
            self._c_.wavefront.reset();
            self._c_.opd2phase();
        }
        self
    }
    pub fn opd2phase(&mut self) -> &mut Self {
        unsafe {
            //            self._c_.wavefront.reset();
            self._c_.opd2phase();
        }
        self
    }
    /// Returns the wavefront error root mean square \[m\]
    pub fn wfe_rms(&mut self) -> Vec<f64> {
        unsafe {
            self._c_.wavefront.rms(self._wfe_rms.as_mut_ptr());
        }
        self._wfe_rms
            .clone()
            .into_iter()
            .map(|x| x as f64)
            .collect()
    }
    /// Returns the wavefront error root mean square [m]x10^-`exp`
    pub fn wfe_rms_10e(&mut self, exp: i32) -> Vec<f64> {
        unsafe {
            self._c_.wavefront.rms(self._wfe_rms.as_mut_ptr());
        }
        self._wfe_rms
            .iter()
            .map(|x| *x as f64 * 10_f64.powi(-exp))
            .collect()
    }
    pub fn gradients(&mut self) -> Vec<f64> {
        let mut sxy: Vec<Vec<f32>> = vec![vec![0.; self.size as usize]; 2];
        unsafe {
            self._c_.wavefront.gradient_average1(
                sxy[0].as_mut_ptr(),
                sxy[1].as_mut_ptr(),
                self._c_.rays.L as f32,
            )
        }
        sxy.into_iter().flatten().map(|x| x as f64).collect()
    }
    pub fn segment_wfe_rms(&mut self) -> Vec<f64> {
        let mut mask = vec![0i32; self._c_.rays.N_RAY_TOTAL as usize];
        unsafe {
            dev2host_int(
                mask.as_mut_ptr(),
                self._c_.rays.d__piston_mask,
                self._c_.rays.N_RAY_TOTAL,
            );
        }
        self.phase();
        let mut segment_wfe_std: Vec<f64> = Vec::with_capacity(7);
        for k in 1..8 {
            let segment_phase = mask
                .iter()
                .zip(self._phase.iter())
                .filter(|x| *x.0 == k)
                .map(|x| *x.1)
                .collect::<Vec<f32>>();
            let n = segment_phase.len() as f32;
            let mean = segment_phase.iter().sum::<f32>() / n;
            let var = segment_phase
                .iter()
                .map(|x| (x - mean).powi(2))
                .sum::<f32>()
                / n;
            segment_wfe_std.push(var.sqrt() as f64);
        }
        segment_wfe_std
    }
    pub fn segment_wfe_rms_10e(&mut self, exp: i32) -> Vec<f64> {
        self.segment_wfe_rms()
            .into_iter()
            .map(|x| x * 10_f64.powi(-exp))
            .collect()
    }
    pub fn segment_piston(&mut self) -> Vec<f64> {
        let mut mask = vec![0i32; self._c_.rays.N_RAY_TOTAL as usize];
        unsafe {
            dev2host_int(
                mask.as_mut_ptr(),
                self._c_.rays.d__piston_mask,
                self._c_.rays.N_RAY_TOTAL,
            );
        }
        self.phase();
        let mut segment_mean: Vec<f64> = Vec::with_capacity(7);
        for k in 1..8 {
            let segment_phase = mask
                .iter()
                .zip(self._phase.iter())
                .filter(|x| *x.0 == k)
                .map(|x| *x.1)
                .collect::<Vec<f32>>();
            let n = segment_phase.len() as f32;
            let mean = segment_phase.iter().sum::<f32>() / n;
            //let var = segment_phase.iter().map(|x| (x-mean).powi(2)).sum::<f32>()/n;
            segment_mean.push(mean as f64);
        }
        segment_mean
    }
    pub fn segment_piston_10e(&mut self, exp: i32) -> Vec<f64> {
        self.segment_piston()
            .iter()
            .map(|x| x * 10_f64.powi(-exp))
            .collect()
    }
    pub fn segment_mask(&mut self) -> Vec<i32> {
        let mut mask = vec![0i32; self._c_.rays.N_RAY_TOTAL as usize];
        unsafe {
            dev2host_int(
                mask.as_mut_ptr(),
                self._c_.rays.d__piston_mask,
                self._c_.rays.N_RAY_TOTAL,
            );
        }
        mask
    }
    /// Returns the x and y gradient of the wavefront in average over each of the GMT segments
    pub fn segment_gradients(&mut self) -> Vec<f64> {
        let mut sxy: Vec<Vec<f32>> = vec![vec![0.; 7 * self.size as usize]; 2];
        unsafe {
            self._c_.wavefront.segments_gradient_averageFast(
                sxy[0].as_mut_ptr(),
                sxy[1].as_mut_ptr(),
                self._c_.rays.L as f32,
                self._c_.rays.d__piston_mask,
            );
        }
        sxy.into_iter()
            .flat_map(|x| x.into_iter().map(|x| x as f64).collect::<Vec<f64>>())
            .collect()
    }
    /// Returns the x and y gradient of the wavefront in average over each lenslet of a `n_lenslet`x`n_lenslet` array, the gradients are saved in `Centroiding`
    pub fn lenslet_gradients(
        &mut self,
        n_lenslet: i32,
        _lenslet_size: f64,
        data: &mut Centroiding,
    ) {
        let lenslet_size = self.pupil_size / n_lenslet as f64;
        unsafe {
            if data.n_valid_lenslet < data.n_lenslet_total {
                self._c_.wavefront.finite_difference1(
                    data.__mut_ceo__().0.d__cx,
                    data.__mut_ceo__().0.d__cy,
                    n_lenslet,
                    lenslet_size as f32,
                    data.__mut_ceo__().1,
                );
            } else {
                self._c_.wavefront.finite_difference(
                    data.__mut_ceo__().0.d__cx,
                    data.__mut_ceo__().0.d__cy,
                    n_lenslet,
                    lenslet_size as f32,
                );
            }
        }
    }
    /// Resets the rays and the wavefront to their original state
    pub fn reset(&mut self) {
        unsafe {
            self._c_.wavefront.reset();
            self._c_.reset_rays();
        }
    }
    /// Updates the `zenith` and `azimuth` of the `Source`
    pub fn update(&mut self, mut zenith: Vec<f64>, mut azimuth: Vec<f64>) {
        unsafe {
            self._c_.update_directions(
                zenith.as_mut_ptr(),
                azimuth.as_mut_ptr(),
                zenith.len() as i32,
            );
        }
    }
    /// Adds `phase` to the `Source` wavefront
    pub fn add(&mut self, phase: &mut Cu<Single>) -> &mut Self {
        unsafe {
            self._c_.wavefront.add_phase(1.0, phase.as_mut_ptr());
        }
        self
    }
    pub fn sub(&mut self, phase: &mut Cu<Single>) -> &mut Self {
        unsafe {
            self._c_.wavefront.add_phase(-1.0, phase.as_mut_ptr());
        }
        self
    }
    /// Adds `phase` to the `Source` wavefront
    pub fn add_same(&mut self, phase: &mut Cu<Single>) -> &mut Self {
        unsafe {
            self._c_.wavefront.add_same_phase(1.0, phase.as_mut_ptr());
        }
        self
    }
    /// Returns the wavefront phase \[m\] in the exit pupil of the telescope
    pub fn phase(&mut self) -> &Vec<f32> {
        unsafe {
            dev2host(
                self._phase.as_mut_ptr(),
                self._c_.wavefront.phase,
                self._c_.wavefront.N_PX,
            );
        }
        &self._phase
    }
    pub fn phase_as_ptr(&mut self) -> Cu<Single> {
        let mut phase: Cu<Single> = Cu::vector(self._c_.wavefront.N_PX as usize);
        phase.from_ptr(self._c_.wavefront.phase);
        phase
    }
    /// Returns the wavefront amplitude in the exit pupil of the telescope
    pub fn amplitude(&mut self) -> Vec<f32> {
        let n = self._c_.wavefront.N_PX;
        let mut a = vec![0f32; n as usize];
        unsafe {
            dev2host(a.as_mut_ptr(), self._c_.wavefront.amplitude, n);
        }
        a
    }
    /// Returns the rays \[x,y,z\] coordinates
    ///
    /// Returns the coordinates as \[x1,y1,z1,x2,y2,z2,...\]
    pub fn rays_coordinates(&mut self) -> Vec<f64> {
        let n = 3 * self._c_.rays.N_RAY_TOTAL as usize;
        let mut d_xyz = Cu::<Double>::vector(n);
        //        let mut d_xyz: Cu<Double> = vec![0f64; n].into();
        unsafe {
            self._c_.rays.get_coordinates(d_xyz.malloc().as_mut_ptr());
        }
        d_xyz.into()
    }
    /// Returns the flux integrated in `n_let`X`n_let` bins
    pub fn fluxlet(&mut self, n_let: usize) -> Vec<f32> {
        let m = (self.pupil_sampling as usize - 1) / n_let;
        assert_eq!(m * n_let + 1, self.pupil_sampling as usize);
        let n = self.pupil_sampling as usize;
        let a = self.amplitude();
        let mut f = vec![0f32; (n_let * n_let) as usize];
        for i_let in 0..n_let {
            let ui = (m * i_let) as usize;
            for j_let in 0..n_let {
                let uj = (m * j_let) as usize;
                let mut s = 0f32;
                for i in 0..m as usize + 1 {
                    for j in 0..m as usize + 1 {
                        let k = ui + i + n * (uj + j);
                        s += a[k];
                    }
                }
                f[i_let + n_let * j_let] = s;
            }
        }
        f
    }
    /// Returns a binary mask where the flux integrated in `n_let`X`n_let` bins is greater or equal to the maximum integrated flux X `flux_threshold`
    pub fn masklet(&mut self, n_let: usize, flux_threshold: f32) -> Vec<i8> {
        let f = self.fluxlet(n_let);
        let f_max = f.iter().cloned().fold(-f32::INFINITY, f32::max);
        let t = flux_threshold * f_max;
        f.iter().map(|x| if *x >= t { 1i8 } else { 0i8 }).collect()
    }
    /// Propagates a `Source` through a `system` that implements the `Propagation` trait
    pub fn through<T: Propagation>(&mut self, system: &mut T) -> &mut Self {
        system.propagate(self);
        self
    }
    /// Returns the number of photon [m^-2.s^-1]
    pub fn n_photon(&mut self) -> Vec<f32> {
        self.magnitude
            .clone()
            .iter()
            .map(|m| unsafe { self._c_.n_photon1(*m) })
            .collect()
    }
    /// Returns the light collecting area
    pub fn light_collecting_area(&self) -> f32 {
        self._c_.rays.V.area
    }
    /// Return the source rays
    pub fn rays(&mut self) -> Rays {
        Rays {
            _c_: &mut self._c_.rays,
        }
    }
}
impl Drop for Source {
    /// Frees CEO memory before dropping `Source`
    fn drop(&mut self) {
        unsafe {
            self._c_.cleanup();
        }
    }
}
impl Default for Source {
    fn default() -> Self {
        Self::empty()
    }
}

pub struct Rays<'a> {
    _c_: &'a mut bundle,
}
impl<'a> Rays<'a> {
    /// Returns the rays \[x,y,z\] coordinates
    ///
    /// Returns the coordinates as [x1,y1,z1,x2,y2,z2,...]
    pub fn coordinates(&mut self) -> Vec<f64> {
        let n = 3 * self._c_.N_RAY_TOTAL as usize;
        let mut d_xyz = Cu::<Double>::vector(n);
        unsafe {
            self._c_.get_coordinates(d_xyz.malloc().as_mut_ptr());
        }
        d_xyz.into()
    }
    /// Returns the rays \[k,l,m\] directions
    ///
    /// Returns the directions as [k1,l1,m1,k2,l2,m2,...]
    pub fn directions(&mut self) -> Vec<f64> {
        let n = 3 * self._c_.N_RAY_TOTAL as usize;
        let mut d_klm = Cu::<Double>::vector(n);
        unsafe {
            self._c_.get_directions(d_klm.malloc().as_mut_ptr());
        }
        d_klm.into()
    }
    /// Returns the rays optical path difference
    pub fn opd(&mut self) -> Vec<f64> {
        let n = self._c_.N_RAY_TOTAL as usize;
        let mut d_opd = Cu::<Double>::vector(n);
        //        let mut d_opd: Cu<Double> = vec![0f64; n].into();
        unsafe {
            self._c_
                .get_optical_path_difference(d_opd.malloc().as_mut_ptr());
        }
        d_opd.into()
    }
}

#[cfg(test)]
mod tests {
    //    use super::*;
    //    use crate::Gmt;

    /*
        #[test]
        fn source_piston() {
            let mut src = Source::new(1, 25.5, 1001);
            src.build("V", vec![0.0], vec![0.0], vec![0.0]);
            let mut gmt = Gmt::new();
            gmt.build(1, None);
            let p0 = src.through(&mut gmt).xpupil().segment_piston_10e(-9);
            let rt = vec![vec![0f64, 0f64, 1e-6, 0f64, 0f64, 0f64]; 7];
            gmt.update(None, Some(&rt), None, None);
            let p = src.through(&mut gmt).xpupil().segment_piston_10e(-9);
            let dp = p
                .iter()
                .zip(p0.iter())
                .map(|x| x.0 - x.1)
                .collect::<Vec<f32>>();
            println!("{:?}", dp);
        }
    */
    /*
        #[test]
        fn source_fluxlet() {
            let n_let = 48usize;
            let mut src = Source::new(1, 25.5, n_let as i32 * 16 + 1);
            src.build("V", vec![0.0], vec![0.0], vec![0.0]);
            let mut gmt = Gmt::new();
            gmt.build(1, None);
            let f = src.through(&mut gmt).xpupil().fluxlet(n_let);
            for i in 0..n_let {
                for j in 0..n_let {
                    let k = i + n_let * j;
                    print!("{:3.0},", f[k])
                }
                println!("");
            }
            let f_max = f.iter().cloned().fold(-f32::INFINITY, f32::max);
            println!("Flux max: {}", f_max);
            let t = 0.9;
            let nv = f
                .iter()
                .cloned()
                .filter(|x| x >= &(t * f_max))
                .collect::<Vec<f32>>()
                .len();
            println!("# of valid let: {}", nv);
            assert_eq!(nv, 1144);
        }
    */
    #[test]
    /*
        fn source_masklet() {
            let n_let = 48usize;
            let mut src = Source::new(1, 25.5, n_let as i32 * 16 + 1);
            src.build("V", vec![0.0], vec![0.0], vec![0.0]);
            let mut gmt = Gmt::new();
            gmt.build(1, None);
            let m = src.through(&mut gmt).xpupil().masklet(n_let, 0.9);
            let nv = m.iter().fold(0u32, |a, x| a + *x as u32);
            println!("# of valid let: {}", nv);
            assert_eq!(nv, 1144);
        }
    */
    #[test]
    fn source_field_delaunay21() {
        use crate::{ceo, Conversion};
        let src = ceo!(SourceBuilder, field_delaunay21 = []);
        src.zenith
            .iter()
            .zip(src.azimuth.iter())
            .enumerate()
            .for_each(|x| {
                println!(
                    "#{:2}: {:.3}arcmin - {:7.3}degree",
                    x.0,
                    x.1 .0.to_arcmin(),
                    x.1 .1.to_degrees()
                );
            });
    }
}
