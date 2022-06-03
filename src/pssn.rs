use crate::FromBuilder;

use super::{cu, Builder, Cu, Propagation, Result, Source, SourceBuilder};
use ffi::pssn;
use serde::ser::{Serialize, SerializeStruct, Serializer};
use std::fmt;

/// CEO PSSn estimator
///
#[derive(Debug, Clone)]
pub struct TelescopeError;
#[derive(Debug, Clone)]
pub struct AtmosphereTelescopeError;
pub trait PSSnErrors {}
impl PSSnErrors for TelescopeError {}
impl PSSnErrors for AtmosphereTelescopeError {}
pub struct PSSn<S> {
    pub _c_: pssn,
    pub r0_at_zenith: f32,
    pub oscale: f32,
    pub zenith_angle: f32,
    pub wavelength: f32,
    /// PSSn estimates
    pub estimates: Vec<f32>,
    pub mode: std::marker::PhantomData<S>,
    pub otf: Vec<f32>,
}

/// [`CEO`](../struct.CEO.html#impl-1) [`PSSn`](../struct.PSSn.html) builder type
pub struct PSSnBuilder<T> {
    pub r0_at_zenith: f64,
    pub oscale: f64,
    pub zenith_angle: f64,
    src: Source,
    marker: std::marker::PhantomData<T>,
}
impl<T: PSSnErrors> PartialEq for PSSnBuilder<T> {
    fn eq(&self, other: &Self) -> bool {
        self.r0_at_zenith == other.r0_at_zenith
            && self.oscale == other.oscale
            && self.zenith_angle == other.zenith_angle
            && self.src == other.src
    }
}
/// Default properties:
///  * r0           : 16cm
///  * L0           : 25m
///  * zenith angle : 30 degrees
impl<T: PSSnErrors> Default for PSSnBuilder<T> {
    fn default() -> Self {
        PSSnBuilder {
            r0_at_zenith: 0.16,
            oscale: 25.0,
            zenith_angle: 30_f64.to_radians(),
            src: SourceBuilder::default().build().unwrap(),
            marker: std::marker::PhantomData,
        }
    }
}
impl<T: PSSnErrors> PSSnBuilder<T> {
    pub fn r0_at_zenith(self, r0_at_zenith: f64) -> Self {
        Self {
            r0_at_zenith,
            ..self
        }
    }
    pub fn outer_scale(self, oscale: f64) -> Self {
        Self { oscale, ..self }
    }
    pub fn zenith_angle(self, zenith_angle_degree: f64) -> Self {
        Self {
            zenith_angle: zenith_angle_degree.to_radians(),
            ..self
        }
    }
    pub fn source(self, src: &Source) -> Self {
        Self {
            src: SourceBuilder::from(src).build().unwrap(),
            ..self
        }
    }
}
impl<T: PSSnErrors> Builder for PSSnBuilder<T> {
    type Component = PSSn<T>;
    fn build(self) -> Result<PSSn<T>> {
        let mut src = self.src;
        let mut pssn = PSSn::<T> {
            _c_: Default::default(),
            r0_at_zenith: self.r0_at_zenith as f32,
            oscale: self.oscale as f32,
            zenith_angle: self.zenith_angle as f32,
            wavelength: src.wavelength() as f32,
            estimates: vec![],
            mode: std::marker::PhantomData,
            otf: Vec::new(),
        };
        let mut gmt = super::ceo!(GmtBuilder);
        src.through(&mut gmt).xpupil();
        unsafe {
            pssn._c_.setup(src.as_raw_mut_ptr(), pssn.r0(), pssn.oscale);
        }
        pssn.estimates = vec![0.0; pssn._c_.N as usize];
        Ok(pssn)
    }
}
impl<S: PSSnErrors> FromBuilder for PSSn<S> {
    type ComponentBuilder = PSSnBuilder<S>;
}
impl<S: PSSnErrors> PSSn<S> {
    /// Creates a new `PSSn` with r0=16cm at zenith, L0=25m a zenith distance of 30 degrees
    pub fn new() -> PSSn<S> {
        PSSn {
            _c_: Default::default(),
            r0_at_zenith: 0.16,
            oscale: 25.0,
            zenith_angle: 30_f32.to_radians(),
            wavelength: 500e-9,
            estimates: vec![],
            mode: std::marker::PhantomData,
            otf: Vec::new(),
        }
    }
    /// Creates a new `PSSn` from r0 at zenith and L0 a zenith distance of 30 degrees
    pub fn from_r0_and_outerscale(r0_at_zenith: f32, oscale: f32) -> PSSn<S> {
        PSSn {
            _c_: Default::default(),
            r0_at_zenith,
            oscale,
            zenith_angle: 30_f32.to_radians(),
            wavelength: 500e-9,
            estimates: vec![],
            mode: std::marker::PhantomData,
            otf: Vec::new(),
        }
    }
    /// Initializes PSSn atmosphere and telescope transfer function from a `Source` object
    pub fn build(&mut self, src: &mut Source) -> &mut Self {
        unsafe {
            self._c_.setup(src.as_raw_mut_ptr(), self.r0(), self.oscale);
        }
        self.estimates = vec![0.0; self._c_.N as usize];
        self
    }
    /// Integrates the `Source` optical transfer function
    pub fn accumulate(&mut self, src: &mut Source) {
        unsafe {
            self._c_.otf(src.as_raw_mut_ptr());
        }
    }
    /// Integrates the `Source` optical transfer function
    pub fn integrate(&mut self, src: &mut Source) {
        unsafe {
            self._c_.otf(src.as_raw_mut_ptr());
        }
    }
    /// Resets the `Source` optical transfer function to its initial value
    pub fn reset(&mut self) -> &mut Self {
        self._c_.N_O = 0;
        self
    }
    /// Computes `PSSn` spatial uniformity
    pub fn spatial_uniformity(&mut self) -> f32 {
        let mut pssn_values = self.estimates.clone();
        pssn_values.sort_by(|a, b| a.partial_cmp(b).unwrap());
        100. * ((pssn_values.len() as f32)
            * (*pssn_values.last().unwrap() - *pssn_values.first().unwrap()))
            / pssn_values.iter().sum::<f32>()
    }
    pub fn r0(&self) -> f32 {
        (self.r0_at_zenith.powf(-5_f32 / 3_f32) / self.zenith_angle.cos()).powf(-3_f32 / 5_f32)
            * (self.wavelength / 0.5e-6_f32).powf(1.2_f32)
    }
    pub fn r0_at_z(r0_at_zenith: f32, zenith_angle: f32) -> f32 {
        (r0_at_zenith.powf(-5_f32 / 3_f32) / zenith_angle.cos()).powf(-3_f32 / 5_f32)
    }
    pub fn xotf(&mut self) -> &Self {
        let mut d_otf = Cu::<cu::Single>::vector(2 * self._c_.NN as usize);
        d_otf.malloc();
        unsafe {
            self._c_.xotf(d_otf.as_ptr());
        }
        self.otf = d_otf.from_dev();
        self
    }
    pub fn telescope_otf(&mut self) -> Vec<f32> {
        let mut d_otf = Cu::<cu::Single>::vector(2 * self._c_.NN as usize);
        d_otf.malloc();
        unsafe {
            self._c_.O0(d_otf.as_ptr());
        }
        d_otf.from_dev()
    }
    pub fn telescope_error_otf(&mut self) -> Vec<f32> {
        let mut d_otf = Cu::<cu::Single>::vector(2 * self._c_.NN as usize);
        d_otf.malloc();
        unsafe {
            self._c_.O(d_otf.as_ptr());
        }
        d_otf.from_dev()
    }
    pub fn telescope_error_into_otf(&mut self) -> &mut Self {
        let mut d_otf = Cu::<cu::Single>::vector(2 * self._c_.NN as usize);
        d_otf.malloc();
        unsafe {
            self._c_.O(d_otf.as_ptr());
        }
        self.otf = d_otf.from_dev();
        self
    }
    pub fn buffer_otf(&mut self) -> Vec<f32> {
        let mut d_otf = Cu::<cu::Single>::vector(2 * self._c_.NN as usize);
        d_otf.malloc();
        unsafe {
            self._c_.B(d_otf.as_ptr());
        }
        d_otf.from_dev()
    }
    pub fn atmosphere_otf(&mut self) -> Vec<f32> {
        let mut d_otf = Cu::<cu::Single>::vector(2 * self._c_.NN as usize);
        d_otf.malloc();
        unsafe {
            self._c_.C(d_otf.as_ptr());
        }
        d_otf.from_dev()
    }
}
impl<T: PSSnErrors> Serialize for PSSn<T> {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("PSSn", 1)?;
        state.serialize_field("r0", &self.r0())?;
        state.serialize_field("L0", &self.oscale)?;
        state.serialize_field("values", &self.estimates)?;
        //state.serialize_field("otf",&self.otf)?;
        state.end()
    }
}
impl PSSn<TelescopeError> {
    /// Estimates the `PSSn` values
    pub fn peek(&mut self) -> &mut Self {
        unsafe { self._c_.eval1(self.estimates.as_mut_ptr()) }
        self
    }
}
impl PSSn<AtmosphereTelescopeError> {
    /// Estimates the `PSSn` values
    pub fn peek(&mut self) -> &mut Self {
        unsafe { self._c_.oeval1(self.estimates.as_mut_ptr()) }
        self
    }
}

