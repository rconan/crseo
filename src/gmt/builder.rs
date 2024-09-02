use crate::{Builder, CrseoError};
use serde::{Deserialize, Serialize};
use std::{env, ffi::CString, path::Path};

use super::{Gmt, GmtM1, GmtM2, GmtMx};

#[derive(Default, Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct MirrorBuilder {
    pub mode_type: String,
    pub n_mode: usize,
    pub a: Vec<f64>,
}
impl MirrorBuilder {
    /// Sets the type of mirror modes
    pub fn mode_type(self, mode_type: &str) -> Self {
        Self {
            mode_type: mode_type.into(),
            ..self
        }
    }
    /// Sets the number of modes
    pub fn n_mode(self, n_mode: usize) -> Self {
        Self {
            n_mode,
            a: vec![0f64; 7 * n_mode],
            ..self
        }
    }
    /// Sets the default values of the modal coefficients
    pub fn default_state(self, a: Vec<f64>) -> Self {
        assert!(
            a.len() == 7 * self.n_mode,
            "Incorrect number of modal coeffcients, expected: {}, found: {}",
            7 * self.n_mode,
            a.len()
        );
        Self { a, ..self }
    }
}

pub trait GmtMirrorBuilder<M: GmtMx> {
    fn n_mode(self, n_mode: usize) -> Self;
}

impl GmtMirrorBuilder<GmtM1> for GmtBuilder {
    #[inline]
    fn n_mode(self, n_mode: usize) -> Self {
        Self {
            m1: self.m1.n_mode(n_mode),
            ..self
        }
    }
}

impl GmtMirrorBuilder<GmtM2> for GmtBuilder {
    #[inline]
    fn n_mode(self, n_mode: usize) -> Self {
        Self {
            m2: self.m2.n_mode(n_mode),
            ..self
        }
    }
}

/* pub enum ModeType {
    None,
    Zernike(usize),
    Modes(String),
}
 */

