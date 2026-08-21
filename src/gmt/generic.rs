use std::fmt::Display;

use super::{GmtM1, GmtM2, GmtMx, Mirror, ModeKind, SurfaceMode, ZernikeMode};

/// GMT wrapper
pub struct GmtGeneric<K1 = SurfaceMode, K2 = SurfaceMode>
where
    K1: ModeKind,
    K2: ModeKind,
    GmtM1: GmtMx<K1>,
    GmtM2: GmtMx<K2>,
{
    pub m1: Mirror<GmtM1, K1>,
    pub m2: Mirror<GmtM2, K2>,
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
    pub(crate) m1_truss_projection: bool,
}
pub type Gmt = GmtGeneric;
pub type ZernikeGmt = GmtGeneric<ZernikeMode, ZernikeMode>;
impl<K1, K2> Display for GmtGeneric<K1, K2>
where
    K1: ModeKind,
    K2: ModeKind,
    GmtM1: GmtMx<K1>,
    GmtM2: GmtMx<K2>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.m1_truss_projection {
            write!(f, "GMT: {}, {}", self.m1, self.m2)?;
        } else {
            write!(f, "GMT (no trusses): {}, {}", self.m1, self.m2)?;
        };
        Ok(())
    }
}
