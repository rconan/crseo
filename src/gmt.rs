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

use super::ceo_bindings::{gmt_m1, gmt_m2, vector};
use super::{Builder, CrseoError, Propagation, Result, Source};
use std::f64::consts::PI;
use std::fmt::Display;
use std::{
    env,
    ffi::{CStr, CString},
    path::Path,
};

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
                    writeln!(f, " -    M2: {}", m2)?;
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
            v.remove(78);
            v.pop();
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
            v.insert(78, 0f64);
            v.push(0f64);
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

#[doc(hidden)]
#[derive(Default, Debug, Clone)]
pub struct Mirror {
    mode_type: String,
    n_mode: usize,
    a: Vec<f64>,
}
impl Mirror {
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
/// use ceo::{Builder, GMT};
/// let mut src = GMT::new().build();
/// ```
///
/// ```
/// use ceo::{Builder, GMT};
/// let mut gmt = GMT::new().m1_n_mode(27).build();
/// ```
#[derive(Debug, Clone)]
pub struct GMT {
    m1: Mirror,
    m2: Mirror,
}
impl Default for GMT {
    fn default() -> Self {
        GMT {
            m1: Mirror {
                mode_type: "bending modes".into(),
                ..Default::default()
            },
            m2: Mirror {
                mode_type: "Karhunen-Loeve".into(),
                ..Default::default()
            },
        }
    }
}
impl GMT {
    /// Set the type and number of modes of M1
    pub fn m1(self, mode_type: &str, n_mode: usize) -> Self {
        Self {
            m1: self.m1.mode_type(mode_type).n_mode(n_mode),
            ..self
        }
    }
    /// Set the number of modes of M1
    pub fn m1_n_mode(self, n_mode: usize) -> Self {
        Self {
            m1: self.m1.n_mode(n_mode),
            ..self
        }
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
}
impl Mirror {
    fn mode_path(&self) -> Result<String> {
        let mode_type = Path::new(&self.mode_type).with_extension("ceo");
        if mode_type.is_file() {
            Ok(mode_type.to_str().unwrap().to_owned())
        } else {
            let env_path =
                env::var("GMT_MODES_PATH").unwrap_or_else(|_| String::from("CEO/gmtMirrors"));
            let path = Path::new(&env_path).join(mode_type);
            if path.is_file() {
                Ok(path.to_str().unwrap().to_owned())
            } else {
                Err(CrseoError::GmtModesPath(path))
            }
        }
    }
}
impl Builder for GMT {
    type Component = Gmt;
    fn build(self) -> std::result::Result<Gmt, CrseoError> {
        let mut gmt = Gmt {
            _c_m1: Default::default(),
            _c_m2: Default::default(),
            m1_n_mode: 0,
            m2_n_mode: 0,
            m2_max_n: 0,
            a1: self.m1.a.clone(),
            a2: self.m2.a.clone(),
        };
        let m1_mode_type = CString::new(self.m1.mode_path()?)?;
        gmt.m1_n_mode = self.m1.n_mode;
        unsafe {
            gmt._c_m1
                .setup1(m1_mode_type.into_raw(), 7, gmt.m1_n_mode as i32);
        }
        let m2_mode_type = CString::new(self.m2.mode_path()?)?;
        gmt.m2_n_mode = self.m2.n_mode;
        unsafe {
            gmt._c_m2
                .setup1(m2_mode_type.into_raw(), 7, gmt.m2_n_mode as i32);
        }
        gmt.reset();
        Ok(gmt)
    }
}
impl From<&Gmt> for GMT {
    fn from(gmt: &Gmt) -> Self {
        Self {
            m1: gmt.get_m1(),
            m2: gmt.get_m2(),
        }
    }
}
/// gmt wrapper
pub struct Gmt {
    _c_m1: gmt_m1,
    _c_m2: gmt_m2,
    /// M1 number of bending modes per segment
    pub m1_n_mode: usize,
    /// M2 number of bending modes per segment
    pub m2_n_mode: usize,
    /// M2 largest Zernike radial order per segment
    pub m2_max_n: usize,
    // default M1 coefs values: Vec of 0f64
    pub a1: Vec<f64>,
    // default M2 coefs values: Vec of 0f64
    pub a2: Vec<f64>,
}
impl Gmt {
    /// Returns `Gmt` M1 mode type
    pub fn get_m1_mode_type(&self) -> String {
        unsafe {
            String::from(
                CStr::from_ptr(self._c_m1.BS.filename.as_ptr())
                    .to_str()
                    .expect("CStr::to_str failed"),
            )
        }
    }
    /// Returns `Gmt` M1 properties
    pub fn get_m1(&self) -> Mirror {
        Mirror {
            mode_type: self.get_m1_mode_type(),
            n_mode: self.m1_n_mode,
            a: self.a1.clone(),
        }
    }
    /// Returns `Gmt` M2 properties
    pub fn get_m2(&self) -> Mirror {
        Mirror {
            mode_type: self.get_m2_mode_type(),
            n_mode: self.m2_n_mode,
            a: self.a2.clone(),
        }
    }
    /// Returns `Gmt` M2 mode type
    pub fn get_m2_mode_type(&self) -> String {
        unsafe {
            String::from(
                CStr::from_ptr(self._c_m2.BS.filename.as_ptr())
                    .to_str()
                    .expect("CStr::to_str failed"),
            )
        }
    }
    /// Resets M1 and M2 to their aligned states
    pub fn reset(&mut self) -> &mut Self {
        unsafe {
            self._c_m1.reset();
            self._c_m2.reset();
            self._c_m1.BS.update(self.a1.as_mut_ptr());
            self._c_m2.BS.update(self.a2.as_mut_ptr());
        }
        self
    }
    /// Keeps only the M1 segment specified in the vector `sid`
    ///
    /// * `sid` - vector of segment ID numbers in the range [1,7]
    pub fn keep(&mut self, sid: &mut [i32]) -> &mut Self {
        unsafe {
            self._c_m1.keep(sid.as_mut_ptr(), sid.len() as i32);
            self._c_m2.keep(sid.as_mut_ptr(), sid.len() as i32);
        }
        self
    }
    /// Sets M1 segment rigid body motion with:
    ///
    /// * `sid` - the segment ID number in the range [1,7]
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
            self._c_m1.update(t_xyz, r_xyz, sid);
        }
    }
    /// Sets M2 segment rigid body motion with:
    ///
    /// * `sid` - the segment ID number in the range [1,7]
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
            self._c_m2.update(t_xyz, r_xyz, sid);
        }
    }
    /// Sets M1 modal coefficients
    ///
    /// The coefficients are given segment wise
    pub fn m1_modes(&mut self, a: &mut [f64]) {
        let a_n_mode = a.len() / 7;
        if self.m1_n_mode > a_n_mode {
            let buf: Vec<_> = a
                .chunks(a_n_mode)
                .zip(self.a1.chunks(self.m1_n_mode))
                .flat_map(|(a, a1)| vec![a, &a1[a_n_mode..]])
                .collect();
            unsafe {
                self._c_m1.BS.update(buf.concat().as_mut_ptr());
            }
        } else {
            unsafe {
                self._c_m1.BS.update(a.as_mut_ptr());
            }
        }
    }
    pub fn m1_modes_ij(&mut self, i: usize, j: usize, value: f64) {
        let mut a = vec![0f64; 7 * self.m1_n_mode];
        a[i * self.m1_n_mode + j] = value;
        unsafe {
            self._c_m1.BS.update(a.as_mut_ptr());
        }
    }
    /// Sets M2 modal coefficients
    pub fn m2_modes(&mut self, a: &mut [f64]) {
        if self.m2_n_mode > a.len() {
            unsafe {
                self._c_m2.BS.update(
                    [a, &self.a2[self.m2_n_mode - a.len()..]]
                        .concat()
                        .as_mut_ptr(),
                );
            }
        } else {
            unsafe {
                self._c_m2.BS.update(a.as_mut_ptr());
            }
        }
    }
    pub fn m2_modes_ij(&mut self, i: usize, j: usize, value: f64) {
        let mut a = vec![0f64; 7 * self.m2_n_mode];
        a[i * self.m2_n_mode + j] = value;
        unsafe {
            self._c_m2.BS.update(a.as_mut_ptr());
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
        let mut a: Vec<f64> = vec![0.0; 7 * self.m1_n_mode as usize];
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
            if self.m1_n_mode > 0 {
                for k_bm in 0..self.m1_n_mode {
                    let idx = id * self.m1_n_mode as usize + k_bm as usize;
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
            self._c_m1.traceall(rays);
            self._c_m2.traceall(rays);
            rays.to_sphere1(-5.830, 2.197173);
        }
        self
    }
}
impl Drop for Gmt {
    /// Frees CEO memory before dropping `Gmt`
    fn drop(&mut self) {
        unsafe {
            self._c_m1.cleanup();
            self._c_m2.cleanup();
        }
    }
}
impl Propagation for Gmt {
    /// Ray traces a `Source` through `Gmt`, ray tracing stops at the exit pupil
    fn propagate(&mut self, src: &mut Source) -> &mut Self {
        unsafe {
            src.as_raw_mut_ptr().reset_rays();
            let rays = &mut src.as_raw_mut_ptr().rays;
            self._c_m2.blocking(rays);
            self._c_m1.trace(rays);
            rays.gmt_truss_onaxis();
            rays.gmt_m2_baffle();
            self._c_m2.trace(rays);
            rays.to_sphere1(-5.830, 2.197173);
        }
        self
    }
    fn time_propagate(&mut self, _secs: f64, src: &mut Source) -> &mut Self {
        self.propagate(src)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Builder;

    #[test]
    fn gmt_new() {
        GMT::new().m1_n_mode(27).m2_n_mode(123).build().unwrap();
    }

    #[test]
    fn gmt_new_with_macro() {
        crate::ceo!(GMT, m1_n_mode = [27], m2_n_mode = [123]);
    }

    #[test]
    fn gmt_optical_alignment() {
        use crate::SOURCE;
        let mut src = SOURCE::new().pupil_sampling(1001).build().unwrap();
        let mut gmt = GMT::new().build().unwrap();
        src.through(&mut gmt).xpupil();
        assert!(src.wfe_rms_10e(-9)[0] < 1.0);
    }

    #[test]
    fn gmt_m1_rx_optical_sensitity() {
        use crate::SOURCE;
        let mut src = SOURCE::new().pupil_sampling(1001).build().unwrap();
        let mut gmt = GMT::new().build().unwrap();
        let seg_tts0 = src.through(&mut gmt).xpupil().segment_gradients();
        let rt = vec![vec![0f64, 0f64, 0f64, 1e-6, 0f64, 0f64]; 7];
        gmt.update(Some(&rt), None, None, None);
        let seg_tts = src.through(&mut gmt).xpupil().segment_gradients();
        let mut delta: Vec<f32> = Vec::with_capacity(7);
        for k in 0..7 {
            delta
                .push(1e6 * (seg_tts[0][k] - seg_tts0[0][k]).hypot(seg_tts[1][k] - seg_tts0[1][k]));
        }
        assert!(delta.iter().all(|x| (x - 2.0).abs() < 1e-1));
    }

    #[test]
    fn gmt_m1_ry_optical_sensitity() {
        use crate::SOURCE;
        let mut src = SOURCE::new().pupil_sampling(1001).build().unwrap();
        let mut gmt = GMT::new().build().unwrap();
        let seg_tts0 = src.through(&mut gmt).xpupil().segment_gradients();
        let rt = vec![vec![0f64, 0f64, 0f64, 0f64, 1e-6, 0f64]; 7];
        gmt.update(Some(&rt), None, None, None);
        let seg_tts = src.through(&mut gmt).xpupil().segment_gradients();
        let mut delta: Vec<f32> = Vec::with_capacity(7);
        for k in 0..7 {
            delta
                .push(1e6 * (seg_tts[0][k] - seg_tts0[0][k]).hypot(seg_tts[1][k] - seg_tts0[1][k]));
        }
        assert!(delta.iter().all(|x| (x - 2.0).abs() < 1e-1));
    }

    #[test]
    fn gmt_m2_rx_optical_sensitity() {
        use crate::SOURCE;
        let mut src = SOURCE::new().pupil_sampling(1001).build().unwrap();
        let mut gmt = GMT::new().build().unwrap();
        let seg_tts0 = src.through(&mut gmt).xpupil().segment_gradients();
        let rt = vec![vec![0f64, 0f64, 0f64, 1e-6, 0f64, 0f64]; 7];
        gmt.update(None, Some(&rt), None, None);
        let seg_tts = src.through(&mut gmt).xpupil().segment_gradients();
        let mut delta: Vec<f32> = Vec::with_capacity(7);
        for k in 0..7 {
            delta
                .push(1e6 * (seg_tts[0][k] - seg_tts0[0][k]).hypot(seg_tts[1][k] - seg_tts0[1][k]));
        }
        assert!(delta.iter().all(|x| (x - 0.25).abs() < 1e-3));
    }

    #[test]
    fn gmt_m2_ry_optical_sensitity() {
        use crate::SOURCE;
        let mut src = SOURCE::new().pupil_sampling(1001).build().unwrap();
        let mut gmt = GMT::new().build().unwrap();
        let seg_tts0 = src.through(&mut gmt).xpupil().segment_gradients();
        let rt = vec![vec![0f64, 0f64, 0f64, 0f64, 1e-6, 0f64]; 7];
        gmt.update(None, Some(&rt), None, None);
        let seg_tts = src.through(&mut gmt).xpupil().segment_gradients();
        let mut delta: Vec<f32> = Vec::with_capacity(7);
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
