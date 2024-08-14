use std::{
    ffi::CString,
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
};

use crate::{AtmosphereError, Builder, CrseoError};

use super::{Atmosphere, RayTracing, TurbulenceProfile};

/// [`CEO`](../struct.CEO.html#impl-6) [`Atmosphere`](../struct.Atmosphere.html) builder type
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AtmosphereBuilder {
    pub r0_at_zenith: f64,
    pub oscale: f64,
    pub zenith_angle: f64,
    pub turbulence: TurbulenceProfile,
    pub ray_tracing: Option<RayTracing>,
}
/// Default properties:
///  * r0           : 16cm
///  * L0           : 25m
///  * zenith angle : 30 degrees
///  * turbulence profile:
///    * n_layer        : 7
///    * altitude       : [25.0, 275.0, 425.0, 1250.0, 4000.0, 8000.0, 13000.0] m
///    * xi0            : [0.1257, 0.0874, 0.0666, 0.3498, 0.2273, 0.0681, 0.0751]
///    * wind speed     : [5.6540, 5.7964, 5.8942, 6.6370, 13.2925, 34.8250, 29.4187] m/s
///    * wind direction : [0.0136, 0.1441, 0.2177, 0.5672, 1.2584, 1.6266, 1.7462] rd
/// * ray tracing : none
impl Default for AtmosphereBuilder {
    fn default() -> Self {
        AtmosphereBuilder {
            r0_at_zenith: 0.16,
            oscale: 25.,
            zenith_angle: 30_f64.to_radians(),
            turbulence: TurbulenceProfile::default(),
            ray_tracing: None,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AtmosphereBuilderError {
    #[error("cannot open`::crseo::AtmosphereBuilder` toml file: {1}")]
    Open(#[source] std::io::Error, PathBuf),
    #[error("cannot create `::crseo::AtmosphereBuilder` toml file: {1}")]
    Create(#[source] std::io::Error, PathBuf),
    #[error("cannot read `::crseo::AtmosphereBuilder` toml file: {1}")]
    Read(#[source] std::io::Error, PathBuf),
    #[error("cannot write `::crseo::AtmosphereBuilder` toml file: {1}")]
    Write(#[source] std::io::Error, PathBuf),
    #[error("cannot deserialize `::crseo::AtmosphereBuilder` from toml")]
    Load(#[from] toml::de::Error),
    #[error("cannot serialize `::crseo::AtmosphereBuilder` into toml")]
    Save(#[from] toml::ser::Error),
}

/// ## `Atmosphere` builder
impl AtmosphereBuilder {
    /// Load the atmospheric builder from a toml
    pub fn load<P: AsRef<Path>>(path: P) -> std::result::Result<Self, AtmosphereBuilderError> {
        let mut file = File::open(&path)
            .map_err(|e| AtmosphereBuilderError::Open(e, path.as_ref().to_path_buf()))?;
        let mut toml = String::new();
        file.read_to_string(&mut toml)
            .map_err(|e| AtmosphereBuilderError::Read(e, path.as_ref().to_path_buf()))?;
        let builder: AtmosphereBuilder = toml::from_str(&toml)?;
        Ok(builder)
    }
    /// Save the atmospheric builder from a toml
    pub fn save<P: AsRef<Path>>(&self, path: P) -> std::result::Result<(), AtmosphereBuilderError> {
        let toml = toml::to_string_pretty(self)?;
        let mut file = File::create(&path)
            .map_err(|e| AtmosphereBuilderError::Create(e, path.as_ref().to_path_buf()))?;
        write!(file, "# ::crseo::AtmosphereBuilder\n\n{}", toml)
            .map_err(|e| AtmosphereBuilderError::Write(e, path.as_ref().to_path_buf()))?;
        Ok(())
    }
    /// Set r0 value taken at pointing the zenith in meters
    pub fn r0_at_zenith(self, r0_at_zenith: f64) -> Self {
        Self {
            r0_at_zenith,
            ..self
        }
    }
    /// Set outer scale value in meters
    pub fn oscale(self, oscale: f64) -> Self {
        Self { oscale, ..self }
    }
    /// Set zenith angle value in radians
    pub fn zenith_angle(self, zenith_angle: f64) -> Self {
        Self {
            zenith_angle,
            ..self
        }
    }
    /// Set the turbulence profile
    pub fn turbulence_profile(self, turbulence: TurbulenceProfile) -> Self {
        Self { turbulence, ..self }
    }
    /// Set a single turbulence layer
    pub fn single_turbulence_layer(
        self,
        altitude: f32,
        wind_speed: Option<f32>,
        wind_direction: Option<f32>,
    ) -> Self {
        Self {
            turbulence: TurbulenceProfile {
                n_layer: 1,
                altitude: vec![altitude],
                xi0: vec![1f32],
                wind_speed: vec![wind_speed.unwrap_or(0f32)],
                wind_direction: vec![wind_direction.unwrap_or(0f32)],
            },
            ..self
        }
    }
    /// Remove a turbulence layer specifield by its zero based index
    pub fn remove_turbulence_layer(self, layer_idx: usize) -> Self {
        let mut turbulence = self.turbulence;
        turbulence.n_layer -= 1;
        turbulence.altitude.remove(layer_idx);
        turbulence.xi0.remove(layer_idx);
        turbulence.wind_speed.remove(layer_idx);
        turbulence.wind_direction.remove(layer_idx);
        Self { turbulence, ..self }
    }
    /// Set parameters for atmosphere ray tracing
    pub fn ray_tracing(self, ray_tracing: RayTracing) -> Self {
        Self {
            ray_tracing: Some(ray_tracing),
            ..self
        }
    }
}
impl Builder for AtmosphereBuilder {
    type Component = Atmosphere;
    /// Build the `Atmosphere`
    fn build(self) -> std::result::Result<Atmosphere, CrseoError> {
        let mut atm = Atmosphere {
            _c_: Default::default(),
            r0_at_zenith: self.r0_at_zenith,
            oscale: self.oscale,
            zenith_angle: self.zenith_angle,
            secs: 0.0,
            //filename: String::new(),
            //k_duration: 0,
            propagate_ptr: |_, _, _| (),
        };
        let secz = 1f64 / atm.zenith_angle.cos();
        let r0 = (atm.r0_at_zenith.powf(-5.0 / 3.0) * secz).powf(-3.0 / 5.0);
        log::info!(
            "Atmosphere r0 at {:.1}degree from zenith: {:.3}m",
            atm.zenith_angle.to_degrees(),
            r0
        );
        let mut altitude = self
            .turbulence
            .altitude
            .iter()
            .map(|x| *x as f32 * secz as f32)
            .collect::<Vec<f32>>();
        let mut wind_speed = self
            .turbulence
            .wind_speed
            .iter()
            .map(|x| *x as f32 / secz as f32)
            .collect::<Vec<f32>>();
        let mut xi0 = self.turbulence.xi0.clone();
        let mut wind_direction = self.turbulence.wind_direction.clone();
        match &self.ray_tracing {
            None => unsafe {
                atm._c_.setup(
                    r0 as f32,
                    self.oscale as f32,
                    self.turbulence.n_layer as i32,
                    altitude.as_mut_ptr(),
                    xi0.as_mut_ptr(),
                    wind_speed.as_mut_ptr(),
                    wind_direction.as_mut_ptr(),
                );
                atm.propagate_ptr = |a, s, t| {
                    let n_xy = s.pupil_sampling;
                    let d_xy = (s.pupil_size / (n_xy - 1) as f64) as f32;
                    a._c_
                        .get_phase_screen4(s.as_raw_mut_ptr(), d_xy, n_xy, d_xy, n_xy, t);
                };
            },
            Some(rtc) => match &rtc.filepath {
                Some(file) => unsafe {
                    let path = Path::new(file).with_extension("").with_extension("toml");
                    if let Ok(builder) = Self::load(&path) {
                        if builder != self {
                            panic!(
                                "{:?} does not match the definition of the AtmosphereBuilder",
                                path
                            );
                        }
                    } else {
                        self.save(&path).map_err(|e| AtmosphereError::from(e))?;
                    }
                    log::info!("Looking up phase screen from file {}", file);
                    atm._c_.setup2(
                        r0 as f32,
                        self.oscale as f32,
                        self.turbulence.n_layer as i32,
                        altitude.as_mut_ptr(),
                        xi0.as_mut_ptr(),
                        wind_speed.as_mut_ptr(),
                        wind_direction.as_mut_ptr(),
                        rtc.width,
                        rtc.n_width_px,
                        rtc.field_size,
                        rtc.duration,
                        CString::new(file.to_owned().into_bytes())
                            .unwrap()
                            .into_raw(),
                        rtc.n_duration.unwrap_or(1),
                    );
                    atm.propagate_ptr = |a, s, t| {
                        let n_xy = s.pupil_sampling;
                        let d_xy = (s.pupil_size / (n_xy - 1) as f64) as f32;
                        a._c_
                            .rayTracing1(s.as_raw_mut_ptr(), d_xy, n_xy, d_xy, n_xy, t);
                    };
                },
                None => unsafe {
                    atm._c_.setup1(
                        r0 as f32,
                        self.oscale as f32,
                        self.turbulence.n_layer as i32,
                        altitude.as_mut_ptr(),
                        xi0.as_mut_ptr(),
                        wind_speed.as_mut_ptr(),
                        wind_direction.as_mut_ptr(),
                        rtc.width,
                        rtc.n_width_px,
                        rtc.field_size,
                        rtc.duration,
                    );
                    atm.propagate_ptr = |a, s, t| {
                        let n_xy = s.pupil_sampling;
                        let d_xy = (s.pupil_size / (n_xy - 1) as f64) as f32;
                        a._c_
                            .rayTracing1(s.as_raw_mut_ptr(), d_xy, n_xy, d_xy, n_xy, t);
                    };
                },
            },
        }
        Ok(atm)
    }
}
