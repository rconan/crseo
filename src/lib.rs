//!
//! # CEO wrapper crate
//!
//! The CEO wrapper is the interface to [CEO CUDA API](https://github.com/rconan/CEO).
//! CEO elements are created using the builder associated to each element.
//!
//! For example, the default CEO elements `Gmt` and `Source` are built with:
//! ```rust
//! use ceo::ceo;
//! let mut gmt = ceo!(GMT);
//! let mut src = ceo!(SOURCE);
//! src.through(&mut gmt).xpupil();
//! println!("WFE RMS: {:?}nm",src.wfe_rms_10e(-9));
//! ```
//! [`ceo!`](macro.ceo.html) is a macro that incorporates the necessary boilerplate code to create CEO elements.

use ffi::{mask, set_device};
use skyangle::*;
use std::{error::Error, fmt};

#[cfg(feature = "analytic")]
pub mod analytic;
pub mod atmosphere;
pub mod calibrations;
pub mod centroiding;
pub mod cu;
pub mod error;
pub mod fwhm;
pub mod gmt;
pub mod imaging;
pub mod lmmse;
pub mod pssn;
pub mod raytracing;
//pub mod sensitivities;
pub mod piston_sensor;
pub mod source;
pub mod wavefrontsensor;

#[doc(inline)]
pub use atmosphere::{Atmosphere, AtmosphereBuilder};
#[doc(inline)]
pub use calibrations::Calibration;
#[doc(inline)]
pub use centroiding::Centroiding;
#[doc(inline)]
pub use cu::Cu;
#[doc(inline)]
pub use error::CrseoError;
#[doc(inline)]
pub use fwhm::Fwhm;
#[doc(inline)]
pub use gmt::{Gmt, GmtBuilder};
#[doc(inline)]
pub use imaging::Imaging;
#[doc(inline)]
pub use lmmse::{LinearMinimumMeanSquareError, LinearMinimumMeanSquareErrorBuilder};
#[doc(inline)]
pub use pssn::{PSSn, PSSnBuilder, PSSnEstimates};
//#[doc(inline)]
//pub use sensitivities::OpticalSensitivities;
#[doc(inline)]
pub use piston_sensor::{PistonSensor, PistonSensorBuilder};
#[doc(inline)]
pub use source::Propagation;
#[doc(inline)]
pub use source::{Source, SourceBuilder};
#[doc(inline)]
pub use wavefrontsensor::{
    Diffractive, Geometric, ShackHartmann, ShackHartmannBuilder, SH24, SH48,
};

pub type GeometricShackHartmann = ShackHartmann<wavefrontsensor::Geometric>;

pub mod prelude {
    pub use super::{
        calibrations, Atmosphere, Builder, Calibration, Diffractive, FromBuilder, Geometric, Gmt,
        LinearMinimumMeanSquareError, PSSn, ShackHartmann, Source, WavefrontSensor,
        WavefrontSensorBuilder, SH24, SH48,
    };
}

/// CEO macro builder
///
/// One macro to rule them all, one macro to find them, one macro to bring them all and in the darkness bind them all
///
/// # Examples
///
///  * GMT
///
/// ```
/// use ceo::ceo;
/// let gmt = ceo!(GMT, set_m1_n_mode = [27], set_m2_n_mode = [123]);
/// ```
///
///  * Geometric Shack-Hartmann
///
/// ```
/// use ceo::ceo;
/// let mut wfs = ceo!(
///     SHACKHARTMANN:Geometric,
///     n_sensor = [1],
///     lenslet_array = [48, 16, 25.5 / 48f64]
/// );
/// let mut src = ceo!(SOURCE, pupil_sampling = [48 * 16 + 1]);
/// let mut gmt = ceo!(GMT);
/// src.through(&mut gmt).xpupil().through(&mut wfs);
/// println!("WFE RMS: {:.3}nm", src.wfe_rms_10e(-9)[0]);
/// ```
///
///  * Diffractive Shack-Hartmann
///
/// ```
/// use ceo::ceo;
/// let mut wfs = ceo!(
///     SHACKHARTMANN<Diffractive>,
///     n_sensor = [1],
///     lenslet_array = [48, 16, 25.5 / 48f64],
///     detector = [8, Some(24), None]
/// );
/// let mut src = ceo!(SOURCE, pupil_sampling = [48 * 16 + 1]);
/// let mut gmt = ceo!(GMT);
/// src.through(&mut gmt).xpupil().through(&mut wfs);
/// println!("WFE RMS: {:.3}nm", src.wfe_rms_10e(-9)[0]);
/// ```
#[macro_export]
macro_rules! ceo {
    ($element:ident) => {
        $crate::Builder::build(<$crate::$element as $crate::Builder>::new()).unwrap()
    };
    ($element:ident, $($arg:ident = [$($val:expr),*]),*) => {
        $crate::Builder::build(<$crate::$element as $crate::Builder>::new()$(.$arg($($val),*))*).unwrap()
    };
    ($element:ident:$model:ident) => {
        $crate::Builder::build(<$crate::$element<$crate::$model> as $crate::Builder>::new()).unwrap()
    };
    ($element:ident:$model:ident, $($arg:ident = [$($val:expr),*]),*) => {
        $crate::Builder::build(<$crate::$element<$crate::$model> as $crate::Builder>::new()$(.$arg($($val),*))*).unwrap()
    };
}
/*
macro_rules! gmt {
    ($($arg:ident = $val:expr),*) => {
        ceo!(element::GMT,$($arg = $val),*)
    };
}
*/
#[derive(Debug)]
pub struct CeoError<T>(T);
impl<T: std::fmt::Debug> Error for CeoError<T> {}
impl<T: std::fmt::Debug> fmt::Display for CeoError<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CEO {:?} builder has failed!", self.0)
    }
}

