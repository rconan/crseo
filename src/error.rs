use std::{ffi, fmt};

pub enum CrseoError {
    GmtModesPath(std::path::PathBuf),
    Env {
        var: String,
        error: std::env::VarError,
    },
    FFI(ffi::NulError),
}

impl fmt::Display for CrseoError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::GmtModesPath(p) => write!(f, "The path {:?} does not exist, set the environment variable GMT_MODES_PATH to the path to the directory that contains the files with the modes.",p),
	    Self::Env{var: v, error: e} => write!(f,"environment variable {} not set,\nCaused by: {}",v,e),
	    Self::FFI(e) => e.fmt(f)
        }
    }
}
impl fmt::Debug for CrseoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <CrseoError as std::fmt::Display>::fmt(self, f)
    }
}

impl std::error::Error for CrseoError {}

/*impl From<std::env::VarError> for CrseoError {
    fn from(e: std::env::VarError) -> Self {
        CrseoError::Env(e)
    }
}*/

impl From<ffi::NulError> for CrseoError {
    fn from(e: ffi::NulError) -> Self {
        CrseoError::FFI(e)
    }
}
