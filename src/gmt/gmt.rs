use crate::{Builder, CrseoError, FromBuilder, Propagation, Result, Source};
use ffi::{gmt_m1, gmt_m2, vector};
use serde::{Deserialize, Serialize};
use std::{
    env,
    ffi::{CStr, CString},
    path::Path,
};

/* pub enum ModeType {
    None,
    Zernike(usize),
    Modes(String),
}
 */
#[doc(hidden)]
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GmtBuilder {
    m1: Mirror,
    m2: Mirror,
}
impl Default for GmtBuilder {
    fn default() -> Self {
        GmtBuilder {
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
impl GmtBuilder {
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
impl Builder for GmtBuilder {
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
impl From<&Gmt> for GmtBuilder {
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
impl FromBuilder for Gmt {
    type ComponentBuilder = GmtBuilder;
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
    /// * `sid` - vector of segment ID numbers in the range \[1,7\]
    pub fn keep(&mut self, sid: &[i32]) -> &mut Self {
        unsafe {
            self._c_m1.keep(sid.as_ptr() as *mut _, sid.len() as i32);
            self._c_m2.keep(sid.as_ptr() as *mut _, sid.len() as i32);
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
            self._c_m1.update(t_xyz, r_xyz, sid);
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
            self._c_m2.update(t_xyz, r_xyz, sid);
        }
    }
    /// Sets M1 modal coefficients
    ///
    /// The coefficients are given segment wise
    /// with the same number of modes per segment
    pub fn m1_modes(&mut self, a: &[f64]) {
        let a_n_mode = a.len() / 7;
        self.a1
            .chunks_mut(self.m1_n_mode)
            .zip(a.chunks(a_n_mode))
            .for_each(|(a1, a)| a1.iter_mut().zip(a).for_each(|(a1, a)| *a1 = *a));
        unsafe {
            self._c_m1.BS.update(self.a1.as_mut_ptr());
        }
    }
    pub fn m1_segment_modes(&mut self, a: &[Vec<f64>]) {
        self.a1
            .chunks_mut(self.m1_n_mode)
            .zip(a)
            .for_each(|(a1, a)| a1.iter_mut().zip(a).for_each(|(a1, a)| *a1 = *a));
        unsafe {
            self._c_m1.BS.update(self.a1.as_mut_ptr());
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
    ///
    /// The coefficients are given segment wise
    /// with the same number of modes per segment
    pub fn m2_modes(&mut self, a: &[f64]) {
        let a_n_mode = a.len() / 7;
        self.a2
            .chunks_mut(self.m2_n_mode)
            .zip(a.chunks(a_n_mode))
            .for_each(|(a2, a)| a2.iter_mut().zip(a).for_each(|(a2, a)| *a2 = *a));
        unsafe {
            self._c_m2.BS.update(self.a2.as_mut_ptr());
        }
        /*
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
        */
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
    fn propagate(&mut self, src: &mut Source) {
        unsafe {
            src.as_raw_mut_ptr().reset_rays();
            let rays = &mut src.as_raw_mut_ptr().rays;
            self._c_m2.blocking(rays);
            self._c_m1.trace(rays);
            // rays.gmt_truss_onaxis();
            rays.gmt_m2_baffle();
            self._c_m2.trace(rays);
            rays.to_sphere1(-5.830, 2.197173);
        }
    }
    fn time_propagate(&mut self, _secs: f64, src: &mut Source) {
        self.propagate(src)
    }
}
