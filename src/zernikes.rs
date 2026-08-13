use std::{fmt::Display, mem};

use ffi::zernikeS;

use crate::{builders::ZernikeSBuilder, cu::Double, Builder, Cu, FromBuilder};

/// Zernike surface
pub struct ZernikeS {
    pub(crate) _c_: zernikeS,
    pub(crate) max_n: i32,
    pub(crate) n_mode: i32,
    pub(crate) n_surf: i32,
    pub(crate) a: Vec<f64>,
}

impl Default for ZernikeS {
    fn default() -> Self {
        Self::builder().build().unwrap()
    }
}

impl FromBuilder for ZernikeS {
    type ComponentBuilder = ZernikeSBuilder;
}

impl Display for ZernikeS {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Zernike surface(x{}) with {} modes",
            self.n_surf, self.n_mode
        )
    }
}

impl ZernikeS {
    /// Returns the number of radial order
    pub fn n_radial_order(&self) -> usize {
        self.max_n as usize
    }
    /// Returns the number of modes
    pub fn n_mode(&self) -> usize {
        self.n_mode as usize
    }
    /// Updates the Zernike coefficients
    pub fn update(&mut self, a: impl Into<Vec<f64>>) -> &mut Self {
        let a = a.into();
        assert_eq!(
            a.len(),
            self.n_mode as usize,
            "expected {} Zernike coefficients, found {}",
            self.n_mode,
            a.len()
        );
        let _ = mem::replace(&mut self.a, a);
        unsafe {
            self._c_.update(self.a.as_mut_ptr());
        }
        self
    }
    /// Resets the Zernike coefficients to 0
    pub fn reset(&mut self) -> &mut Self {
        self.a.fill(0f64);
        unsafe {
            self._c_.update(self.a.as_mut_ptr());
        }
        self
    }
    /// Computes the Zernike surface(s)
    pub fn surface(&mut self, r: &mut Cu<Double>, o: &mut Cu<Double>) -> Vec<f64> {
        let mut surface = Cu::<Double>::vector(r.size());
        surface.malloc();
        for i in 0..self.n_surf {
            unsafe {
                self._c_.surface1(
                    surface.as_mut_ptr(),
                    r.as_ptr(),
                    o.as_ptr(),
                    r.size() as i32,
                    i,
                );
            }
        }
        surface.into()
    }
}

#[cfg(test)]
mod test {
    use std::f64;

    use triangle_rs::Delaunay;

    use super::*;

    #[test]
    pub fn empty() {
        let zs = ZernikeS::default();
        println!("{zs}");
    }

    #[test]
    pub fn builder() {
        let zs = ZernikeS::builder().n_radial_order(5).build().unwrap();
        println!("{zs}");
    }

    #[test]
    pub fn buildern() {
        let zs = ZernikeS::builder()
            .n_radial_order(5)
            .n_surface(7)
            .build()
            .unwrap();
        println!("{zs}");
    }

    #[test]
    pub fn surface() {
        let mut zs = ZernikeS::builder().n_radial_order(11).build().unwrap();
        println!("{zs}");

        let mut a = vec![0f64; zs.n_mode()];
        a[65] = 1f64;
        zs.update(a);

        let n = 101;
        let ps: Vec<_> = (0..n)
            .flat_map(|i| {
                let (s, c) = (2f64 * i as f64 * f64::consts::PI / n as f64).sin_cos();
                vec![c, s]
            })
            .collect();
        let mesh = Delaunay::builder()
            .add_polygon(&ps)
            .set_switches("qQDa0.001")
            .build();
        println!("{mesh}");
        let (r, o): (Vec<_>, Vec<_>) = mesh
            .vertex_iter()
            .map(|xy| (xy[0].hypot(xy[1]), xy[1].atan2(xy[0])))
            .unzip();
        let mut cu_r = Cu::<Double>::from(r);
        let mut cu_o = Cu::<Double>::from(o);
        let s = zs.surface(&mut cu_r, &mut cu_o);
        let red_s: Vec<_> = mesh.triangle_iter().map(|idx| (s[idx[0]]+s[idx[1]]+s[idx[2]])/3f64).collect();
        let iter = mesh.triangle_vertex_iter();
        let _ = complot::tri::Mesh::from((iter, None));
        let iter = mesh.triangle_vertex_iter().zip(&red_s).map(|(xy, s)| (xy, *s));
        let _ = complot::tri::Heatmap::from((iter, None));
    }
}
