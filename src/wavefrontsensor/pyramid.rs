use super::LensletArray;
use crate::{Builder, FromBuilder, Propagation};
use ffi::pyramid;
use serde::{Deserialize, Serialize};

type Mat = nalgebra::DMatrix<f32>;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PyramidBuilder {
    lenslet_array: LensletArray,
    modulation: f32,
    modulation_sampling: i32,
    alpha: f32,
    n_gs: i32,
}
impl Default for PyramidBuilder {
    fn default() -> Self {
        Self {
            lenslet_array: LensletArray(30, 8, 0f64),
            modulation: 0f32,
            modulation_sampling: 0i32,
            alpha: 0.5f32,
            n_gs: 1,
        }
    }
}
impl FromBuilder for Pyramid {
    type ComponentBuilder = PyramidBuilder;
}
impl PyramidBuilder {
    pub fn n_lenslet(mut self, n_lenslet: usize) -> Self {
        self.lenslet_array.0 = n_lenslet;
        self
    }
}

pub struct Pyramid {
    _c_: pyramid,
    lenslet_array: LensletArray,
    alpha: f32,
}
impl Drop for Pyramid {
    /// Frees CEO memory before dropping `Pyramid`
    fn drop(&mut self) {
        unsafe {
            self._c_.cleanup();
        }
    }
}
impl Builder for PyramidBuilder {
    type Component = Pyramid;

    fn build(self) -> crate::Result<Self::Component> {
        let mut pym = Pyramid {
            _c_: pyramid::default(),
            lenslet_array: self.lenslet_array,
            alpha: self.alpha,
        };
        let LensletArray(n_side_lenslet, n_px_lenslet, _) = self.lenslet_array;
        let n_pupil_sampling = n_side_lenslet * n_px_lenslet;
        unsafe {
            pym._c_.setup(
                n_side_lenslet as i32,
                n_pupil_sampling as i32,
                self.modulation,
                self.modulation_sampling,
                self.alpha,
                self.n_gs,
            );
        };

        Ok(pym)
    }
}

impl Propagation for Pyramid {
    fn propagate(&mut self, src: &mut crate::Source) {
        unsafe {
            self._c_
                .camera
                .propagateThroughPyramid(src.as_raw_mut_ptr(), self.alpha)
        }
    }

    fn time_propagate(&mut self, _secs: f64, _src: &mut crate::Source) {
        todo!()
    }
}

impl Pyramid {
    pub fn frame(&self) -> Vec<f32> {
        let n = self._c_.camera.N_PX_CAMERA.pow(2) * self._c_.camera.N_SOURCE;
        let mut frame = vec![0f32; n as usize];
        unsafe {
            ffi::dev2host(frame.as_mut_ptr(), self._c_.camera.d__frame, n);
        }
        frame
    }
    pub fn reset(&mut self) {
        unsafe {
            self._c_.camera.reset();
        }
    }
    #[inline]
    pub fn n_px_camera(&self) -> usize {
        self._c_.camera.N_PX_CAMERA as usize
    }
    pub fn pupil_sampling(&self) -> usize {
        self.lenslet_array.0 * self.lenslet_array.1
    }
    pub fn camera_resolution(&self) -> (usize, usize) {
        (self.n_px_camera(), self.n_px_camera())
    }
    pub fn data(&mut self) -> (Mat, Mat) {
        let (n, m) = self.camera_resolution();
        let LensletArray(n_lenslet, _, _) = self.lenslet_array;
        let n0 = n_lenslet / 2;
        let n1 = n0 + n / 2;
        let mat: Mat = nalgebra::DMatrix::from_column_slice(n, m, &self.frame());
        let row_diff = mat.rows(n0, n_lenslet) - mat.rows(n1, n_lenslet);
        let row_col_data = row_diff.columns(n0, n_lenslet) + row_diff.columns(n1, n_lenslet);
        let col_diff = mat.columns(n0, n_lenslet) - mat.columns(n1, n_lenslet);
        let col_row_data = col_diff.rows(n0, n_lenslet) + col_diff.rows(n1, n_lenslet);
        (row_col_data, col_row_data)
    }
    pub fn add_quads(&mut self) -> Mat {
        let (n, m) = self.camera_resolution();
        let LensletArray(n_lenslet, _, _) = self.lenslet_array;
        let n0 = n_lenslet / 2;
        let n1 = n0 + n / 2;
        let mat: Mat = nalgebra::DMatrix::from_column_slice(n, m, &self.frame());
        let row_diff = mat.rows(n0, n_lenslet) + mat.rows(n1, n_lenslet);
        row_diff.columns(n0, n_lenslet) + row_diff.columns(n1, n_lenslet)
    }
}