pub trait PSSnEstimates: Propagation + Send {
    fn estimates(&mut self) -> Vec<f64>;
}
impl PSSnEstimates for PSSn<TelescopeError> {
    fn estimates(&mut self) -> Vec<f64> {
        self.peek().estimates.iter().map(|x| *x as f64).collect()
    }
}
impl PSSnEstimates for PSSn<AtmosphereTelescopeError> {
    fn estimates(&mut self) -> Vec<f64> {
        self.peek().estimates.iter().map(|x| *x as f64).collect()
    }
}
impl Propagation for Box<dyn PSSnEstimates> {
    fn propagate(&mut self, src: &mut Source) {
        (**self).propagate(src);
    }

    fn time_propagate(&mut self, secs: f64, src: &mut Source) {
        (**self).time_propagate(secs, src);
    }
}
impl<T> PSSnEstimates for Box<T>
where
    Box<T>: Propagation,
    T: PSSnEstimates + ?Sized,
{
    fn estimates(&mut self) -> Vec<f64> {
        (**self).estimates()
    }
}
impl<S> Drop for PSSn<S> {
    /// Frees CEO memory before dropping `PSSn`
    fn drop(&mut self) {
        unsafe {
            self._c_.cleanup();
        }
    }
}
impl<S> fmt::Display for PSSn<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}]",
            self.estimates
                .iter()
                .map(|x| format!("{:.4}", x))
                .collect::<Vec<String>>()
                .as_slice()
                .join(",")
        )
    }
}

impl<S: PSSnErrors> Propagation for PSSn<S> {
    fn propagate(&mut self, src: &mut Source) {
        self.integrate(src);
    }
    fn time_propagate(&mut self, _secs: f64, src: &mut Source) {
        self.integrate(src);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pssn_new() {
        use crate::{Gmt, Source};
        let mut src = Source::builder().build().unwrap();
        let mut gmt = Gmt::builder().build().unwrap();
        src.through(&mut gmt).xpupil();
        let mut pssn = PSSn::<TelescopeError>::builder().build().unwrap();
        src.through(&mut pssn);
        println!("PSSN: {}", pssn.peek());
    }
}
