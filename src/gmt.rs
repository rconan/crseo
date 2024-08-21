//!
//! # CEO gmt wrapper
//!
//! Provides a structure `Gmt` that is a wrapper for [CEO](https://github.com/rconan/CEO) gmt C++ structure.
//! `Gmt` is instantiated and initialized with the `GMT` builder
//!
//! # Examples
//!
//! ```
//! use crseo::{ceo, Gmt};
//! // Creates a gmt instance with default parameters
//! let mut src = ceo!(Gmt);
//! ```
//!
//! ```
//! use crseo::{ceo, Gmt};
//! // Creates a gmt instance with 27 M1 bending modes
//! let mut gmt = ceo!(Gmt, m1.n_mode = [27]);
//! ```

use crate::{FromBuilder, Propagation, Source};
use ffi::{gmt_m1, gmt_m2, vector};
use std::{
    ffi::CStr,
    ops::{Deref, DerefMut},
};

mod builder;
pub use builder::{GmtBuilder, GmtMirrorBuilder, GmtModesError, MirrorBuilder};

pub type GmtM1 = gmt_m1;
pub type GmtM2 = gmt_m2;

#[derive(Debug, thiserror::Error)]
pub enum GmtError {
    #[error(
        r#"GMT DOF pattern mismatch, expected:
 [None|Some(None|Some(M1((Some(RigidBodyMotions((Some(Txyz(_)),Some(Rxyz(_))))),None|Some(Modes(_)))),
            None|Some(M2((Some(RigidBodyMotions((Some(Txyz(_)),Some(Rxyz(_))))),None|Some(Modes(_)))))));7]
"#
    )]
    GmtDofMatch,
    #[error("invalid SegmentDof pattern")]
    SegmentDof,
    #[error("mirror modes file not found")]
    Modes(#[from] GmtModesError),
}

pub trait GmtMx {
    fn modes_as_mut(&mut self) -> &mut ffi::modes;
    fn update(&mut self, origin_: vector, euler_angles_: vector, idx: ::std::os::raw::c_int);
}

impl GmtMx for gmt_m1 {
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
        self
    }
}

#[derive(Debug, Default)]
pub struct Mirror<M: GmtMx> {
    pub _c_: M,
    /// mirror mode shapes name
    pub mode_type: String,
    /// number of modes per segment
    pub n_mode: usize,
    // modes coefficients
    pub a: Vec<f64>,
}

impl<M: GmtMx + Default> From<MirrorBuilder> for Mirror<M> {
    fn from(builder: MirrorBuilder) -> Self {
        Self {
            _c_: Default::default(),
            mode_type: builder.mode_type,
            n_mode: builder.n_mode,
            a: builder.a,
        }
    }
}

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

pub trait GmtMirror<M: GmtMx> {
    fn as_mut(&mut self) -> &mut Mirror<M>;
}

impl GmtMirror<gmt_m1> for Gmt {
    fn as_mut(&mut self) -> &mut Mirror<gmt_m1> {
        &mut self.m1
    }
}

impl GmtMirror<gmt_m2> for Gmt {
    fn as_mut(&mut self) -> &mut Mirror<gmt_m2> {
        &mut self.m2
    }
}