#[derive(Default)]
pub struct QuadCell {
    mask: Option<nalgebra::DMatrix<bool>>,
    sxy0: Option<Slopes>,
}
impl QuadCell {
    pub fn new(mask: nalgebra::DMatrix<bool>) -> Self {
        Self {
            mask: Some(mask),
            sxy0: None,
        }
    }
    pub fn set_ref_with(&mut self, slopes: Slopes) {
        self.sxy0 = Some(slopes);
    }
    pub fn sx(&self, slopes: &Slopes) -> Option<Mat> {
        let Some(mask) = self.mask.as_ref() else { return None };
        let (nrows, ncols) = mask.shape();
        let mut slopes_iter = slopes.0.iter().step_by(2);
        Some(Mat::from_iterator(
            nrows,
            ncols,
            mask.iter().map(|m| {
                if *m {
                    *slopes_iter.next().unwrap()
                } else {
                    0f32
                }
            }),
        ))
    }
    pub fn sy(&self, slopes: &Slopes) -> Option<Mat> {
        let Some(mask) = self.mask.as_ref() else { return None };
        let (nrows, ncols) = mask.shape();
        let mut slopes_iter = slopes.0.iter().skip(1).step_by(2);
        Some(Mat::from_iterator(
            nrows,
            ncols,
            mask.iter().map(|m| {
                if *m {
                    *slopes_iter.next().unwrap()
                } else {
                    0f32
                }
            }),
        ))
    }
}

