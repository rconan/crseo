use std::{env, ffi::CString, marker::PhantomData, path::Path};

use serde::{Deserialize, Serialize};

use crate::{
    GmtError,
    gmt::{GmtM1, GmtM2, Mirror, ModeKind, ModeType, SurfaceMode},
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
}
impl MirrorBuilder {
    /// Sets the type of mirror modes
    pub fn mode_type(self, mode_type: impl Into<ModeType>) -> Self {
        Self {
            mode_type: mode_type.into(),
            ..self
        }
    }
    pub fn try_mode_type<T>(self, mode_type: T) -> Result<Self, <ModeType as TryFrom<T>>::Error>
    where
        ModeType: TryFrom<T>,
    {
        Ok(Self {
            mode_type: mode_type.try_into()?,
            ..self
        })
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
        let MirrorBuilder {
            mode_type,
            n_mode,
            a,
            ..
        } = builder;
        match mode_type {
            ModeType::CeoFile(_) => {
                let mut mirror = Self {
                    _c_: Default::default(),
                    mode_type,
                    n_mode,
                    a,
                    mode_kind: PhantomData,
                };
                let mode_type = CString::new(mode_path.map_err(|e| GmtError::from(e))?)?;
                unsafe {
                    let n_mode = mirror.n_mode;
                    mirror.setup1(mode_type.into_raw(), 7, n_mode as i32);
                }
                return Ok(mirror);
            }
            mut ds @ ModeType::DataSet {
                n_sample,
                width,
                n_set,
                n_mode,
                ..
            } => {
                let (s2b, data) = if let ModeType::DataSet { s2b, data, .. } = &mut ds {
                    (s2b.as_mut_ptr(), data.as_mut_ptr())
                } else {
                    unreachable!()
                };
                let mut mirror = Self {
                    _c_: Default::default(),
                    mode_type: ds,
                    n_mode: builder.n_mode,
                    a,
                    mode_kind: PhantomData,
                };
                unsafe {
                    mirror.setup4(
                        n_sample as i32,
                        width,
                        n_set as i32,
                        n_mode as i32,
                        s2b,
                        data,
                        7,
                        n_mode as i32,
                    );
                }
                return Ok(mirror);
            }

            ModeType::Zernike(_) => {
                unimplemented!("Zernike modes are available in the `zernike` git branch")
            }
        }
    }
}
impl TryFrom<MirrorBuilder> for Mirror<GmtM2> {
    type Error = GmtError;
    fn try_from(builder: MirrorBuilder) -> Result<Self, Self::Error> {
        let mode_path = builder.mode_path();
        let MirrorBuilder {
            mode_type,
            n_mode,
            a,
            ..
        } = builder;
        match mode_type {
            ModeType::CeoFile(_) => {
                let mut mirror = Self {
                    _c_: Default::default(),
                    mode_type,
                    n_mode,
                    a,
                    mode_kind: PhantomData,
                };
                let mode_type = CString::new(mode_path.map_err(|e| GmtError::from(e))?)?;
                unsafe {
                    let n_mode = mirror.n_mode;
                    mirror.setup1(mode_type.into_raw(), 7, n_mode as i32);
                }
                return Ok(mirror);
            }
            mut ds @ ModeType::DataSet {
                n_sample,
                width,
                n_set,
                n_mode,
                ..
            } => {
                let (s2b, data) = if let ModeType::DataSet { s2b, data, .. } = &mut ds {
                    (s2b.as_mut_ptr(), data.as_mut_ptr())
                } else {
                    unreachable!()
                };
                let mut mirror = Self {
                    _c_: Default::default(),
                    mode_type: ds,
                    n_mode: builder.n_mode,
                    a,
                    mode_kind: PhantomData,
                };
                unsafe {
                    mirror.setup4(
                        n_sample as i32,
                        width,
                        n_set as i32,
                        n_mode as i32,
                        s2b,
                        data,
                        7,
                        n_mode as i32,
                    );
                }
                return Ok(mirror);
            }

            ModeType::Zernike(_) => {
                unimplemented!("Zernike modes are available in the `zernike` git branch")
            }
        }
    }
}
