//!
//! # CEO gmt wrapper
//!
//! Provides a structure `Gmt` that is a wrapper for [CEO](https://github.com/rconan/CEO) gmt C++ structure.
//! `Gmt` is instantiated and initialized with the `GMT` builder
//!
//! # Examples
//!
//! ```
//! use ceo::ceo;
//! // Creates a gmt instance with default parameters
//! let mut src = ceo!(GMT);
//! ```
//!
//! ```
//! use ceo::ceo;
//! // Creates a gmt instance with 27 M1 bending modes
//! let mut gmt = ceo!(GMT, m1_n_mode = [27]);
//! ```

use std::f64::consts::PI;
use std::fmt::Display;

mod wrapper;
pub use wrapper::{Gmt, GmtBuilder};

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
}
pub type GmtResult<T> = std::result::Result<T, GmtError>;

/// Rigid body motions
#[derive(Clone, Debug)]
pub enum RBM {
    /// Translations
    Txyz(Vec<f64>),
    /// Rotations
    Rxyz(Vec<f64>),
}
impl Display for RBM {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use RBM::*;
        match self {
            Txyz(val) => write!(
                f,
                "{:+6.0?}",
                val.iter().map(|x| x * 1e9).collect::<Vec<f64>>()
            ),
            Rxyz(val) => write!(
                f,
                " {:+6.0?}",
                val.iter()
                    .map(|x| x * 180. * 3600e3 / PI)
                    .collect::<Vec<f64>>()
            ),
        }
    }
}
/// Mirror degrees-of-freedom
#[derive(Clone, Debug)]
pub enum MirrorDof {
    /// Rigid body motions
    RigidBodyMotions((Option<RBM>, Option<RBM>)),
    /// Mirror surface figures
    Modes(Vec<f64>),
}
impl Display for MirrorDof {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use MirrorDof::*;
        match self {
            RigidBodyMotions((Some(t_xyz), Some(r_xyz))) => {
                t_xyz.fmt(f)?;
                r_xyz.fmt(f)
            }
            Modes(modes) => write!(
                f,
                " [{}]",
                modes
                    .iter()
                    .take(5)
                    .map(|x| format!("{:>+10.3e}", x))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            _ => write!(f, ""),
        }
    }
}
impl From<MirrorDof> for Vec<f64> {
    fn from(mirror_dof: MirrorDof) -> Self {
        use MirrorDof::*;
        use RBM::*;
        match mirror_dof {
            RigidBodyMotions((Some(Txyz(mut tr_xyz)), Some(Rxyz(mut r_xyz)))) => {
                tr_xyz.append(&mut r_xyz);
                tr_xyz
            }
            RigidBodyMotions((None, Some(Rxyz(r_xyz)))) => r_xyz,
            RigidBodyMotions((Some(Txyz(t_xyz)), None)) => t_xyz,
            Modes(modes) => modes,
            _ => Vec::new(),
        }
    }
}
/// Segment pair degrees-of-freedom
#[derive(Clone, Debug)]
pub enum SegmentDof {
    M1((Option<MirrorDof>, Option<MirrorDof>)),
    M2((Option<MirrorDof>, Option<MirrorDof>)),
}
impl Display for SegmentDof {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use SegmentDof::*;
        match self {
            M1((Some(rbm), Some(mode))) => {
                rbm.fmt(f)?;
                mode.fmt(f)
            }
            M1((Some(rbm), None)) => rbm.fmt(f),
            M2((Some(rbm), Some(mode))) => {
                rbm.fmt(f)?;
                mode.fmt(f)
            }
            M2((Some(rbm), None)) => rbm.fmt(f),
            _ => write!(f, ""),
        }
    }
}
impl From<SegmentDof> for Vec<f64> {
    fn from(segment_dof: SegmentDof) -> Self {
        use SegmentDof::*;
        match segment_dof {
            M1((Some(rbm), Some(mode))) => {
                let mut a: Vec<f64> = rbm.into();
                let mut b: Vec<f64> = mode.into();
                a.append(&mut b);
                a
            }
            M1((Some(rbm), None)) => rbm.into(),
            M1((None, Some(mode))) => mode.into(),
            M2((Some(rbm), Some(mode))) => {
                let mut a: Vec<f64> = rbm.into();
                let mut b: Vec<f64> = mode.into();
                a.append(&mut b);
                a
            }
            M2((Some(rbm), None)) => rbm.into(),
            M2((None, Some(mode))) => mode.into(),
            _ => Vec::new(),
        }
    }
}
/**
Degrees-of-freedom of 7 pairs of M1/M2 segments

The degrees of freedom are ordered segment wise `[Si]` for i in `[1,7]`
where `Si = [M1,M2]` and `Mj = [Txyz, Rxyz, Modes]`
*/
#[derive(Default, Clone, Debug)]
pub struct SegmentsDof {
    dof: Option<Vec<(Option<SegmentDof>, Option<SegmentDof>)>>,
    m1_n_mode: usize,
    m2_n_mode: usize,
    s7_rz: bool,
    n_segment: usize,
}
impl Display for SegmentsDof {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(dofs) = &self.dof {
            writeln!(f, "GMT state (Txyz[nm], Rxyz[mas], modes):")?;
            Ok(for (k, dof) in dofs.iter().enumerate() {
                if let (Some(m1), Some(m2)) = dof {
                    writeln!(f, " - S{} M1: {}", k + 1, m1)?;
                    writeln!(f, "      M2: {}", m2)?;
                }
            })
        } else {
            writeln!(f, "")
        }
    }
}
impl IntoIterator for SegmentsDof {
    type Item = (Option<SegmentDof>, Option<SegmentDof>);
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        if let Some(segments) = self.dof {
            segments.into_iter()
        } else {
            Vec::new().into_iter()
        }
    }
}
impl From<SegmentsDof> for Vec<f64> {
    fn from(segments: SegmentsDof) -> Self {
        let s7_rz = segments.s7_rz;
        let idx1 = (segments.m1_n_mode + segments.m2_n_mode + 13) * 6;
        let idx2 = (segments.m1_n_mode + segments.m2_n_mode + 12) * 7 - segments.m2_n_mode - 1;
        let mut v: Self = segments
            .into_iter()
            .flat_map(|segment| match segment {
                (Some(m1), Some(m2)) => {
                    let mut a: Vec<f64> = m1.into();
                    let mut b: Vec<f64> = m2.into();
                    a.append(&mut b);
                    a
                }
                (Some(m1), None) => m1.into(),
                (None, Some(m2)) => m2.into(),
                _ => Vec::new(),
            })
            .collect();
        if !s7_rz {
            v.remove(idx2);
            v.remove(idx1);
        }
        v
    }
}
impl SegmentsDof {
    /// Creates a new GMT state enum
    ///
    /// All the rigid body motions are set to 0.
    /// The clocking of M1 and M2 center segments are exluded from
    /// the rigid body motions count leading to a total of 82 RBMs.
    /// There are no modes on the segments
    pub fn new() -> Self {
        use MirrorDof::*;
        use SegmentDof::*;
        use RBM::*;
        let u = vec![0f64; 3];
        Self {
            dof: Some(
                (0..7)
                    .map(|_| {
                        (
                            Some(M1((
                                Some(RigidBodyMotions((
                                    Some(Txyz(u.clone())),
                                    Some(Rxyz(u.clone())),
                                ))),
                                None,
                            ))),
                            Some(M2((
                                Some(RigidBodyMotions((
                                    Some(Txyz(u.clone())),
                                    Some(Rxyz(u.clone())),
                                ))),
                                None,
                            ))),
                        )
                    })
                    .collect(),
            ),
            n_segment: 7,
            ..Default::default()
        }
    }
    /// Adds `n` modes to M1 segments (set to 0)
    pub fn m1_n_mode(self, n: usize) -> Self {
        let mut dofs = vec![];
        for dof in self.dof.unwrap().into_iter() {
            if let (Some(m1), Some(m2)) = dof {
                if let SegmentDof::M1((Some(rbm), _)) = m1 {
                    dofs.push((
                        Some(SegmentDof::M1((
                            Some(rbm),
                            Some(MirrorDof::Modes(vec![0f64; n])),
                        ))),
                        Some(m2),
                    ));
                }
            }
        }
        Self {
            dof: Some(dofs),
            m1_n_mode: n,
            ..self
        }
    }
    /// Adds `n` modes to M2 segments (set to 0)
    pub fn m2_n_mode(self, n: usize) -> Self {
        let mut dofs = vec![];
        for dof in self.dof.unwrap().into_iter() {
            if let (Some(m1), Some(m2)) = dof {
                if let SegmentDof::M2((Some(rbm), _)) = m2 {
                    dofs.push((
                        Some(m1),
                        Some(SegmentDof::M2((
                            Some(rbm),
                            Some(MirrorDof::Modes(vec![0f64; n])),
                        ))),
                    ));
                }
            }
        }
        Self {
            dof: Some(dofs),
            m2_n_mode: n,
            ..self
        }
    }
    /// Adds M1 and M2 center segment clocking to segment RBMs
    pub fn include_s7_rz(self) -> Self {
        Self {
            s7_rz: true,
            ..self
        }
    }
    pub fn from_vec(self, vals: Vec<f64>) -> Self {
        let expected_len =
            self.n_segment * (self.m1_n_mode + self.m2_n_mode) + if self.s7_rz { 84 } else { 82 };
        assert_eq!(
            vals.len(),
            expected_len,
            "Expected {} elements found {}",
            expected_len,
            vals.len()
        );
        let mut v = vals;
        if !self.s7_rz {
            let idx = (self.m1_n_mode + self.m2_n_mode + 13) * 6;
            v.insert(idx, 0f64);
            let idx = (self.m1_n_mode + self.m2_n_mode + 12) * 7 - self.m2_n_mode - 1;
            v.insert(idx, 0f64);
        }
        use MirrorDof::*;
        use SegmentDof::*;
        use RBM::*;
        let n = self.m1_n_mode + self.m2_n_mode + 12;
        Self {
            dof: Some(
                v.chunks(n)
                    .map(|s| {
                        let mut so = s.to_vec();
                        (
                            Some(M1((
                                Some(RigidBodyMotions((
                                    Some(Txyz(so.drain(..3).collect())),
                                    Some(Rxyz(so.drain(..3).collect())),
                                ))),
                                (self.m1_n_mode > 0)
                                    .then(|| Modes(so.drain(..self.m1_n_mode).collect())),
                            ))),
                            Some(M2((
                                Some(RigidBodyMotions((
                                    Some(Txyz(so.drain(..3).collect())),
                                    Some(Rxyz(so.drain(..3).collect())),
                                ))),
                                (self.m2_n_mode > 0)
                                    .then(|| Modes(so.drain(..self.m2_n_mode).collect())),
                            ))),
                        )
                    })
                    .collect(),
            ),
            ..self
        }
    }
    pub fn from_rigid_body_motions(self, vals: Vec<f64>) -> Self {
        let expected_len = if self.s7_rz { 84 } else { 82 };
        assert_eq!(
            vals.len(),
            expected_len,
            "Expected {} elements found {}",
            expected_len,
            vals.len()
        );
        let mut v = vals;
        if !self.s7_rz {
            let idx = (self.m1_n_mode + self.m2_n_mode + 13) * 6;
            v.insert(idx, 0f64);
            let idx = (self.m1_n_mode + self.m2_n_mode + 12) * 7 - self.m2_n_mode - 1;
            v.insert(idx, 0f64);
        }
        use MirrorDof::*;
        use SegmentDof::*;
        use RBM::*;
        let n = self.m1_n_mode + self.m2_n_mode + 12;
        Self {
            dof: Some(
                v.chunks(n)
                    .map(|s| {
                        let mut so = s.to_vec();
                        (
                            Some(M1((
                                Some(RigidBodyMotions((
                                    Some(Txyz(so.drain(..3).collect())),
                                    Some(Rxyz(so.drain(..3).collect())),
                                ))),
                                (self.m1_n_mode > 0).then(|| Modes(vec![0f64; self.m1_n_mode])),
                            ))),
                            Some(M2((
                                Some(RigidBodyMotions((
                                    Some(Txyz(so.drain(..3).collect())),
                                    Some(Rxyz(so.drain(..3).collect())),
                                ))),
                                (self.m2_n_mode > 0).then(|| Modes(vec![0f64; self.m2_n_mode])),
                            ))),
                        )
                    })
                    .collect(),
            ),
            ..self
        }
    }
    pub fn segment(&mut self, sid: usize, segment_dof: SegmentDof) -> GmtResult<&mut Self> {
        use MirrorDof::*;
        use SegmentDof::*;
        use RBM::*;
        match segment_dof {
            M1((Some(RigidBodyMotions((Some(Txyz(_)), Some(Rxyz(_))))), _)) => {
                if let Some(dof) = self.dof.as_mut() {
                    dof[sid - 1] = (Some(segment_dof), dof[sid - 1].1.clone());
                }
                Ok(self)
            }
            M1((Some(RigidBodyMotions((Some(Txyz(ref val)), None))), ref x)) => {
                if let Some(dof) = self.dof.as_mut() {
                    let sdof = M1((
                        Some(RigidBodyMotions((
                            Some(Txyz(val.to_owned())),
                            Some(Rxyz(vec![0f64; 3])),
                        ))),
                        x.to_owned(),
                    ));
                    dof[sid - 1] = (Some(sdof), dof[sid - 1].1.clone());
                }
                Ok(self)
            }
            M1((Some(RigidBodyMotions((None, Some(Rxyz(ref val))))), ref x)) => {
                if let Some(dof) = self.dof.as_mut() {
                    let sdof = M1((
                        Some(RigidBodyMotions((
                            Some(Txyz(vec![0f64; 3])),
                            Some(Rxyz(val.to_owned())),
                        ))),
                        x.to_owned(),
                    ));
                    dof[sid - 1] = (Some(sdof), dof[sid - 1].1.clone());
                }
                Ok(self)
            }
            M2((Some(RigidBodyMotions((Some(Txyz(_)), Some(Rxyz(_))))), _)) => {
                if let Some(dof) = self.dof.as_mut() {
                    dof[sid - 1] = (dof[sid - 1].0.clone(), Some(segment_dof));
                }
                Ok(self)
            }
            M2((Some(RigidBodyMotions((Some(Txyz(ref val)), None))), ref x)) => {
                if let Some(dof) = self.dof.as_mut() {
                    let sdof = M2((
                        Some(RigidBodyMotions((
                            Some(Txyz(val.to_owned())),
                            Some(Rxyz(vec![0f64; 3])),
                        ))),
                        x.to_owned(),
                    ));
                    dof[sid - 1] = (dof[sid - 1].0.clone(), Some(sdof));
                }
                Ok(self)
            }
            M2((Some(RigidBodyMotions((None, Some(Rxyz(ref val))))), ref x)) => {
                if let Some(dof) = self.dof.as_mut() {
                    let sdof = M2((
                        Some(RigidBodyMotions((
                            Some(Txyz(vec![0f64; 3])),
                            Some(Rxyz(val.to_owned())),
                        ))),
                        x.to_owned(),
                    ));
                    dof[sid - 1] = (dof[sid - 1].0.clone(), Some(sdof));
                }
                Ok(self)
            }
            _ => Err(GmtError::SegmentDof),
        }
    }
    pub fn apply_to(&self, gmt: &mut Gmt) -> GmtResult<()> {
        if let Some(segments) = &self.dof {
            use MirrorDof::*;
            use SegmentDof::*;
            use RBM::*;
            let mut a1 = vec![];
            let mut a2 = vec![];
            for (k, segment) in segments.iter().enumerate() {
                match segment {
                    (
                        Some(M1((
                            Some(RigidBodyMotions((Some(Txyz(m1_t_xyz)), Some(Rxyz(m1_r_xyz))))),
                            Some(Modes(m1_modes)),
                        ))),
                        Some(M2((
                            Some(RigidBodyMotions((Some(Txyz(m2_t_xyz)), Some(Rxyz(m2_r_xyz))))),
                            Some(Modes(m2_modes)),
                        ))),
                    ) => {
                        let sid = k as i32 + 1;
                        gmt.m1_segment_state(sid, m1_t_xyz, m1_r_xyz);
                        gmt.m2_segment_state(sid, m2_t_xyz, m2_r_xyz);
                        a1.extend_from_slice(m1_modes);
                        a2.extend_from_slice(m2_modes);
                        Ok(())
                    }
                    (
                        Some(M1((
                            Some(RigidBodyMotions((Some(Txyz(m1_t_xyz)), Some(Rxyz(m1_r_xyz))))),
                            None,
                        ))),
                        None,
                    ) => {
                        let sid = k as i32 + 1;
                        gmt.m1_segment_state(sid, m1_t_xyz, m1_r_xyz);
                        Ok(())
                    }
                    (
                        None,
                        Some(M2((
                            Some(RigidBodyMotions((Some(Txyz(m2_t_xyz)), Some(Rxyz(m2_r_xyz))))),
                            None,
                        ))),
                    ) => {
                        let sid = k as i32 + 1;
                        gmt.m2_segment_state(sid, m2_t_xyz, m2_r_xyz);
                        Ok(())
                    }
                    (
                        Some(M1((
                            Some(RigidBodyMotions((Some(Txyz(m1_t_xyz)), Some(Rxyz(m1_r_xyz))))),
                            None,
                        ))),
                        Some(M2((
                            Some(RigidBodyMotions((Some(Txyz(m2_t_xyz)), Some(Rxyz(m2_r_xyz))))),
                            None,
                        ))),
                    ) => {
                        let sid = k as i32 + 1;
                        gmt.m1_segment_state(sid, m1_t_xyz, m1_r_xyz);
                        gmt.m2_segment_state(sid, m2_t_xyz, m2_r_xyz);
                        Ok(())
                    }
                    (
                        Some(M1((
                            Some(RigidBodyMotions((Some(Txyz(m1_t_xyz)), Some(Rxyz(m1_r_xyz))))),
                            None,
                        ))),
                        Some(M2((
                            Some(RigidBodyMotions((Some(Txyz(m2_t_xyz)), Some(Rxyz(m2_r_xyz))))),
                            Some(Modes(m2_modes)),
                        ))),
                    ) => {
                        let sid = k as i32 + 1;
                        gmt.m1_segment_state(sid, m1_t_xyz, m1_r_xyz);
                        gmt.m2_segment_state(sid, m2_t_xyz, m2_r_xyz);
                        a2.extend_from_slice(m2_modes);
                        Ok(())
                    }
                    (
                        Some(M1((
                            Some(RigidBodyMotions((Some(Txyz(m1_t_xyz)), Some(Rxyz(m1_r_xyz))))),
                            Some(Modes(m1_modes)),
                        ))),
                        Some(M2((
                            Some(RigidBodyMotions((Some(Txyz(m2_t_xyz)), Some(Rxyz(m2_r_xyz))))),
                            None,
                        ))),
                    ) => {
                        let sid = k as i32 + 1;
                        gmt.m1_segment_state(sid, m1_t_xyz, m1_r_xyz);
                        gmt.m2_segment_state(sid, m2_t_xyz, m2_r_xyz);
                        a1.extend_from_slice(m1_modes);
                        Ok(())
                    }
                    _ => Err(GmtError::GmtDofMatch),
                }?;
            }
            if !a1.is_empty() {
                gmt.m1_modes(&mut a1);
            }
            if !a2.is_empty() {
                gmt.m2_modes(&mut a2);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Builder, FromBuilder, SourceBuilder};

    #[test]
    fn gmt_new() {
        Gmt::builder().m1_n_mode(27).m2_n_mode(123).build().unwrap();
    }

    #[test]
    fn gmt_new_with_macro() {
        crate::ceo!(GmtBuilder, m1_n_mode = [27], m2_n_mode = [123]);
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