/// GMT wrapper
pub struct Gmt {
    pub m1: Mirror<gmt_m1>,
    pub m2: Mirror<gmt_m2>,
    /*     /// M1 number of bending modes per segment
       pub m1.n_mode: usize,
       /// M2 number of bending modes per segment
       pub m2.n_mode: usize,
       /// M2 largest Zernike radial order per segment
       pub m2_max_n: usize,
    // default M1 coefs values: Vec of 0f64
    pub a1: Vec<f64>,
    // default M2 coefs values: Vec of 0f64
    pub a2: Vec<f64>,
    */
    // pointing error
    pub pointing_error: Option<(f64, f64)>,
    m1_truss_projection: bool,
}
impl FromBuilder for Gmt {
    type ComponentBuilder = GmtBuilder;
}
impl Gmt {
    /// Returns `Gmt` M1 mode type
    pub fn get_m1_mode_type(&self) -> String {
        unsafe {
            String::from(
                CStr::from_ptr(self.m1.BS.filename.as_ptr())
                    .to_str()
                    .expect("CStr::to_str failed"),
            )
        }
    }
    /// Returns `Gmt` M1 properties
    pub fn get_m1(&self) -> MirrorBuilder {
        MirrorBuilder {
            mode_type: self.get_m1_mode_type(),
            n_mode: self.m1.n_mode,
            a: self.m1.a.clone(),
        }
    }
    /// Returns `Gmt` M2 properties
    pub fn get_m2(&self) -> MirrorBuilder {
        MirrorBuilder {
            mode_type: self.get_m2_mode_type(),
            n_mode: self.m2.n_mode,
            a: self.m2.a.clone(),
        }
    }
    /// Returns `Gmt` M2 mode type
    pub fn get_m2_mode_type(&self) -> String {
        unsafe {
            String::from(
                CStr::from_ptr(self.m2.BS.filename.as_ptr())
                    .to_str()
                    .expect("CStr::to_str failed"),
            )
        }
    }
    /// Resets M1 and M2 to their aligned states
    pub fn reset(&mut self) -> &mut Self {
        unsafe {
            self.m1.reset();
            self.m2.reset();
            let a = self.m1.a.as_mut_ptr();
            self.m1.BS.update(a);
            let a = self.m2.a.as_mut_ptr();
            self.m2.BS.update(a);
        }
        self
    }
    /// Keeps only the M1 segment specified in the vector `sid`
    ///
    /// * `sid` - vector of segment ID numbers in the range \[1,7\]
    pub fn keep(&mut self, sid: &[i32]) -> &mut Self {
        unsafe {
            self.m1.keep(sid.as_ptr() as *mut _, sid.len() as i32);
            self.m2.keep(sid.as_ptr() as *mut _, sid.len() as i32);
        }
        self
    }
    /// Sets M1 segment rigid body motion with:
    ///
    /// * `sid` - the segment ID number in the range \[1,7\]
    /// * `t_xyz` - the 3 translations Tx, Ty and Tz
    /// * `r_xyz` - the 3 rotations Rx, Ry and Rz
    pub fn m1_segment_state(&mut self, sid: i32, t_xyz: &[f64], r_xyz: &[f64]) {
        assert!(sid > 0 && sid < 8, "Segment ID must be in the range [1,7]!");
        let t_xyz = vector {
            x: t_xyz[0],
            y: t_xyz[1],
            z: t_xyz[2],
        };
        let r_xyz = vector {
            x: r_xyz[0],
            y: r_xyz[1],
            z: r_xyz[2],
        };
        unsafe {
            self.m1.update(t_xyz, r_xyz, sid);
        }
    }
    /// Sets M2 segment rigid body motion with:
    ///
    /// * `sid` - the segment ID number in the range \[1,7\]
    /// * `t_xyz` - the 3 translations Tx, Ty and Tz
    /// * `r_xyz` - the 3 rotations Rx, Ry and Rz
    pub fn m2_segment_state(&mut self, sid: i32, t_xyz: &[f64], r_xyz: &[f64]) {
        let t_xyz = vector {
            x: t_xyz[0],
            y: t_xyz[1],
            z: t_xyz[2],
        };
        let r_xyz = vector {
            x: r_xyz[0],
            y: r_xyz[1],
            z: r_xyz[2],
        };
        unsafe {
            self.m2.update(t_xyz, r_xyz, sid);
        }
    }
    /// Sets M1 modal coefficients
    ///
    /// The coefficients are given segment wise
    /// with the same number of modes per segment
    pub fn m1_modes(&mut self, a: &[f64]) {
        let a_n_mode = a.len() / 7;
        self.m1
            .a
            .chunks_mut(self.m1.n_mode)
            .zip(a.chunks(a_n_mode))
            .for_each(|(a1, a)| a1.iter_mut().zip(a).for_each(|(a1, a)| *a1 = *a));
        unsafe {
            let m1_a = self.m1.a.as_mut_ptr();
            self.m1.BS.update(m1_a);
        }
    }
    pub fn m1_segment_modes(&mut self, a: &[Vec<f64>]) {
        self.m1
            .a
            .chunks_mut(self.m1.n_mode)
            .zip(a)
            .for_each(|(a1, a)| a1.iter_mut().zip(a).for_each(|(a1, a)| *a1 = *a));
        unsafe {
            let m1_a = self.m1.a.as_mut_ptr();
            self.m1.BS.update(m1_a);
        }
    }
    pub fn m1_modes_ij(&mut self, i: usize, j: usize, value: f64) {
        let mut a = vec![0f64; 7 * self.m1.n_mode];
        a[i * self.m1.n_mode + j] = value;
        unsafe {
            self.m1.BS.update(a.as_mut_ptr());
        }
    }
    /// Sets M2 modal coefficients
    ///
    /// The coefficients are given segment wise
    /// with the same number of modes per segment
    pub fn m2_modes(&mut self, a: &[f64]) {
        let a_n_mode = a.len() / 7;
        self.m2
            .a
            .chunks_mut(self.m2.n_mode)
            .zip(a.chunks(a_n_mode))
            .for_each(|(a2, a)| a2.iter_mut().zip(a).for_each(|(a2, a)| *a2 = *a));
        unsafe {
            let m2_a = self.m2.a.as_mut_ptr();
            self.m2.BS.update(m2_a);
        }
    }
    /// Sets M2 modal coefficients for segment #`sid` (0 < `sid` < 8)
    pub fn m2_segment_modes(&mut self, sid: u8, a: &[f64]) {
        self.m2
            .a
            .chunks_mut(self.m2.n_mode)
            .skip(sid as usize - 1)
            .take(1)
            .for_each(|a2| a2.iter_mut().zip(a).for_each(|(a2, a)| *a2 = *a));
        unsafe {
            let m2_a = self.m2.a.as_mut_ptr();
            self.m2.BS.update(m2_a);
        }
    }
    /// Reset the segment modes to 0 and sets M2 modal coefficient #`j` for segment #`i`
    pub fn m2_modes_ij(&mut self, i: usize, j: usize, value: f64) {
        let mut a = vec![0f64; 7 * self.m2.n_mode];
        a[i * self.m2.n_mode + j] = value;
        unsafe {
            self.m2.BS.update(a.as_mut_ptr());
        }
    }
    /// Updates M1 and M1 rigid body motion and M1 model coefficients
    pub fn update(
        &mut self,
        m1_rbm: Option<&Vec<Vec<f64>>>,
        m2_rbm: Option<&Vec<Vec<f64>>>,
        m1_mode: Option<&Vec<Vec<f64>>>,
        m2_mode: Option<&Vec<Vec<f64>>>,
    ) {
        if let Some(m1_rbm) = m1_rbm {
            for (k, rbm) in m1_rbm.iter().enumerate() {
                self.m1_segment_state((k + 1) as i32, &rbm[..3], &rbm[3..]);
            }
        }
        if let Some(m2_rbm) = m2_rbm {
            for (k, rbm) in m2_rbm.iter().enumerate() {
                self.m2_segment_state((k + 1) as i32, &rbm[..3], &rbm[3..]);
            }
        }
        if let Some(m1_mode) = m1_mode {
            let mut m = m1_mode.clone().into_iter().flatten().collect::<Vec<f64>>();
            self.m1_modes(&mut m);
        }
        if let Some(m2_mode) = m2_mode {
            let mut m = m2_mode.clone().into_iter().flatten().collect::<Vec<f64>>();
            self.m2_modes(&mut m);
        }
    }
    pub fn update42(
        &mut self,
        m1_rbm: Option<&[f64]>,
        m2_rbm: Option<&[f64]>,
        m1_mode: Option<&[f64]>,
        m2_mode: Option<&[f64]>,
    ) {
        if let Some(m1_rbm) = m1_rbm {
            for (k, rbm) in m1_rbm.chunks(6).enumerate() {
                self.m1_segment_state((k + 1) as i32, &rbm[..3], &rbm[3..]);
            }
        }
        if let Some(m2_rbm) = m2_rbm {
            for (k, rbm) in m2_rbm.chunks(6).enumerate() {
                self.m2_segment_state((k + 1) as i32, &rbm[..3], &rbm[3..]);
            }
        }
        if let Some(m1_mode) = m1_mode {
            let mut m = m1_mode.to_vec();
            self.m1_modes(&mut m);
        }
        if let Some(m2_mode) = m2_mode {
            let mut m = m2_mode.to_vec();
            self.m2_modes(&mut m);
        }
    }
    /*
    pub fn update(&mut self, gstate: &GmtState) {
        let mut t_xyz = vec![0.0; 3];
        let mut r_xyz = vec![0.0; 3];
        let mut a: Vec<f64> = vec![0.0; 7 * self.m1.n_mode as usize];
        let mut id = 0;

        for sid in 1..8 {
            //print!("{}", sid);••••••••••••
            id = sid - 1;
            t_xyz[0] = gstate.rbm[[id, 0]] as f64;
            t_xyz[1] = gstate.rbm[[id, 1]] as f64;
            t_xyz[2] = gstate.rbm[[id, 2]] as f64;
            r_xyz[0] = gstate.rbm[[id, 3]] as f64;
            r_xyz[1] = gstate.rbm[[id, 4]] as f64;
            r_xyz[2] = gstate.rbm[[id, 5]] as f64;
            self.m1_segment_state(sid as i32, &t_xyz, &r_xyz);
            if self.m1.n_mode > 0 {
                for k_bm in 0..self.m1.n_mode {
                    let idx = id * self.m1.n_mode as usize + k_bm as usize;
                    a[idx as usize] = gstate.bm[[id, k_bm as usize]] as f64;
                }
            }
            id += 7;
            t_xyz[0] = gstate.rbm[[id, 0]] as f64;
            t_xyz[1] = gstate.rbm[[id, 1]] as f64;
            t_xyz[2] = gstate.rbm[[id, 2]] as f64;
            r_xyz[0] = gstate.rbm[[id, 3]] as f64;
            r_xyz[1] = gstate.rbm[[id, 4]] as f64;
            r_xyz[2] = gstate.rbm[[id, 5]] as f64;
            self.m2_segment_state(sid as i32, &t_xyz, &r_xyz);
        }
        self.m1_modes(&mut a);
    }
     */
    pub fn trace_all(&mut self, src: &mut Source) -> &mut Self {
        unsafe {
            src.as_raw_mut_ptr().reset_rays();
            let rays = &mut src.as_raw_mut_ptr().rays;
            self.m1.traceall(rays);
            self.m2.traceall(rays);
            rays.to_sphere1(-5.830, 2.197173);
        }
        self
    }
}
impl Drop for Gmt {
    /// Frees CEO memory before dropping `Gmt`
    fn drop(&mut self) {
        unsafe {
            self.m1.cleanup();
            self.m2.cleanup();
        }
    }
}
impl Propagation for Gmt {
    /// Ray traces a `Source` through `Gmt`, ray tracing stops at the exit pupil
    fn propagate(&mut self, src: &mut Source) {
        if let Some((pz, pa)) = self.pointing_error {
            let (s, c) = pa.sin_cos();
            let (px, py) = (pz * c, pz * s);
            let (zenith, azimuth): (Vec<_>, Vec<_>) = src
                .zenith
                .iter()
                .map(|z| *z as f64)
                .zip(src.azimuth.iter().map(|a| *a as f64))
                .map(|(z, a)| {
                    let (s, c) = a.sin_cos();
                    (z * c - px, z * s - py)
                })
                .map(|(x, y)| (x.hypot(y), y.atan2(x)))
                .unzip();
            src.update(zenith, azimuth);
            unsafe {
                src.as_raw_mut_ptr().reset_rays();
                let rays = &mut src.as_raw_mut_ptr().rays;
                self.m2.blocking(rays);
                self.m1.trace(rays);
                if self.m1_truss_projection {
                    rays.gmt_truss_onaxis();
                }
                rays.gmt_m2_baffle();
                self.m2.trace(rays);
                rays.to_sphere1(-5.830, 2.197173);
            }
            src.update(
                src.zenith.iter().map(|x| *x as f64).collect(),
                src.azimuth.iter().map(|x| *x as f64).collect(),
            );
        } else {
            unsafe {
                src.as_raw_mut_ptr().reset_rays();
                let rays = &mut src.as_raw_mut_ptr().rays;
                self.m2.blocking(rays);
                self.m1.trace(rays);
                if self.m1_truss_projection {
                    rays.gmt_truss_onaxis();
                }
                rays.gmt_m2_baffle();
                self.m2.trace(rays);
                rays.to_sphere1(-5.830, 2.197173);
            }
        }
    }
    fn time_propagate(&mut self, _secs: f64, src: &mut Source) {
        self.propagate(src)
    }
}

