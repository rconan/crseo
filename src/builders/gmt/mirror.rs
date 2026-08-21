use std::{env, ffi::CString, marker::PhantomData, path::Path};

use serde::{Deserialize, Serialize};

use crate::{
    GmtError,
    gmt::{GmtM1, GmtM2, Mirror, ModeKind, ModeType, SurfaceMode, ZernikeMode},
};

#[derive(Default, Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct MirrorBuilder<K = SurfaceMode>
where
    K: ModeKind,
{
    pub mode_type: ModeType,
    pub n_mode: usize,
    pub a: Vec<f64>,
    pub(crate) mode_kind: PhantomData<K>,
    pub max_n: Option<usize>,
}
impl MirrorBuilder {
    /// Sets the type of mirror modes
    pub fn mode_type(self, mode_type: &str) -> Self {
        Self {
            mode_type: mode_type.into(),
            ..self
        }
    }
}
impl<K: ModeKind> MirrorBuilder<K> {
    pub fn new() -> Self {
        Default::default()
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
impl MirrorBuilder<ZernikeMode> {
    /// Sets the modes largest radial order and the number of modes
    pub fn radial_order(self, value: usize) -> Self {
        let n_mode = (value + 1) * (value + 2) / 2;
        Self {
            mode_type: ModeType::Zernike(value),
            n_mode,
            a: vec![0f64; 7 * n_mode],
            mode_kind: PhantomData,
            max_n: Some(value),
        }
    }
}
use super::GmtModesError;
impl MirrorBuilder {
    fn mode_path(&self) -> std::result::Result<String, GmtModesError> {
        let mode_type = Path::new(&self.mode_type.to_string()).with_extension("ceo");
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
impl TryFrom<MirrorBuilder> for Mirror<GmtM1> {
    type Error = GmtError;
    fn try_from(builder: MirrorBuilder) -> Result<Self, Self::Error> {
        let mode_path = builder.mode_path();
        let mut mirror = Self {
            _c_: Default::default(),
            mode_type: builder.mode_type,
            n_mode: builder.n_mode,
            a: builder.a,
            mode_kind: PhantomData,
        };
        let mode_type = CString::new(mode_path.map_err(|e| GmtError::from(e))?)?;
        unsafe {
            let n_mode = mirror.n_mode;
            mirror._c_.setup1(mode_type.into_raw(), 7, n_mode as i32);
        }
        Ok(mirror)
    }
}
impl TryFrom<MirrorBuilder<ZernikeMode>> for Mirror<GmtM1, ZernikeMode> {
    type Error = GmtError;
    fn try_from(builder: MirrorBuilder<ZernikeMode>) -> Result<Self, Self::Error> {
        // let mode_path = builder.mode_path();
        let mut mirror = Self {
            _c_: Default::default(),
            mode_type: builder.mode_type,
            n_mode: builder.n_mode,
            a: builder.a,
            mode_kind: PhantomData,
        };
        let ro = builder.max_n.ok_or(GmtModesError::RadialOrder)? as i32;
        let a = mirror.a.as_mut_ptr();
        unsafe {
            mirror._c_.setup3(ro, a);
        }
        Ok(mirror)
    }
}
impl TryFrom<MirrorBuilder> for Mirror<GmtM2> {
    type Error = GmtError;
    fn try_from(builder: MirrorBuilder) -> Result<Self, Self::Error> {
        let mode_path = builder.mode_path().map_err(|e| GmtError::from(e))?;
        let mut mirror = Self {
            _c_: Default::default(),
            mode_type: builder.mode_type,
            n_mode: builder.n_mode,
            a: builder.a,
            mode_kind: PhantomData,
        };
        let mode_type = CString::new(mode_path)?;
        unsafe {
            let n_mode = mirror.n_mode;
            mirror._c_.setup1(mode_type.into_raw(), 7, n_mode as i32);
        }
        Ok(mirror)
    }
}
impl TryFrom<MirrorBuilder<ZernikeMode>> for Mirror<GmtM2, ZernikeMode> {
    type Error = GmtError;
    fn try_from(builder: MirrorBuilder<ZernikeMode>) -> Result<Self, Self::Error> {
        // let mode_path = builder.mode_path();
        let mut mirror = Self {
            _c_: Default::default(),
            mode_type: builder.mode_type,
            n_mode: builder.n_mode,
            a: builder.a,
            mode_kind: PhantomData,
        };
        let ro = builder.max_n.ok_or(GmtModesError::RadialOrder)? as i32;
        dbg!(ro);
        dbg!(mirror.a.len());
        let a = mirror.a.as_mut_ptr();
        unsafe {
            mirror._c_.setup3(ro, a);
        }
        Ok(mirror)
    }
}
