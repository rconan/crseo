use super::ceo_bindings::{
    aaStats, iterativeSolvers, paStats, stopwatch, BTBT, GBTBT, LMMSE as ceo_LMMSE,
};
use super::{
    cu::Single, Atmosphere, Builder, Conversion, Cu, GeometricShackHartmann as WFS, Mask, Result,
    Source, ATMOSPHERE, GMT, SOURCE,
};
use std::{ffi::CString, ptr};

impl Default for stopwatch {
    fn default() -> Self {
        Self {
            elapsedTime: 0f32,
            start: ptr::null_mut(),
            stop: ptr::null_mut(),
        }
    }
}
impl Default for aaStats {
    fn default() -> Self {
        Self {
            N: 0,
            N2: 0,
            NU: 0,
            NU2: 0,
            NF: 0,
            NF2: 0,
            psd_size: 0,
            cov_size: 0,
            ind_size: 0,
            d__psd: ptr::null_mut(),
            d__cov: ptr::null_mut(),
            d__alpha: ptr::null_mut(),
            d__beta: ptr::null_mut(),
            n_full: 0f32,
            n_comp: 0f32,
            b_full: 0f32,
            b_comp: 0f32,
            cov_eval_et: 0f32,
            sampling: 0f32,
            plan: 0,
            N_SRC2: 0,
        }
    }
}
impl Default for paStats {
    fn default() -> Self {
        Self {
            osf: 0,
            M: 0,
            shift: 0,
            M_LAYER: ptr::null_mut(),
            N: 0,
            N2: 0,
            NU: 0,
            NU2: 0,
            NF: 0,
            NF2: 0,
            psd_size: 0,
            cov_size: 0,
            ind_size: 0,
            d__psd: ptr::null_mut(),
            d__cov: ptr::null_mut(),
            d__alpha: ptr::null_mut(),
            d__beta: ptr::null_mut(),
            n_full: 0f32,
            n_comp: 0f32,
            b_full: 0f32,
            b_comp: 0f32,
            cov_eval_et: 0f32,
            sampling: 0f32,
            plan: 0,
            N_SRC2: 0,
        }
    }
}
impl Default for BTBT {
    fn default() -> Self {
        Self {
            M: 0,
            N: 0,
            MT: 0,
            MT2: 0,
            NT: 0,
            NT2: 0,
            NU: 0,
            NU2: 0,
            NDFT: 0,
            HALF_NDFT: 0,
            NU_TOTAL: 0,
            NF: 0,
            NF2: 0,
            ind_size: 0,
            cov_size: 0,
            mask: ptr::null_mut(),
            d__cov: ptr::null_mut(),
            d__b: ptr::null_mut(),
            d__c: ptr::null_mut(),
            d__alpha: ptr::null_mut(),
            d__beta: ptr::null_mut(),
            n_full: 0f32,
            n_comp: 0f32,
            b_full: 0f32,
            b_comp: 0f32,
            cov_eval_et: 0f32,
            d__mu: ptr::null_mut(),
            d__xi: ptr::null_mut(),
            raster_plan: 0,
            MVM_input_plan: 0,
            MVM_output_plan: 0,
        }
    }
}
impl Default for GBTBT {
    fn default() -> Self {
        Self {
            M: 0,
            N: 0,
            NT: 0,
            NT2: 0,
            NDFT: 0,
            HALF_NDFT: 0,
            NU_TOTAL: 0,
            NF: 0,
            NF2: 0,
            ind_size: 0,
            cov_size: 0,
            MT2_TOTAL: 0,
            MT_size: 0,
            MAX_MT: 0,
            MT: ptr::null_mut(),
            MT2: ptr::null_mut(),
            NU: ptr::null_mut(),
            NU2: ptr::null_mut(),
            CS_MT2: ptr::null_mut(),
            d__MT: ptr::null_mut(),
            d__MT2: ptr::null_mut(),
            d__NU: ptr::null_mut(),
            d__NU2: ptr::null_mut(),
            d__CS_MT2: ptr::null_mut(),
            mask: ptr::null_mut(),
            d__cov: ptr::null_mut(),
            d__b: ptr::null_mut(),
            d__c: ptr::null_mut(),
            d__alpha: ptr::null_mut(),
            d__beta: ptr::null_mut(),
            n_full: 0f32,
            n_comp: 0f32,
            b_full: 0f32,
            b_comp: 0f32,
            cov_eval_et: 0f32,
            d__mu: ptr::null_mut(),
            d__xi: ptr::null_mut(),
            raster_plan: 0,
            MVM_input_plan: 0,
            MVM_output_plan: 0,
        }
    }
}
impl Default for iterativeSolvers {
    fn default() -> Self {
        Self {
            d__vectors: ptr::null_mut(),
            q: ptr::null_mut(),
            x: ptr::null_mut(),
            r: ptr::null_mut(),
            p: ptr::null_mut(),
            z: ptr::null_mut(),
            nu_i: ptr::null_mut(),
            nu_im1: ptr::null_mut(),
            nu_ip1: ptr::null_mut(),
            w_i: ptr::null_mut(),
            w_im1: ptr::null_mut(),
            w_im2: ptr::null_mut(),
            rnorm: 0f32,
            rel_rnorm: 0f32,
            mean_time_per_iteration: 0f32,
            RTOL: 0f32,
            ATOL: 0f32,
            N: 0,
            N_ITERATION: 0,
            cvgce_iteration: 0,
            tid: Default::default(),
            handle: ptr::null_mut(),
            status: 0,
            VERBOSE: 0,
        }
    }
}
impl Default for ceo_LMMSE {
    fn default() -> Self {
        Self {
            d__idx: ptr::null_mut(),
            PS_E_N_PX: 0,
            N_guide_star: 0,
            N_mmse_star: 0,
            offset: 0,
            N_SIDE_LENSLET_: 0,
            NP: 0,
            NS: 0,
            osf: 0,
            d__ce: ptr::null_mut(),
            d__phase_est: ptr::null_mut(),
            d__phase_est_c: ptr::null_mut(),
            d__phase_est_i: ptr::null_mut(),
            d__x: ptr::null_mut(),
            d__zp_x: ptr::null_mut(),
            aa: Default::default(),
            aaCov: Default::default(),
            pa: Default::default(),
            paCov: Default::default(),
            iSolve: Default::default(),
            tid: Default::default(),
            nnz: 0,
            NI: 0,
            csrValH: ptr::null_mut(),
            csrColIndH: ptr::null_mut(),
            csrRowPtrH: ptr::null_mut(),
            alpha: 0f32,
            beta: 0f32,
            elapsed_time: 0f32,
            cudaStat: 0,
            status: 0,
            handle: ptr::null_mut(),
            descr: ptr::null_mut(),
            start: ptr::null_mut(),
            stop: ptr::null_mut(),
        }
    }
}
pub struct LinearMinimumMeanSquareError {
    _c_: ceo_LMMSE,
    atm: Atmosphere,
    guide_star: Source,
    mmse_star: Source,
    fov_diameter: Option<f64>,
    pupil_mask: Mask,
}
#[derive(Debug, Clone)]
pub struct LMMSE {
    pub atm: super::ATMOSPHERE,
    pub guide_star: super::SOURCE,
    pub mmse_star: super::SOURCE,
    pub fov_diameter: Option<f64>,
    pub n_side_lenslet: usize,
    pub solver_id: String,
    pub wavefront_osf: usize,
}
impl Default for LMMSE {
    fn default() -> Self {
        LMMSE {
            atm: super::ATMOSPHERE::new(),
            guide_star: super::SOURCE::new(),
            mmse_star: super::SOURCE::new(),
            fov_diameter: None,
            n_side_lenslet: 0,
            solver_id: "MINRES".to_owned(),
            wavefront_osf: 1,
        }
    }
}
impl LMMSE {
    pub fn atmosphere(self, atm: ATMOSPHERE) -> Self {
        Self { atm, ..self }
    }
    pub fn guide_star(self, guide_star: &Source) -> Self {
        Self {
            guide_star: SOURCE::from(guide_star),
            ..self
        }
    }
    pub fn mmse_star(self, mmse_star: &Source) -> Self {
        Self {
            mmse_star: SOURCE::from(mmse_star),
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
impl Builder for LMMSE {
    type Component = LinearMinimumMeanSquareError;
    fn build(self) -> Result<LinearMinimumMeanSquareError> {
        let mut gmt = GMT::new().build().unwrap();
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
impl LinearMinimumMeanSquareError {
    pub fn get_wavefront_estimate(&mut self, wfs: &mut WFS) -> &mut Self {
        unsafe {
            self._c_.estimation(&wfs.as_raw_mut_ptr().data_proc);
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
        let mut gmt = GMT::new().m2_n_mode(n_kl).build().unwrap();
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