/* #[cfg(test)]
mod tests {
    use super::*;
    use crate::{Builder, FromBuilder};

    #[test]
    fn gmt_new() {
        Gmt::builder().m1.n_mode(27).m2.n_mode(123).build().unwrap();
    }

    #[test]
    fn gmt_new_with_macro() {
        crate::ceo!(GmtBuilder, m1.n_mode = [27], m2.n_mode = [123]);
    }

    #[test]
    fn gmt_optical_alignment() {
        use crate::Source;
        let mut src = Source::builder().build().unwrap();
        let mut gmt = Gmt::builder().build().unwrap();
        src.through(&mut gmt).xpupil();
        assert!(src.wfe_rms_10e(-9)[0] < 1.0);
    }

    #[test]
    fn gmt_m1_rx_optical_sensitivity() {
        use crate::Source;
        let mut src = Source::builder().pupil_sampling(1001).build().unwrap();
        let mut gmt = Gmt::builder().build().unwrap();
        let seg_tts0: Vec<_> = src
            .through(&mut gmt)
            .xpupil()
            .segment_gradients()
            .chunks(7)
            .map(|x| x.to_owned())
            .collect();
        let rt = vec![vec![0f64, 0f64, 0f64, 1e-6, 0f64, 0f64]; 7];
        gmt.update(Some(&rt), None, None, None);
        let seg_tts: Vec<_> = src
            .through(&mut gmt)
            .xpupil()
            .segment_gradients()
            .chunks(7)
            .map(|x| x.to_owned())
            .collect();
        let mut delta: Vec<f64> = Vec::with_capacity(7);
        for k in 0..7 {
            delta
                .push(1e6 * (seg_tts[0][k] - seg_tts0[0][k]).hypot(seg_tts[1][k] - seg_tts0[1][k]));
        }
        assert!(delta.iter().all(|x| (x - 2.0).abs() < 1e-1));
    }

    #[test]
    fn gmt_m1_ry_optical_sensitivity() {
        use crate::Source;
        let mut src = Source::builder().pupil_sampling(1001).build().unwrap();
        let mut gmt = Gmt::builder().build().unwrap();
        let seg_tts0: Vec<_> = src
            .through(&mut gmt)
            .xpupil()
            .segment_gradients()
            .chunks(7)
            .map(|x| x.to_owned())
            .collect();
        let rt = vec![vec![0f64, 0f64, 0f64, 0f64, 1e-6, 0f64]; 7];
        gmt.update(Some(&rt), None, None, None);
        let seg_tts: Vec<_> = src
            .through(&mut gmt)
            .xpupil()
            .segment_gradients()
            .chunks(7)
            .map(|x| x.to_owned())
            .collect();
        let mut delta: Vec<f64> = Vec::with_capacity(7);
        for k in 0..7 {
            delta
                .push(1e6 * (seg_tts[0][k] - seg_tts0[0][k]).hypot(seg_tts[1][k] - seg_tts0[1][k]));
        }
        assert!(delta.iter().all(|x| (x - 2.0).abs() < 1e-1));
    }

    #[test]
    fn gmt_m2_rx_optical_sensitivity() {
        use crate::Source;
        let mut src = Source::builder().pupil_sampling(1001).build().unwrap();
        let mut gmt = Gmt::builder().build().unwrap();
        let seg_tts0: Vec<_> = src
            .through(&mut gmt)
            .xpupil()
            .segment_gradients()
            .chunks(7)
            .map(|x| x.to_owned())
            .collect();
        let rt = vec![vec![0f64, 0f64, 0f64, 1e-6, 0f64, 0f64]; 7];
        gmt.update(None, Some(&rt), None, None);
        let seg_tts: Vec<_> = src
            .through(&mut gmt)
            .xpupil()
            .segment_gradients()
            .chunks(7)
            .map(|x| x.to_owned())
            .collect();
        let mut delta: Vec<f64> = Vec::with_capacity(7);
        for k in 0..7 {
            delta
                .push(1e6 * (seg_tts[0][k] - seg_tts0[0][k]).hypot(seg_tts[1][k] - seg_tts0[1][k]));
        }
        assert!(delta.iter().all(|x| (x - 0.25).abs() < 1e-3));
    }

    #[test]
    fn gmt_m2_ry_optical_sensitivity() {
        use crate::Source;
        let mut src = Source::builder().pupil_sampling(1001).build().unwrap();
        let mut gmt = Gmt::builder().build().unwrap();
        let seg_tts0: Vec<_> = src
            .through(&mut gmt)
            .xpupil()
            .segment_gradients()
            .chunks(7)
            .map(|x| x.to_owned())
            .collect();
        let rt = vec![vec![0f64, 0f64, 0f64, 0f64, 1e-6, 0f64]; 7];
        gmt.update(None, Some(&rt), None, None);
        let seg_tts: Vec<_> = src
            .through(&mut gmt)
            .xpupil()
            .segment_gradients()
            .chunks(7)
            .map(|x| x.to_owned())
            .collect();
        let mut delta: Vec<f64> = Vec::with_capacity(7);
        for k in 0..7 {
            delta
                .push(1e6 * (seg_tts[0][k] - seg_tts0[0][k]).hypot(seg_tts[1][k] - seg_tts0[1][k]));
        }
        assert!(delta.iter().all(|x| (x - 0.25).abs() < 1e-2));
    }

    /*
    #[test]
    fn gmt_lenslet_gradients() {
        let pupil_size = 25.5f64;
        let n_lenslet = 48i32;
        let lenslet_size = pupil_size / n_lenslet as f64;
        let mut gmt = Gmt::new();
        gmt.build(1, None);
        let mut src = Source::new(1, pupil_size, n_lenslet * 16 + 1);
        src.build("V", vec![0.0], vec![0.0], vec![0.0]);
        src.fwhm(4.0);
        let mut sensor = Imaging::new();
        sensor.build(1, n_lenslet as i32, 16, 2, 24, 3);
        let mut cog0 = Centroiding::new();
        cog0.build(n_lenslet as u32, None);

        src.through(&mut gmt).xpupil().through(&mut sensor);
        cog0.process(&sensor, None)
            .valid_lenslets(Some(0.9), None);
        src.lenslet_gradients(n_lenslet, lenslet_size, &mut cog0);
        let s0 = cog0.grab().valids(None);
        if s0.iter().any(|x| x.is_nan()) {
            let n = (n_lenslet * n_lenslet) as usize;
            for k in 0..n {
                if k % n_lenslet as usize == 0 {
                    println!("");
                }
                if cog0.centroids[k].is_nan() || cog0.centroids[k + n].is_nan() {
                    print!("X");
                } else {
                    print!("o");
                }
            }
        }
        let s0_any_nan = s0.iter().any(|x| x.is_nan());
        assert!(!s0_any_nan);

        let rt = vec![vec![0f64, 0f64, 0f64, 1e-6, 1e-6, 0f64]; 7];
        gmt.update(None, Some(&rt), None);

        let mut cog = Centroiding::new();
        cog.build(n_lenslet as u32, None);
        cog.valid_lenslets(None, Some(cog0.valid_lenslets.clone()));
        src.through(&mut gmt)
            .xpupil()
            .lenslet_gradients(n_lenslet, lenslet_size, &mut cog);

        let s = cog.grab().valids(None);
        let s_any_nan = s.iter().any(|x| x.is_nan());
        assert!(!s_any_nan);
    }
    */
}
 */