/// `Gmt` builder
///
/// Default properties:
///  - M1:
///    - mode type : "bending modes"
///    - \# mode    : 0
///  - M2:
///    - mode type : "Karhunen-Loeve"
///    - \# mode    : 0
///
/// # Examples
///
/// ```
/// use crseo::{FromBuilder, Builder, Gmt};
/// let mut src = Gmt::builder().build();
/// ```
///
/// ```
/// use crseo::{FromBuilder, Builder, Gmt};
/// let mut gmt = Gmt::builder().m1_n_mode(27).build();
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GmtBuilder {
    pub m1: MirrorBuilder,
    pub m2: MirrorBuilder,
    pub pointing_error: Option<(f64, f64)>,
    pub m1_truss_projection: bool,
}
impl Default for GmtBuilder {
    fn default() -> Self {
        GmtBuilder {
            m1: MirrorBuilder {
                mode_type: "bending modes".into(),
                ..Default::default()
            },
            m2: MirrorBuilder {
                mode_type: "Karhunen-Loeve".into(),
                ..Default::default()
            },
            pointing_error: None,
            m1_truss_projection: true,
        }
    }
}
impl GmtBuilder {
    /// Set the type and number of modes of M1
    pub fn m1(self, mode_type: &str, n_mode: usize) -> Self {
        Self {
            m1: self.m1.mode_type(mode_type).n_mode(n_mode),
            ..self
        }
    }
    pub fn n_mode<M: GmtMx>(self, n_mode: usize) -> Self
    where
        GmtBuilder: GmtMirrorBuilder<M>,
    {
        <Self as GmtMirrorBuilder<M>>::n_mode(self, n_mode)
    }
    /// Set the number of modes of M1
    pub fn m1_n_mode(self, n_mode: usize) -> Self {
        Self {
            m1: self.m1.n_mode(n_mode),
            ..self
        }
    }
    /// Turns the truss projection on M1 on (`true`) or off (`false`)
    pub fn m1_truss_projection(mut self, m1_truss_projection: bool) -> Self {
        self.m1_truss_projection = m1_truss_projection;
        self
    }
    /// Set the default M1 modal coefficients
    pub fn m1_default_state(self, a: Vec<f64>) -> Self {
        Self {
            m1: self.m1.default_state(a),
            ..self
        }
    }
    /// Set the type and number of modes of M2
    pub fn m2(self, mode_type: &str, n_mode: usize) -> Self {
        Self {
            m2: self.m2.mode_type(mode_type).n_mode(n_mode),
            ..self
        }
    }
    /// Set the number of modes of M2
    pub fn m2_n_mode(self, n_mode: usize) -> Self {
        Self {
            m2: self.m2.n_mode(n_mode),
            ..self
        }
    }
    /// Set the default M2 modal coefficients
    pub fn m2_default_state(self, a: Vec<f64>) -> Self {
        Self {
            m2: self.m2.default_state(a),
            ..self
        }
    }
    /// Set the pointing error
    ///
    /// The pointing error is given as the pair (delta_zenith, azimuth) in radians
    pub fn pointing_error(mut self, pointing_error: (f64, f64)) -> Self {
        self.pointing_error = Some(pointing_error);
        self
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GmtModesError {
    #[error("the mirror modes file ({0}) does not exist")]
    Path(String),
    #[error(r#"the environment variable "GMT_MODES_PATH" is not set"#)]
    EnvVar(#[from] std::env::VarError),
}

impl MirrorBuilder {
    fn mode_path(&self) -> std::result::Result<String, GmtModesError> {
        let mode_type = Path::new(&self.mode_type).with_extension("ceo");
        if mode_type.is_file() {
            Ok(mode_type.to_str().unwrap().to_owned())
        } else {
            let env_path = env::var("GMT_MODES_PATH")?;
            let path = Path::new(&env_path).join(mode_type);
            if path.is_file() {
                Ok(path.to_str().unwrap().to_owned())
            } else {
                Err(GmtModesError::Path(path.to_str().unwrap().to_string()))
            }
        }
    }
}
impl Builder for GmtBuilder {
    type Component = Gmt;
    fn build(self) -> std::result::Result<Gmt, CrseoError> {
        let m1_mode_type =
            CString::new(self.m1.mode_path().map_err(|e| super::GmtError::from(e))?)?;
        let m2_mode_type =
            CString::new(self.m2.mode_path().map_err(|e| super::GmtError::from(e))?)?;

        let mut gmt = Gmt {
            m1: self.m1.into(),
            m2: self.m2.into(),
            // m1_n_mode: 0,
            // m2_n_mode: 0,
            // m2_max_n: 0,
            // a1: self.m1.a.clone(),
            // a2: self.m2.a.clone(),
            pointing_error: self.pointing_error,
            m1_truss_projection: self.m1_truss_projection,
        };

        // gmt.m1_n_mode = self.m1.n_mode;
        unsafe {
            let n_mode = gmt.m1.n_mode;
            gmt.m1.setup1(m1_mode_type.into_raw(), 7, n_mode as i32);
        }

        // gmt.m2_n_mode = self.m2.n_mode;
        unsafe {
            let n_mode = gmt.m2.n_mode;
            gmt.m2.setup1(m2_mode_type.into_raw(), 7, n_mode as i32);
        }
        gmt.reset();
        Ok(gmt)
    }
}
impl From<&Gmt> for GmtBuilder {
    fn from(gmt: &Gmt) -> Self {
        Self {
            m1: gmt.get_m1(),
            m2: gmt.get_m2(),
            pointing_error: gmt.pointing_error,
            m1_truss_projection: gmt.m1_truss_projection,
        }
    }
}

#[cfg(test)]
pub mod tests {
    use std::f64::consts::FRAC_PI_2;

    use crate::{FromBuilder, Source};

    use super::*;
    use skyangle::Conversion;

    #[test]
    fn pointing_error() {
        let mut gmt = Gmt::builder()
            .pointing_error((1f64.from_arcsec(), FRAC_PI_2))
            .build()
            .unwrap();
        let mut src = Source::builder().build().unwrap();
        src.through(&mut gmt).xpupil();
        let tt: Vec<_> = src.gradients().into_iter().map(|x| x.to_mas()).collect();
        dbg!(tt);
    }
}
