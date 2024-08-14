use serde::{Deserialize, Serialize};
use std::{
    f32,
    ffi::CString,
    ops::{Div, Mul},
    path::Path,
};

use super::{Cu, FromBuilder, Propagation, Single, Source};
use ffi::atmosphere;

mod builder;
pub use builder::{AtmosphereBuilder, AtmosphereBuilderError};

#[derive(Debug, thiserror::Error)]
pub enum AtmosphereError {
    #[error("cannot create `::crseo::AtmosphereBuilder`")]
    Builder(#[from] AtmosphereBuilderError),
}
pub type Result<T> = std::result::Result<T, AtmosphereError>;

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct GmtAtmosphere {
    r0: f32,
    #[serde(rename = "L0")]
    l_not: f32,
    #[serde(rename = "L")]
    length: f32,
    #[serde(rename = "lower_case")]
    nxy_pupil: i32,
    fov: f32,
    duration: f32,
    #[serde(rename = "lower_case")]
    n_duration: i32,
    filename: String,
    #[serde(rename = "lower_case")]
    seed: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[doc(hidden)]
pub struct TurbulenceProfile {
    pub n_layer: usize,
    pub altitude: Vec<f32>,
    pub xi0: Vec<f32>,
    pub wind_speed: Vec<f32>,
    pub wind_direction: Vec<f32>,
}
impl Default for TurbulenceProfile {
    fn default() -> Self {
        TurbulenceProfile {
            n_layer: 7,
            altitude: [25.0, 275.0, 425.0, 1_250.0, 4_000.0, 8_000.0, 13_000.0].to_vec(),
            xi0: [0.1257, 0.0874, 0.0666, 0.3498, 0.2273, 0.0681, 0.0751].to_vec(),
            wind_speed: [5.6540, 5.7964, 5.8942, 6.6370, 13.2925, 34.8250, 29.4187].to_vec(),
            wind_direction: [0.0136, 0.1441, 0.2177, 0.5672, 1.2584, 1.6266, 1.7462].to_vec(),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[doc(hidden)]
pub struct RayTracing {
    pub width: f32,
    pub n_width_px: i32,
    pub field_size: f32,
    pub duration: f32,
    pub filepath: Option<String>,
    pub n_duration: Option<i32>,
}
/// Default properties:
///  * width        : 25.5m
///  * width_px     : 512px
///  * field_size   : 0rd
///  * duration     : 1s
///  * filepath     : None
///  * n_duration   : None
impl Default for RayTracing {
    fn default() -> Self {
        Self {
            width: 25.5,
            n_width_px: 512,
            field_size: 0.0,
            duration: 1.0,
            filepath: None,
            n_duration: None,
        }
    }
}
impl RayTracing {
    /// Size in meters of the phase screen at altitude 0m
    pub fn width(mut self, width: f64) -> Self {
        self.width = width as f32;
        self
    }
    /// Size in pixels of the phase screen at altitude 0m
    pub fn n_width_px(mut self, n_width_px: usize) -> Self {
        self.n_width_px = n_width_px as i32;
        self
    }
    /// Field-of-view in radians
    pub fn field_size(mut self, field_size: f64) -> Self {
        self.field_size = field_size as f32;
        self
    }
    /// Phase screen minimum time length in seconds
    ///
    /// Phase screens of that time length must fit with the GPU memory
    pub fn duration(mut self, duration: f64) -> Self {
        self.duration = duration as f32;
        self
    }
    /// Path where to write the phase screens data file
    pub fn filepath<P: AsRef<Path>>(mut self, filepath: P) -> Self {
        let path = filepath.as_ref();
        self.filepath = Some(path.to_str().unwrap().to_string());
        self
    }
    /// Total number of durations
    ///
    /// The total time length of the phase screens is `n_duration X duration` seconds
    pub fn n_duration(mut self, n_duration: u64) -> Self {
        self.n_duration = Some(n_duration as i32);
        self
    }
}

pub struct Atmosphere {
    _c_: atmosphere,
    pub r0_at_zenith: f64,
    pub oscale: f64,
    pub zenith_angle: f64,
    pub secs: f64,
    //filename: String,
    //k_duration: i32,
    propagate_ptr: fn(&mut Atmosphere, &mut Source, f32),
}
impl FromBuilder for Atmosphere {
    type ComponentBuilder = AtmosphereBuilder;
}
impl Atmosphere {
    /*
        pub fn new() -> Atmosphere {
            Atmosphere {
                _c_: unsafe { mem::zeroed() },
                r0_at_zenith: 0.16,
                oscale: 25.5,
                zenith_angle: 0.0,
                secs: 0.0,
                //filename: String::new(),
                //k_duration: 0,
                propagate_ptr: |_, _, _| (),
            }
        }
        pub fn build(
            &mut self,
            r_not: f32,
            l_not: f32,
            n_layer: i32,
            mut altitude: Vec<f32>,
            mut xi0: Vec<f32>,
            mut wind_speed: Vec<f32>,
            mut wind_direction: Vec<f32>,
        ) -> &mut Self {
            unsafe {
                self._c_.setup(
                    r_not,
                    l_not,
                    n_layer,
                    altitude.as_mut_ptr(),
                    xi0.as_mut_ptr(),
                    wind_speed.as_mut_ptr(),
                    wind_direction.as_mut_ptr(),
                );
            }
            self.propagate_ptr = |a, s, t| unsafe {
                let n_xy = s.pupil_sampling;
                let d_xy = (s.pupil_size / (n_xy - 1) as f64) as f32;
                a._c_
                    .get_phase_screen4(s.as_raw_mut_ptr(), d_xy, n_xy, d_xy, n_xy, t);
            };
            self
        }
    */
    pub fn as_raw_mut_ptr(&mut self) -> &mut atmosphere {
        &mut self._c_
    }
    pub fn raytrace_build(
        &mut self,
        r_not: f32,
        l_not: f32,
        n_layer: i32,
        mut altitude: Vec<f32>,
        mut xi0: Vec<f32>,
        mut wind_speed: Vec<f32>,
        mut wind_direction: Vec<f32>,
        width: f32,
        n_width_px: i32,
        field_size: f32,
        duration: f32,
        filepath: Option<&str>,
        n_duration: Option<i32>,
    ) -> &mut Self {
        match filepath {
            Some(file) => unsafe {
                self._c_.setup2(
                    r_not,
                    l_not,
                    n_layer,
                    altitude.as_mut_ptr(),
                    xi0.as_mut_ptr(),
                    wind_speed.as_mut_ptr(),
                    wind_direction.as_mut_ptr(),
                    width,
                    n_width_px,
                    field_size,
                    duration,
                    CString::new(file.to_owned().into_bytes())
                        .unwrap()
                        .into_raw(),
                    n_duration.unwrap_or(1),
                );
            },
            None => unsafe {
                self._c_.setup1(
                    r_not,
                    l_not,
                    n_layer,
                    altitude.as_mut_ptr(),
                    xi0.as_mut_ptr(),
                    wind_speed.as_mut_ptr(),
                    wind_direction.as_mut_ptr(),
                    width,
                    n_width_px,
                    field_size,
                    duration,
                );
            },
        }
        self.propagate_ptr = |a, s, t| unsafe {
            let n_xy = s.pupil_sampling;
            let d_xy = (s.pupil_size / (n_xy - 1) as f64) as f32;
            a._c_
                .rayTracing1(s.as_raw_mut_ptr(), d_xy, n_xy, d_xy, n_xy, t);
        };
        self
    }
    pub fn gmt_build(&mut self, r_not: f32, l_not: f32) -> &mut Self {
        unsafe {
            self._c_.gmt_setup4(r_not, l_not, 2020);
        }
        self
    }
    pub fn get_phase_values<'a, T>(
        &mut self,
        src: &mut Source,
        x: &'a [T],
        y: &'a [T],
        t: f64,
    ) -> Vec<T>
    where
        &'a [T]: Into<Cu<Single>>,
        Cu<Single>: Into<Vec<T>>,
        T: 'a,
    {
        let n = x.len();
        let mut gx: Cu<Single> = x.into();
        let mut gy: Cu<Single> = y.into();
        let mut ps = Cu::<Single>::vector(n);
        ps.malloc();
        unsafe {
            self._c_.get_phase_screen(
                ps.as_mut_ptr(),
                gx.as_mut_ptr(),
                gy.as_mut_ptr(),
                n as i32,
                src.as_raw_mut_ptr(),
                t as f32,
            )
        }
        ps.into()
    }
    pub fn get_phase_screen<'a, T>(
        &mut self,
        src: &mut Source,
        t: f64,
        (s_x, n_x): (T, usize),
        other_side: Option<(T, usize)>,
    ) -> Vec<T>
    where
        T: 'a + Copy + From<u32> + Div<Output = T> + Mul<Output = T>,
        Vec<T>: Into<Cu<Single>>,
        Cu<Single>: Into<Vec<T>>,
    {
        let (s_y, n_y) = other_side.unwrap_or((s_x, n_x));
        let n = n_x * n_y;
        let mut x: Vec<T> = Vec::with_capacity(n);
        let mut y: Vec<T> = Vec::with_capacity(n);
        let delta_x = s_x / T::try_from(n_x as u32 - 1).unwrap();
        let delta_y = s_y / T::try_from(n_x as u32 - 1).unwrap();
        for i in 0..n_x {
            for j in 0..n_y {
                x.push(delta_x * T::try_from(i as u32).unwrap());
                y.push(delta_y * T::try_from(j as u32).unwrap());
            }
        }
        let mut gx: Cu<Single> = x.into();
        let mut gy: Cu<Single> = y.into();
        let mut ps = Cu::<Single>::vector(n);
        ps.malloc();
        unsafe {
            self._c_.get_phase_screen(
                ps.as_mut_ptr(),
                gx.as_mut_ptr(),
                gy.as_mut_ptr(),
                n as i32,
                src.as_raw_mut_ptr(),
                t as f32,
            )
        }
        ps.into()
    }
    pub fn update_r0(&mut self, new_r0: f64) {
        self._c_.r0 = new_r0 as f32;
    }
    pub fn r0(&self) -> f64 {
        let secz = 1f64 / self.zenith_angle.cos();
        (self.r0_at_zenith.powf(-5.0 / 3.0) * secz).powf(-3.0 / 5.0)
    }
    pub fn reset(&mut self) {
        unsafe {
            self._c_.reset();
        }
    }
}
impl Drop for Atmosphere {
    fn drop(&mut self) {
        unsafe {
            self._c_.cleanup();
        }
    }
}
impl Propagation for Atmosphere {
    fn time_propagate(&mut self, secs: f64, src: &mut Source) {
        (self.propagate_ptr)(self, src, secs as f32);
    }
    fn propagate(&mut self, src: &mut Source) {
        self.time_propagate(self.secs, src)
    }
}

/* #[cfg(test)]
mod tests {
    use super::*;

    // cargo test --release --package crseo --lib  -- atmosphere::tests::atmosphere_new --exact --nocapture
    #[test]
    fn atmosphere_new() {
        crate::ceo!(AtmosphereBuilder);
    }

    // cargo test --release --package crseo --lib  -- atmosphere::tests::dump_toml --exact --nocapture
    #[test]
    fn dump_toml() -> anyhow::Result<()> {
        let builder = AtmosphereBuilder::default().ray_tracing(Default::default());
        builder.save("atm_builder.toml")?;
        Ok(())
    }

    // cargo test --release --package crseo --lib  -- atmosphere::tests::load_toml --exact --nocapture
    #[test]
    fn load_toml() -> anyhow::Result<()> {
        let builder = AtmosphereBuilder::load("atm_builder.toml")?;
        dbg!(&builder);
        Ok(())
    }
}
 */