pub type Result<T> = std::result::Result<T, CrseoError>;
/// CEO builder type trait
pub trait Builder: Default {
    type Component;
    fn new() -> Self {
        Default::default()
    }
    fn build(self) -> Result<Self::Component>;
}

pub trait FromBuilder {
    type ComponentBuilder: Builder;
    fn builder() -> Self::ComponentBuilder {
        Self::ComponentBuilder::new()
    }
}

/// Interface for wavefront sensor builders
pub trait WavefrontSensorBuilder {
    fn guide_stars(&self, template: Option<SourceBuilder>) -> SourceBuilder {
        template.unwrap_or_default()
    }
    fn detector_noise_specs(self, _noise_specs: imaging::NoiseDataSheet) -> Self
    where
        Self: Sized,
    {
        self
    }
    fn decouple(
        &self,
        _gmt_builder: GmtBuilder,
        _src: &mut crate::Source,
        _threshold: f64,
    ) -> Result<Vec<i32>> {
        Ok(Vec::new())
    }
}

/// Interface for wavefront sensors
pub trait WavefrontSensor: Propagation + Send {
    fn calibrate(&mut self, src: &mut Source, threshold: f64);
    fn reset(&mut self);
    fn process(&mut self);
    fn readout(&mut self);
    fn data(&mut self) -> Vec<f64>;
    fn frame(&self) -> Option<Vec<f32>>;
    fn n_frame(&self) -> usize;
    fn valid_lenslet_from(&mut self, wfs: &mut dyn WavefrontSensor);
    fn valid_lenslet(&mut self) -> &mut mask;
    fn n_valid_lenslet(&mut self) -> Vec<usize>;
}

impl<'a, T: WavefrontSensor + ?Sized> WavefrontSensor for &'a mut Box<T> {
    fn calibrate(&mut self, src: &mut Source, threshold: f64) {
        (**self).calibrate(src, threshold);
    }

    fn reset(&mut self) {
        (**self).reset();
    }

    fn process(&mut self) {
        (**self).process();
    }

    fn readout(&mut self) {
        (**self).readout();
    }

    fn data(&mut self) -> Vec<f64> {
        (**self).data()
    }

    fn frame(&self) -> Option<Vec<f32>> {
        (**self).frame()
    }

    fn n_frame(&self) -> usize {
        (**self).n_frame()
    }

    fn valid_lenslet_from(&mut self, wfs: &mut dyn WavefrontSensor) {
        (**self).valid_lenslet_from(wfs)
    }

    fn valid_lenslet(&mut self) -> &mut mask {
        (**self).valid_lenslet()
    }

    fn n_valid_lenslet(&mut self) -> Vec<usize> {
        (**self).n_valid_lenslet()
    }
}
impl<'a, T: WavefrontSensor + ?Sized> Propagation for &'a mut Box<T> {
    fn propagate(&mut self, src: &mut Source) {
        (**self).propagate(src);
    }

    fn time_propagate(&mut self, secs: f64, src: &mut Source) {
        (**self).time_propagate(secs, src);
    }
}

pub fn set_gpu(id: i32) {
    unsafe {
        set_device(id);
    }
}

use cu::Single;
#[derive(Clone, Debug)]
pub struct Mask {
    _c_: mask,
}
impl Mask {
    pub fn new() -> Self {
        Mask {
            _c_: Default::default(),
        }
    }
    pub fn build(&mut self, n_el: usize) -> &mut Self {
        unsafe { self._c_.setup(n_el as i32) }
        self
    }
    pub fn filter(&mut self, f: &mut Cu<Single>) -> &mut Self {
        unsafe {
            self._c_.alter(f.as_ptr());
            self._c_.set_index();
        }
        self
    }
    pub fn nnz(&self) -> usize {
        self._c_.nnz as usize
    }
    pub fn as_mut_prt(&mut self) -> *mut mask {
        &mut self._c_
    }
    pub fn as_raw_mut_ptr(&mut self) -> &mut mask {
        &mut self._c_
    }
    pub fn as_mut(&mut self) -> &mut mask {
        &mut self._c_
    }
}
impl Default for Mask {
    fn default() -> Self {
        Self::new()
    }
}

