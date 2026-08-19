use std::{
    ffi::CStr,
    fmt::Display,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use ffi::{gmt_m1, gmt_m2, vector};
use serde::{Deserialize, Serialize};

pub type GmtM1 = gmt_m1;
pub type GmtM2 = gmt_m2;

pub trait ModeKind {}
#[derive(Default, Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ZernikeMode;
impl ModeKind for ZernikeMode {}
#[derive(Default, Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct SurfaceMode;
impl ModeKind for SurfaceMode {}

pub trait GmtMx<K = SurfaceMode>
where
    K: ModeKind,
{
    fn mode_type(&self) -> ModeType;
    fn modes_as_mut(&mut self) -> &mut ffi::modes;
    fn update(&mut self, origin_: vector, euler_angles_: vector, idx: ::std::os::raw::c_int);
}

impl GmtMx for gmt_m1 {
    fn mode_type(&self) -> ModeType {
        unsafe {
            String::from(
                CStr::from_ptr(self.BS.filename.as_ptr())
                    .to_str()
                    .expect("CStr::to_str failed"),
            )
        }
        .into()
    }
    #[inline]
    fn modes_as_mut(&mut self) -> &mut ffi::modes {
        &mut self.BS
    }
    #[inline]
    fn update(&mut self, origin_: vector, euler_angles_: vector, idx: ::std::os::raw::c_int) {
        unsafe { self.update(origin_, euler_angles_, idx) }
    }
}
impl GmtMx for gmt_m2 {
    fn mode_type(&self) -> ModeType {
        unsafe {
            String::from(
                CStr::from_ptr(self.BS.filename.as_ptr())
                    .to_str()
                    .expect("CStr::to_str failed"),
            )
        }
        .into()
    }
    #[inline]
    fn modes_as_mut(&mut self) -> &mut ffi::modes {
        &mut self.BS
    }
    #[inline]
    fn update(&mut self, origin_: vector, euler_angles_: vector, idx: ::std::os::raw::c_int) {
        unsafe { self.update(origin_, euler_angles_, idx) }
    }
}

pub trait MirrorGetSet {
    fn get_mode_type(&self) -> ModeType;
    /// Sets M2 modal coefficients
    ///
    /// The coefficients are given segment wise
    /// with the same number of modes per segment
    fn set_modes(&mut self, a: &[f64]) -> &mut Self;
    /// Setsmodal coefficients for segment #`sid` (0 < `sid` < 8)
    fn set_segment_modes(&mut self, sid: u8, a: &[f64]) -> &mut Self;
    /// Sets M1 segment rigid body motion with:
    ///
    /// * `sid` - the segment ID number in the range \[1,7\]
    /// * `t_xyz` - the 3 translations Tx, Ty and Tz
    /// * `r_xyz` - the 3 rotations Rx, Ry and Rz
    fn set_rigid_body_motions(&mut self, sid: u8, tr_xyz: &[f64]) -> &mut Self;
}

impl<M: GmtMx> MirrorGetSet for Mirror<M> {
    fn get_mode_type(&self) -> ModeType {
        self._c_.mode_type()
    }
    fn set_segment_modes(&mut self, sid: u8, a: &[f64]) -> &mut Self {
        self.a
            .chunks_mut(self.n_mode)
            .skip(sid as usize - 1)
            .take(1)
            .for_each(|a_sid: &mut [f64]| {
                a_sid.iter_mut().zip(a).for_each(|(a_sid, a)| *a_sid = *a)
            });
        unsafe {
            let m_sid_a = self.a.as_mut_ptr();
            self._c_.modes_as_mut().update(m_sid_a);
        }
        self
    }

    fn set_modes(&mut self, a: &[f64]) -> &mut Self {
        let a_n_mode = a.len() / 7;
        self.a
            .chunks_mut(self.n_mode)
            .zip(a.chunks(a_n_mode))
            .for_each(|(a_sid, a)| a_sid.iter_mut().zip(a).for_each(|(a_sid, a)| *a_sid = *a));
        unsafe {
            let m_sid_a = self.a.as_mut_ptr();
            self.modes_as_mut().update(m_sid_a);
        }
        self
    }

    fn set_rigid_body_motions(&mut self, sid: u8, tr_xyz: &[f64]) -> &mut Self {
        assert!(sid > 0 && sid < 8, "Segment ID must be in the range [1,7]!");
        let t_xyz = vector {
            x: tr_xyz[0],
            y: tr_xyz[1],
            z: tr_xyz[2],
        };
        let r_xyz = vector {
            x: tr_xyz[3],
            y: tr_xyz[4],
            z: tr_xyz[5],
        };
        self.update(t_xyz, r_xyz, sid as i32);
        //     unsafe { self.update(origin_, euler_angles_, idx) }
        self
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum ModeType {
    CeoFile(String),
    Zernike(usize),
}
impl Default for ModeType {
    fn default() -> Self {
        Self::CeoFile(String::new())
    }
}
impl Display for ModeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModeType::CeoFile(name) => write!(f, "{name}"),
            ModeType::Zernike(_) => write!(f, "zernike"),
        }
    }
}
impl From<&str> for ModeType {
    fn from(value: &str) -> Self {
        Self::CeoFile(value.into())
    }
}
impl From<String> for ModeType {
    fn from(value: String) -> Self {
        Self::CeoFile(value)
    }
}

#[derive(Debug, Default)]
pub struct Mirror<M: GmtMx<K>, K = SurfaceMode>
where
    K: ModeKind,
{
    pub _c_: M,
    /// mirror mode shapes name
    pub mode_type: ModeType,
    /// number of modes per segment
    pub n_mode: usize,
    // modes coefficients
    pub a: Vec<f64>,
    pub(crate) mode_kind: PhantomData<K>,
}

impl<M: GmtMx + Display> Display for Mirror<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self._c_, self.n_mode)
    }
}

// impl<M: GmtMx + Default> From<MirrorBuilder> for Mirror<M> {
//     fn from(builder: MirrorBuilder) -> Self {
//         Self {
//             _c_: Default::default(),
//             mode_type: builder.mode_type,
//             n_mode: builder.n_mode,
//             a: builder.a,
//             mode_kind: PhantomData,
//         }
//     }
// }
// impl Mirror<GmtM1> {
//     fn global_tiptilt(&mut self, tip: f64, tilt: f64) {
//         unsafe { self._c_.global_tiptilt(tip as f32, tilt as f32) };
//     }
// }
// impl Mirror<GmtM2> {
//     fn global_tiptilt(&mut self, tip: f64, tilt: f64) {
//         unsafe { self._c_.global_tiptilt(tip as f32, tilt as f32) };
//     }
// }
impl<M: GmtMx> Deref for Mirror<M> {
    type Target = M;

    fn deref(&self) -> &Self::Target {
        &self._c_
    }
}

impl<M: GmtMx> DerefMut for Mirror<M> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self._c_
    }
}