pub struct Slopes(Vec<f32>);
impl From<(&QuadCell, &Pyramid)> for Slopes {
    fn from((qc, pym): (&QuadCell, &Pyramid)) -> Self {
        let (sx, sy, a) = {
            let (n, m) = pym.camera_resolution();
            let LensletArray(n_lenslet, _, _) = pym.lenslet_array;
            let n0 = n_lenslet / 2;
            let n1 = n0 + n / 2;
            let mat: Mat = nalgebra::DMatrix::from_column_slice(n, m, &pym.frame());
            let row_diff = mat.rows(n0, n_lenslet) - mat.rows(n1, n_lenslet);
            let sx = row_diff.columns(n0, n_lenslet) + row_diff.columns(n1, n_lenslet);
            let col_diff = mat.columns(n0, n_lenslet) - mat.columns(n1, n_lenslet);
            let sy = col_diff.rows(n0, n_lenslet) + col_diff.rows(n1, n_lenslet);

            let row_sum = mat.rows(n0, n_lenslet) + mat.rows(n1, n_lenslet);
            let a = row_sum.columns(n0, n_lenslet) + row_sum.columns(n1, n_lenslet);
            (sx, sy, a)
        };

        let iter = sx.into_iter().zip(sy.into_iter()).zip(&a);
        let mut sxy: Vec<_> = if let Some(mask) = qc.mask.as_ref() {
            iter.zip(mask)
                .filter(|(_, &m)| m)
                .flat_map(|(((sx, sy), a), _)| vec![sx / a, sy / a])
                .collect()
        } else {
            iter.flat_map(|((sx, sy), a)| vec![sx / a, sy / a])
                .collect()
        };
        if let Some(Slopes(sxy0)) = qc.sxy0.as_ref() {
            sxy.iter_mut()
                .zip(sxy0)
                .for_each(|(sxy, sxy0)| *sxy -= sxy0);
        }
        Slopes(sxy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{FromBuilder, Gmt, Source};

    #[test]
    fn propagation() {
        let n_lenslet = 90;
        let mut gmt = Gmt::builder().build().unwrap();
        let mut pym = Pyramid::builder().n_lenslet(n_lenslet).build().unwrap();
        let mut src = Source::builder()
            .pupil_sampling(pym.pupil_sampling())
            .build()
            .unwrap();
        src.through(&mut gmt).xpupil().through(&mut pym);

        let _: complot::Heatmap = (
            (pym.frame().as_slice(), dbg!(pym.camera_resolution())),
            None,
        )
            .into();

        let (sx, sy) = pym.data();

        let _: complot::Heatmap = (
            (sx.as_slice(), (n_lenslet, n_lenslet)),
            Some(complot::Config::new().filename("pym_sx.png")),
        )
            .into();

        let _: complot::Heatmap = (
            (sy.as_slice(), (n_lenslet, n_lenslet)),
            Some(complot::Config::new().filename("pym_sy.png")),
        )
            .into();
    }

    #[test]
    fn quad_cell() {
        let sid = 1_usize;
        let n_lenslet = 90;

        let mut gmt = Gmt::builder().build().unwrap();
        gmt.keep(&[sid as i32]);
        let mut src = Source::builder().pupil_sampling(n_lenslet).build().unwrap();
        src.rotate_rays(0.5 * std::f64::consts::FRAC_PI_6);
        src.through(&mut gmt).xpupil();

        let pupil = nalgebra::DMatrix::<f32>::from_iterator(
            n_lenslet,
            n_lenslet,
            src.amplitude().into_iter().rev(),
        );
        let _: complot::Heatmap = (
            (pupil.as_slice(), (n_lenslet, n_lenslet)),
            Some(complot::Config::new().filename("pupil.png")),
        )
            .into();

        let mut quad_cell = QuadCell::new(nalgebra::DMatrix::<bool>::from_iterator(
            pupil.nrows(),
            pupil.ncols(),
            pupil
                .into_iter()
                .map(|p| if *p > 0f32 { true } else { false }),
        ));

        let mut gmt = Gmt::builder().m2("Karhunen-Loeve", 100).build().unwrap();
        gmt.keep(&[sid as i32]);
        let mut pym = Pyramid::builder().n_lenslet(n_lenslet).build().unwrap();
        let mut src = Source::builder()
            .pupil_sampling(pym.pupil_sampling())
            .build()
            .unwrap();
        src.rotate_rays(0.5 * std::f64::consts::FRAC_PI_6);

        src.through(&mut gmt).xpupil().through(&mut pym);
        quad_cell.set_ref_with(Slopes::from((&quad_cell, &pym)));
        pym.reset();

        let kl_mode = 1;
        let kl_a0 = 1e-6;
        gmt.m2_modes_ij(sid - 1, kl_mode, kl_a0);
        src.through(&mut gmt).xpupil();
        let opd = src.phase().clone();
        let opd_minmax =
            opd.iter()
                .fold((f32::INFINITY, f32::NEG_INFINITY), |(min, max), value| {
                    (
                        if *value < min { *value } else { min },
                        if *value > max { *value } else { max },
                    )
                });
        let o2p = (2. * std::f64::consts::PI / src.wavelength()) as f32;
        let phase_minmax = (o2p * opd_minmax.0, o2p * opd_minmax.1);
        println!("𝜑 minmax: {:?}", phase_minmax);
        let kl_coef = 1e-2 * kl_a0 as f32 / phase_minmax.0.abs().max(phase_minmax.1);
        println!("KL coef.:{:e}", kl_coef);

        gmt.m2_modes_ij(sid - 1, kl_mode, kl_coef as f64);
        src.through(&mut gmt).xpupil().through(&mut pym);

        serde_pickle::to_writer(
            &mut std::fs::File::create(format!("KL{kl_mode}.pkl")).unwrap(),
            &pym.frame(),
            Default::default(),
        )
        .unwrap();

        let _: complot::Heatmap = (
            (
                src.phase().as_slice(),
                (pym.pupil_sampling(), pym.pupil_sampling()),
            ),
            Some(complot::Config::new().filename("kl.png")),
        )
            .into();

        let _: complot::Heatmap = (
            (pym.frame().as_slice(), dbg!(pym.camera_resolution())),
            None,
        )
            .into();

        let slopes = Slopes::from((&quad_cell, &pym));

        let _: complot::Heatmap = (
            (
                quad_cell.sx(&slopes).unwrap().as_slice(),
                (n_lenslet, n_lenslet),
            ),
            Some(complot::Config::new().filename("pym_sx.png")),
        )
            .into();

        let _: complot::Heatmap = (
            (
                quad_cell.sy(&slopes).unwrap().as_slice(),
                (n_lenslet, n_lenslet),
            ),
            Some(complot::Config::new().filename("pym_sy.png")),
        )
            .into();
    }

    #[test]
    fn karhunen_loeve() {
        let sid = 7_usize;
        let n_lenslet = 90;

        let mut gmt = Gmt::builder().build().unwrap();
        gmt.keep(&[sid as i32]);
        let mut src = Source::builder().pupil_sampling(n_lenslet).build().unwrap();
        src.rotate_rays(0.5 * std::f64::consts::FRAC_PI_6);
        src.through(&mut gmt).xpupil();

        let pupil = nalgebra::DMatrix::<f32>::from_iterator(
            n_lenslet,
            n_lenslet,
            src.amplitude().into_iter().rev(),
        );
        let _: complot::Heatmap = (
            (pupil.as_slice(), (n_lenslet, n_lenslet)),
            Some(complot::Config::new().filename("pupil.png")),
        )
            .into();

        let mut gmt = Gmt::builder().m2("Karhunen-Loeve", 100).build().unwrap();
        gmt.keep(&[sid as i32]);
        let mut pym = Pyramid::builder().n_lenslet(n_lenslet).build().unwrap();
        let mut src = Source::builder()
            .pupil_sampling(pym.pupil_sampling())
            .build()
            .unwrap();
        src.rotate_rays(0.5 * std::f64::consts::FRAC_PI_6);

        src.through(&mut gmt).xpupil().through(&mut pym);
        let (mut sx0, mut sy0) = pym.data();
        let a0 = pym.add_quads();
        sx0.iter_mut()
            .zip(&mut sy0)
            .zip(&a0)
            .zip(&pupil)
            .for_each(|(((sx, sy), a), p)| {
                if *p > 0f32 {
                    *sx /= a;
                    *sy /= a;
                } else {
                    *sx = 0f32;
                    *sy = 0f32;
                }
            });
        pym.reset();

        let kl_mode = 2;

        gmt.m2_modes_ij(sid - 1, kl_mode, 1e-6);
        src.through(&mut gmt).xpupil();
        let opd = src.phase().clone();
        let opd_minmax =
            opd.iter()
                .fold((f32::INFINITY, f32::NEG_INFINITY), |(min, max), value| {
                    (
                        if *value < min { *value } else { min },
                        if *value > max { *value } else { max },
                    )
                });
        let o2p = (2. * std::f64::consts::PI / src.wavelength()) as f32;
        let phase_minmax = (o2p * opd_minmax.0, o2p * opd_minmax.1);
        println!("𝜑 minmax: {:?}", phase_minmax);
        let kl_coef = 0.1e-6 / phase_minmax.0.abs().max(phase_minmax.1);
        println!("KL coef.:{:e}", kl_coef);

        gmt.m2_modes_ij(sid - 1, kl_mode, kl_coef as f64 / 2.);
        src.through(&mut gmt).xpupil().through(&mut pym);

        serde_pickle::to_writer(
            &mut std::fs::File::create(format!("KL{kl_mode}.pkl")).unwrap(),
            &pym.frame(),
            Default::default(),
        )
        .unwrap();

        let _: complot::Heatmap = (
            (
                src.phase().as_slice(),
                (pym.pupil_sampling(), pym.pupil_sampling()),
            ),
            Some(complot::Config::new().filename("kl.png")),
        )
            .into();

        let _: complot::Heatmap = (
            (pym.frame().as_slice(), dbg!(pym.camera_resolution())),
            None,
        )
            .into();

        let (mut sx, mut sy) = pym.data();

        let a = pym.add_quads();
        sx.iter_mut()
            .zip(&mut sy)
            .zip(&a)
            .zip(&pupil)
            .for_each(|(((sx, sy), a), p)| {
                if *p > 0f32 {
                    *sx /= a;
                    *sy /= a;
                } else {
                    *sx = 0f32;
                    *sy = 0f32;
                }
            });
        sx -= &sx0;
        sy -= &sy0;
        dbg!(sx.sum());
        dbg!(sy.sum());
        let _: complot::Heatmap = (
            (sx.as_slice(), (n_lenslet, n_lenslet)),
            Some(complot::Config::new().filename("pym_sx.png")),
        )
            .into();

        let _: complot::Heatmap = (
            (sy.as_slice(), (n_lenslet, n_lenslet)),
            Some(complot::Config::new().filename("pym_sy.png")),
        )
            .into();
    }

    #[test]
    fn add_quads() {
        let n_lenslet = 90;
        let mut gmt = Gmt::builder().build().unwrap();
        let mut pym = Pyramid::builder().n_lenslet(n_lenslet).build().unwrap();
        let mut src = Source::builder()
            .pupil_sampling(pym.pupil_sampling())
            .build()
            .unwrap();
        src.rotate_rays(0.5 * std::f64::consts::FRAC_PI_6);

        src.through(&mut gmt).xpupil().through(&mut pym);

        let a = pym.add_quads();

        let _: complot::Heatmap = (
            (a.as_slice(), (n_lenslet, n_lenslet)),
            Some(complot::Config::new().filename("pym_a15.png")),
        )
            .into();
    }

    #[test]
    fn add_quads_with_kl() {
        let n_lenslet = 90;

        let mut gmt = Gmt::builder().build().unwrap();
        gmt.keep(&[1]);
        let mut src = Source::builder().pupil_sampling(n_lenslet).build().unwrap();
        src.rotate_rays(0.5 * std::f64::consts::FRAC_PI_6);
        src.through(&mut gmt).xpupil();

        let pupil = nalgebra::DMatrix::<f32>::from_iterator(
            n_lenslet,
            n_lenslet,
            src.amplitude().into_iter().rev(),
        );
        let _: complot::Heatmap = (
            (pupil.as_slice(), (n_lenslet, n_lenslet)),
            Some(complot::Config::new().filename("pupil.png")),
        )
            .into();

        let mut gmt = Gmt::builder().m2("Karhunen-Loeve", 100).build().unwrap();
        gmt.keep(&[1]);
        let mut pym = Pyramid::builder().n_lenslet(n_lenslet).build().unwrap();
        let mut src = Source::builder()
            .pupil_sampling(pym.pupil_sampling())
            .build()
            .unwrap();
        src.rotate_rays(0.5 * std::f64::consts::FRAC_PI_6);

        src.through(&mut gmt).xpupil().through(&mut pym);

        let mut a0 = pym.add_quads();
        a0.iter_mut().zip(&pupil).for_each(|(a, p)| *a *= p);
        let _: complot::Heatmap = (
            (a0.as_slice(), (n_lenslet, n_lenslet)),
            Some(complot::Config::new().filename("pym_a0.png")),
        )
            .into();
        pym.reset();

        let kl_mode = 63;
        let sid = 1;

        let kl_a0 = -1e-6;
        gmt.m2_modes_ij(sid - 1, kl_mode, kl_a0);
        src.through(&mut gmt).xpupil();
        let opd = src.phase().clone();
        let opd_minmax =
            opd.iter()
                .fold((f32::INFINITY, f32::NEG_INFINITY), |(min, max), value| {
                    (
                        if *value < min { *value } else { min },
                        if *value > max { *value } else { max },
                    )
                });
        let o2p = (2. * std::f64::consts::PI / src.wavelength()) as f32;
        let phase_minmax = (o2p * opd_minmax.0, o2p * opd_minmax.1);
        println!("𝜑 minmax: {:?}", phase_minmax);
        let kl_coef = 1e-2 * kl_a0 as f32 / phase_minmax.0.abs().max(phase_minmax.1);
        println!("KL coef.:{:e}", kl_coef);

        gmt.m2_modes_ij(sid - 1, kl_mode, kl_coef as f64);
        src.through(&mut gmt).xpupil().through(&mut pym);

        let _: complot::Heatmap = (
            (
                src.phase().as_slice(),
                (pym.pupil_sampling(), pym.pupil_sampling()),
            ),
            Some(complot::Config::new().filename("kl.png")),
        )
            .into();

        let mut a = pym.add_quads() - &a0;
        a.iter_mut().zip(&pupil).for_each(|(a, p)| *a *= p);
        let _: complot::Heatmap = (
            (a.as_slice(), (n_lenslet, n_lenslet)),
            Some(complot::Config::new().filename("pym_a.png")),
        )
            .into();
    }
}