/*
pub struct CuFloat {
    _c_: gpu_float,
    host_data: Vec<f32>,
}
impl CuFloat {
    pub fn new() -> CuFloat {
        CuFloat {
            _c_: unsafe { mem::zeroed() },
            host_data: vec![],
        }
    }
    pub fn malloc(&mut self, len: usize) -> &mut Self {
        unsafe {
            self._c_.setup1(len as i32);
            self._c_.dev_malloc();
        }
        self
    }
    pub fn into(&mut self, host_data: &mut Vec<f32>) -> &mut Self {
        unsafe {
            self._c_.host_data = host_data.as_mut_ptr();
            self._c_.host2dev();
        }
        self
    }
    pub fn from(&mut self) -> Vec<f32> {
        self.host_data = vec![0f32; self._c_.N as usize];
        unsafe {
            self._c_.host_data = self.host_data.as_mut_ptr();
            self._c_.dev2host()
        }
        self.host_data.clone()
    }
    pub fn as_mut_ptr(&mut self) -> *mut f32 {
        self._c_.dev_data
    }
    pub fn mv(&mut self, y: &mut CuFloat, x: &mut CuFloat) -> &mut Self {
        unsafe {
            self._c_.mv(&mut y._c_, &mut x._c_);
        }
        self
    }
    pub fn qr(&mut self, m: i32) -> &mut Self {
        unsafe {
            self._c_.qr(m);
        }
        self
    }
    pub fn qr_solve(&mut self, x: &mut CuFloat, b: &mut CuFloat) -> &mut Self {
        unsafe {
            self._c_.qr_solve(&mut x._c_, &mut b._c_);
        }
        self
    }
}
impl Drop for CuFloat {
    fn drop(&mut self) {
        unsafe { self._c_.free_dev() }
    }
}

pub fn qr(tau: &mut CuFloat, a: &mut CuFloat, m: i32, n: i32) {
    unsafe { geqrf(tau._c_.dev_data, a._c_.dev_data, m, n) }
}
pub fn qtb(b: &mut CuFloat, m: i32, a: &mut CuFloat, tau: &mut CuFloat, n: i32) {
    unsafe { ormqr(b._c_.dev_data, m, a._c_.dev_data, tau._c_.dev_data, n) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    #[test]
    fn cufloat_mv() {
        let mut a = CuFloat::new();
        a.malloc(9)
            .into(&mut (0..9).map(|x| x as f32).collect::<Vec<f32>>());
        let mut x = CuFloat::new();
        x.malloc(3).into(&mut vec![1f32; 3]);
        let mut y = CuFloat::new();
        y.malloc(3);
        a.mv(&mut y, &mut x);
        println!("R: {:?}", y.from());
        assert_eq!(y.from(), vec![9f32, 12f32, 15f32]);
    }

    #[test]
    fn cufloat_qr() {
        let mut a = CuFloat::new();
        a.malloc(12)
            .into(&mut vec![
                1f32, 4f32, 2f32, 1f32, 2f32, 5f32, 1f32, 1f32, 3f32, 6f32, 1f32, 1f32,
            ])
            .qr(4);
        println!("a: {:?}", a.from());
        let mut b = CuFloat::new();
        b.malloc(4).into(&mut vec![6f32, 15f32, 4f32, 3f32]);
        let mut x = CuFloat::new();
        x.malloc(3);
        a.qr_solve(&mut x, &mut b);
        println!("x: {:?}", x.from());
    }

    #[test]
    fn cufloat_bigqr() {
        let m = 5000;
        let n = 700;
        let p = m * n;
        let mut rng = rand::thread_rng();
        let mut a = CuFloat::new();
        a.malloc(p).into(
            &mut (0..p)
                .map(|_| rng.gen_range(-100, 100) as f32)
                .collect::<Vec<f32>>(),
        );
        //println!("a: {:?}",a.from());
        let mut b = CuFloat::new();
        b.malloc(m);
        {
            let mut x = CuFloat::new();
            x.malloc(n).into(&mut vec![1f32; n]);
            a.mv(&mut b, &mut x);
        }
        //println!("b: {:?}",b.from());
        a.qr(m as i32);
        //println!("a: {:?}",a.from());
        let mut x = CuFloat::new();
        x.malloc(n);
        a.qr_solve(&mut x, &mut b);
        let sx = x.from().iter().sum::<f32>();
        println!("sum of x: {:?}", sx);
        assert!((sx - (n as f32)).abs() < 1e-6);
    }
}
*/
