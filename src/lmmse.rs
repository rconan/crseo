use crate::{
    builders::{AtmosphereBuilder, SourceBuilder},
    cu::Single,
    Atmosphere, Builder, Conversion, Cu, FromBuilder, GeometricShackHartmann as WFS, Gmt, Mask,
    Result, Source,
};
use ffi::LMMSE as ceo_LMMSE;
use std::ffi::CString;

pub struct LinearMinimumMeanSquareError {
    _c_: ceo_LMMSE,
    atm: Atmosphere,
    guide_star: Source,
    mmse_star: Source,
    fov_diameter: Option<f64>,
    pupil_mask: Mask,
}
#[derive(Debug, Clone)]
pub struct LinearMinimumMeanSquareErrorBuilder {
    pub atm: AtmosphereBuilder,
    pub guide_star: SourceBuilder,
    pub mmse_star: SourceBuilder,
    pub fov_diameter: Option<f64>,
    pub n_side_lenslet: usize,
    pub solver_id: String,
    pub wavefront_osf: usize,
}
impl Default for LinearMinimumMeanSquareErrorBuilder {
    fn default() -> Self {
        LinearMinimumMeanSquareErrorBuilder {
            atm: Atmosphere::builder(),
            guide_star: Source::builder(),
            mmse_star: Source::builder(),
            fov_diameter: None,
            n_side_lenslet: 0,
            solver_id: "MINRES".to_owned(),
            wavefront_osf: 1,
        }
    }
}
impl LinearMinimumMeanSquareErrorBuilder {
    pub fn atmosphere(self, atm: AtmosphereBuilder) -> Self {
        Self { atm, ..self }
    }
    pub fn guide_star(self, guide_star: &Source) -> Self {
        Self {
            guide_star: SourceBuilder::from(guide_star),
            ..self
        }
    }
    pub fn mmse_star(self, mmse_star: &Source) -> Self {
        Self {
            mmse_star: SourceBuilder::from(mmse_star),
            fov_diameter: None,
            ..self
        }
    }
    pub fn fov_diameter(self, fov_diameter: f64) -> Self {
        Self {
            fov_diameter: Some(fov_diameter),
            ..self
        }
    }
    pub fn n_side_lenslet(self, n_side_lenslet: usize) -> Self {
        Self {
            n_side_lenslet,
            ..self
        }
    }
}
impl Builder for LinearMinimumMeanSquareErrorBuilder {
    type Component = LinearMinimumMeanSquareError;
    fn build(self) -> Result<LinearMinimumMeanSquareError> {
        let mut gmt = Gmt::builder().build().unwrap();
        let mut mmse_star = self.mmse_star.build().unwrap();
        mmse_star.through(&mut gmt).xpupil();
        let mut pupil_mask = Mask::new();
        let n_actuator = self.n_side_lenslet + 1;
        let d = self.guide_star.pupil_size / self.n_side_lenslet as f64;
        pupil_mask
            .build(n_actuator * n_actuator)
            .filter(&mut mmse_star.amplitude().into());
        let mut lmmse = LinearMinimumMeanSquareError {
            _c_: Default::default(),
            atm: self.atm.build()?,
            guide_star: self.guide_star.build()?,
            mmse_star,
            fov_diameter: self.fov_diameter,
            pupil_mask,
        };
        let solver_id = CString::new(self.solver_id.into_bytes()).unwrap();
        match lmmse.fov_diameter {
            Some(fov) => unsafe {
                log::info!("LMMSE for a {:.1}arcmin field of view", fov.to_arcmin());
                lmmse._c_.setup3(
                    lmmse.atm.as_raw_mut_ptr(),
                    lmmse.guide_star.as_raw_mut_ptr(),
                    d as f32,
                    self.n_side_lenslet as i32,
                    lmmse.pupil_mask.as_raw_mut_ptr(),
                    solver_id.into_raw(),
                    self.wavefront_osf as i32,
                    0.5 * fov as f32,
                )
            },
            None => unsafe {
                log::info!("LMMSE for a single point in the field");
                lmmse._c_.setup2(
                    lmmse.atm.as_raw_mut_ptr(),
                    lmmse.guide_star.as_raw_mut_ptr(),
                    lmmse.mmse_star.as_raw_mut_ptr(),
                    d as f32,
                    self.n_side_lenslet as i32,
                    lmmse.pupil_mask.as_raw_mut_ptr(),
                    solver_id.into_raw(),
                    self.wavefront_osf as i32,
                )
            },
        }
        Ok(lmmse)
    }
}
impl FromBuilder for LinearMinimumMeanSquareError {
    type ComponentBuilder = LinearMinimumMeanSquareErrorBuilder;
}
impl LinearMinimumMeanSquareError {
    pub fn get_wavefront_estimate(&mut self, wfs: &mut WFS) -> &mut Self {
        unsafe {
            self._c_.estimation(&wfs.as_mut_ptr().data_proc);
        }
        self
    }
    pub fn phase_as_ptr(&mut self) -> Cu<Single> {
        //println!("PS_E_N_PX: {}",self._c_.PS_E_N_PX);
        let mut phase: Cu<Single> = Cu::vector(self._c_.PS_E_N_PX as usize);
        phase.from_ptr(self._c_.d__phase_est);
        phase
    }
    pub fn n_iteration(&mut self, n_iteration: usize) {
        self._c_.iSolve.N_ITERATION = n_iteration as i32;
    }
    pub fn get_n_iteration(&mut self) -> usize {
        self._c_.iSolve.N_ITERATION as usize
    }
    pub fn calibrate_karhunen_loeve(
        &mut self,
        n_kl: usize,
        first_kl: Option<usize>,
        stroke: Option<f64>,
    ) -> Vec<Vec<f64>> {
        let mut gmt = Gmt::builder().m2_n_mode(n_kl).build().unwrap();
        let mut kl: Vec<Vec<f32>> = vec![];
        let first_kl = first_kl.unwrap_or(0);
        let stroke = stroke.unwrap_or(1e-6);
        for s in 0..7 {
            for k in first_kl..n_kl {
                gmt.m2_modes_ij(s, k, stroke);
                self.mmse_star.through(&mut gmt).xpupil();
                let b_push: Vec<f32> = self.mmse_star.phase_as_ptr().into();
                gmt.m2_modes_ij(s, k, -stroke);
                self.mmse_star.through(&mut gmt).xpupil();
                let b_pull: Vec<f32> = self.mmse_star.phase_as_ptr().into();
                kl.push(
                    b_push
                        .iter()
                        .zip(b_pull.iter())
                        .map(|x| 0.5 * (x.0 - x.1) / stroke as f32)
                        .collect::<Vec<f32>>(),
                );
            }
        }
        gmt.reset();
        let kl_norm = kl
            .iter()
            .map(|x| x.iter().map(|y| (y * y) as f64).sum::<f64>())
            .collect::<Vec<f64>>();
        kl.iter()
            .zip(kl_norm.into_iter())
            .map(|x| x.0.iter().map(|&y| y as f64 / x.1).collect::<Vec<f64>>())
            .collect::<_>()
    }
    pub fn get_karhunen_loeve_coefficients(
        &mut self,
        kln: &Vec<Vec<f64>>,
        stroke: Option<f64>,
    ) -> Vec<f64> {
        let stroke = stroke.unwrap_or(1f64);
        kln.iter()
            .map(|x| {
                x.iter()
                    .zip(Vec::<f32>::from(self.phase_as_ptr()).into_iter())
                    .map(|y| stroke * y.0 * y.1 as f64)
                    .sum::<f64>()
            })
            .collect::<_>()
    }
}
impl Drop for LinearMinimumMeanSquareError {
    fn drop(&mut self) {
        unsafe {
            self._c_.cleanup();
        }
    }
}
