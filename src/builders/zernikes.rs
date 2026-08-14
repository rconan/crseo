use crate::{Builder, ZernikeS};

/// Zernike surface builder
#[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ZernikeSBuilder {
    max_n: usize,
    n_surf: usize,
}

impl ZernikeSBuilder {
    /// Sets the number of radial orders of the Zernike surface
    pub fn n_radial_order(mut self, n: usize) -> Self {
        self.max_n = n;
        if self.n_surf < 1 {
            self.n_surf = 1;
        }
        self
    }
    /// Sets the number of Zernike surface
    ///
    /// 7 surfaces for the GMT
    pub fn n_surface(mut self, n: usize) -> Self {
        self.n_surf = n;
        self
    }
}

impl Builder for ZernikeSBuilder {
    type Component = ZernikeS;

    fn build(self) -> crate::Result<Self::Component> {
        let ZernikeSBuilder { max_n, n_surf } = self;
        let n_mode = (max_n + 1) * max_n / 2;
        let mut zs = ZernikeS {
            _c_: Default::default(),
            max_n: max_n,
            n_mode: n_mode,
            n_surf: n_surf,
            a: vec![0f64; (n_surf * n_mode) as usize],
        };
        let origin = ffi::vector::default();
        let euler_angles = ffi::vector::default();
        unsafe {
            zs._c_.setup3(
                max_n as i32 - 1,
                zs.a.as_mut_ptr(),
                origin,
                euler_angles,
                n_surf as i32,
            );
        }
        Ok(zs)
    }
}
