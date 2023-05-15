#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(improper_ctypes)]

include!("bindings.rs");

use std::ptr;

unsafe impl Send for vector {}
impl Default for vector {
    fn default() -> Self {
        Self {
            x: 0.,
            y: 0.,
            z: 0.,
        }
    }
}
impl From<[f64; 3]> for vector {
    fn from(v: [f64; 3]) -> Self {
        Self {
            x: v[0],
            y: v[1],
            z: v[2],
        }
    }
}
unsafe impl Send for mask {}
impl Default for mask {
    fn default() -> Self {
        Self {
            m: ptr::null_mut(),
            f: ptr::null_mut(),
            idx: ptr::null_mut(),
            size_px: [0; 2usize],
            nel: 0,
            nnz: 0f32,
            size_m: [0f32; 2usize],
            area: 0f32,
            delta: [0f32; 2usize],
            handle: ptr::null_mut(),
            d__piston_mask: ptr::null_mut(),
        }
    }
}
unsafe impl Send for complex_amplitude {}
impl Default for complex_amplitude {
    fn default() -> Self {
        Self {
            N_PX: 0,
            N: 0,
            amplitude: ptr::null_mut(),
            phase: ptr::null_mut(),
            M: ptr::null_mut(),
            handle: ptr::null_mut(),
            buffer: ptr::null_mut(),
        }
    }
}
unsafe impl Send for bundle {}
impl Default for bundle {
    fn default() -> Self {
        Self {
            N_RAY: 0,
            d__ray: ptr::null_mut(),
            N_BUNDLE: 0,
            N_RAY_TOTAL: 0,
            d__origin: ptr::null_mut(),
            rot_angle: 0.,
            d__chief_ray: ptr::null_mut(),
            d__chief_origin: ptr::null_mut(),
            V: Default::default(),
            geom: [0; 8usize],
            N_RADIUS: 0,
            N_THETA: 0,
            N_L: 0,
            L: 0.,
            d__sphere_distance: ptr::null_mut(),
            d__sphere_radius: ptr::null_mut(),
            d__sphere_origin: ptr::null_mut(),
            d__piston_mask: ptr::null_mut(),
            refractive_index: 0.,
            d__Vx: ptr::null_mut(),
            d__Vy: ptr::null_mut(),
        }
    }
}
unsafe impl Send for source {}
impl Default for source {
    fn default() -> Self {
        Self {
            N_SRC: 0,
            zenith: 0.,
            azimuth: 0.,
            height: 0.,
            theta_x: 0.,
            theta_y: 0.,
            _zenith_64_: 0.,
            _azimuth_64_: 0.,
            _height_64_: 0.,
            _theta_x_64_: 0.,
            _theta_y_64_: 0.,
            photometric_band: ptr::null(),
            magnitude: 0.,
            N_PHOTON: 0.,
            fwhm: 0.,
            wavefront: Default::default(),
            dev_ptr: ptr::null_mut(),
            tag: [0; 8usize],
            rays_exist: 0,
            rays: Default::default(),
        }
    }
}
unsafe impl Send for coordinate_system {}
impl Default for coordinate_system {
    fn default() -> Self {
        Self {
            origin: ptr::null_mut(),
            euler_angles: ptr::null_mut(),
            N: 0,
            R: ptr::null_mut(),
            d__R: ptr::null_mut(),
            float_R: ptr::null_mut(),
            d__origin: ptr::null_mut(),
            tag: [0; 32usize],
        }
    }
}
unsafe impl Send for modes {}
impl Default for modes {
    fn default() -> Self {
        Self {
            d__x_BM: ptr::null_mut(),
            d__y_BM: ptr::null_mut(),
            d__BM: ptr::null_mut(),
            d__BMS: ptr::null_mut(),
            BM_radius: 0.,
            BM_N_SAMPLE: 0,
            d__BM_buffer: ptr::null_mut(),
            n_mode: 0,
            b: ptr::null_mut(),
            d__b: ptr::null_mut(),
            N: 0,
            filename: [0; 256usize],
            N_SET: 0,
            N_MODE: 0,
            d__s2b: ptr::null_mut(),
        }
    }
}
unsafe impl Send for gmt_m1 {}
impl Default for gmt_m1 {
    fn default() -> Self {
        Self {
            M_ID: 0,
            D_assembly: 0.,
            D_clear: 0.,
            D_full: 0.,
            ri: 0.,
            beta: 0.,
            L: 0.,
            area0: 0.,
            area_fraction: 0.,
            area0_px: 0.,
            area: 0.,
            N: 0,
            depth: 0.,
            aperture_CS: Default::default(),
            conic_CS: Default::default(),
            conic_origin: [Default::default(); 7usize],
            d__conic_origin: ptr::null_mut(),
            conic_c: 0.,
            conic_k: 0.,
            d__conic_c: ptr::null_mut(),
            d__conic_k: ptr::null_mut(),
            rigid_body_CS: Default::default(),
            motion_CS: Default::default(),
            height: 0.,
            V: ptr::null_mut(),
            idx_offset: 0,
            ZS: ptr::null_mut(),
            d__piston_mask: ptr::null_mut(),
            TT_CS: Default::default(),
            d__C: ptr::null_mut(),
            d__D: ptr::null_mut(),
            handle: ptr::null_mut(),
            d__valid_segments: ptr::null_mut(),
            BS: Default::default(),
            d__segment_reflectivity: ptr::null_mut(),
        }
    }
}
unsafe impl Send for gmt_m2 {}
impl Default for gmt_m2 {
    fn default() -> Self {
        Self {
            M_ID: 0,
            D_assembly: 0.,
            D_clear: 0.,
            D_full: 0.,
            ri: 0.,
            beta: 0.,
            L: 0.,
            area0: 0.,
            area_fraction: 0.,
            area0_px: 0.,
            area: 0.,
            N: 0,
            depth: 0.,
            aperture_CS: Default::default(),
            conic_CS: Default::default(),
            conic_origin: [Default::default(); 7usize],
            d__conic_origin: ptr::null_mut(),
            conic_c: 0.,
            conic_k: 0.,
            d__conic_c: ptr::null_mut(),
            d__conic_k: ptr::null_mut(),
            rigid_body_CS: Default::default(),
            motion_CS: Default::default(),
            height: 0.,
            V: ptr::null_mut(),
            idx_offset: 0,
            ZS: ptr::null_mut(),
            d__piston_mask: ptr::null_mut(),
            TT_CS: Default::default(),
            d__C: ptr::null_mut(),
            d__D: ptr::null_mut(),
            handle: ptr::null_mut(),
            d__valid_segments: ptr::null_mut(),
            BS: Default::default(),
            d__segment_reflectivity: ptr::null_mut(),
        }
    }
}
unsafe impl Send for geometricShackHartmann {}
impl Default for geometricShackHartmann {
    fn default() -> Self {
        Self {
            N_WFS: 0,
            N_SIDE_LENSLET: 0,
            N_LENSLET: 0,
            N_ACTUATOR: 0,
            N_SLOPE: 0,
            d__c0: ptr::null_mut(),
            d__cx0: ptr::null_mut(),
            d__cy0: ptr::null_mut(),
            valid_lenslet: Default::default(),
            valid_actuator: Default::default(),
            camera: Default::default(),
            data_proc: Default::default(),
            DFT_osf: 0,
            lenslet_pitch: 0f32,
            pixel_scale: 0f32,
            intensity_threshold: 0f32,
            slopes_gain: 0f32,
            _d__c_: ptr::null_mut(),
            _d__cx_: ptr::null_mut(),
            _d__cy_: ptr::null_mut(),
            N_FRAME: 0,
            handle: ptr::null_mut(),
        }
    }
}
unsafe impl Send for shackHartmann {}
impl Default for shackHartmann {
    fn default() -> Self {
        Self {
            N_WFS: 0,
            N_SIDE_LENSLET: 0,
            N_LENSLET: 0,
            N_ACTUATOR: 0,
            N_SLOPE: 0,
            d__c0: ptr::null_mut(),
            d__cx0: ptr::null_mut(),
            d__cy0: ptr::null_mut(),
            valid_lenslet: Default::default(),
            valid_actuator: Default::default(),
            camera: Default::default(),
            data_proc: Default::default(),
            DFT_osf: 0,
            lenslet_pitch: 0f32,
            pixel_scale: 0f32,
            intensity_threshold: 0f32,
            slopes_gain: 0f32,
        }
    }
}
unsafe impl Send for profile {}
impl Default for profile {
    fn default() -> Self {
        Self {
            L0: 0f32,
            l0: 0f32,
            L: 0f32,
            f: 0f32,
            delta: 0f32,
            N_k: 0f32,
            N_a: 0f32,
            kmin: 0f32,
            altitude: ptr::null_mut(),
            xi0: ptr::null_mut(),
            wind_speed: ptr::null_mut(),
            wind_direction: ptr::null_mut(),
        }
    }
}
unsafe impl Send for atmosphere {}
impl Default for atmosphere {
    fn default() -> Self {
        Self {
            photometric_band: ptr::null_mut(),
            wavelength: 0f32,
            r0: 0f32,
            wavenumber: 0f32,
            N_LAYER: 0,
            field_size: 0f32,
            layers_OSF: 0,
            layers_duration: 0f32,
            layers_tau0: 0f32,
            W: 0f32,
            N_W: 0,
            phase_screen_LAYER: ptr::null_mut(),
            N_DURATION: 0,
            LOCAL_RAND_SEED: 0,
            ID: 0,
            EPH: 0f32,
            d__phase_screen_LAYER: ptr::null_mut(),
            N_PHASE_LAYER: 0,
            mmap_size: 0u64,
            zeta1: ptr::null_mut(),
            eta1: ptr::null_mut(),
            zeta2: ptr::null_mut(),
            eta2: ptr::null_mut(),
            devStates: ptr::null_mut(),
            turbulence: Default::default(),
            d__turbulence: ptr::null_mut(),
            layers: ptr::null_mut(),
            d__layers: ptr::null_mut(),
        }
    }
}
unsafe impl Send for imaging {}
impl Default for imaging {
    fn default() -> Self {
        Self {
            N_PX_PUPIL: 0,
            N_DFT: 0,
            N_SIDE_LENSLET: 0,
            N_LENSLET: 0,
            N_SOURCE: 0,
            N_PX_IMAGE: 0,
            N_PX_CAMERA: 0,
            N_FRAME: 0,
            BIN_IMAGE: 0,
            LOCAL_RAND_SEED: 0,
            plan: 0,
            N_PHOTON_PER_SECOND_PER_FRAME: 0f32,
            N_PHOTON_PER_FRAME: 0f32,
            d__wave_PUPIL: ptr::null_mut(),
            d__frame: ptr::null_mut(),
            zenith: 0f32,
            azimuth: 0f32,
            theta_x: 0f32,
            theta_y: 0f32,
            d__zenith: ptr::null_mut(),
            d__azimuth: ptr::null_mut(),
            d__theta_x: ptr::null_mut(),
            d__theta_y: ptr::null_mut(),
            pixel_scale: 0f32,
            photoelectron_gain: 0f32,
            absolute_pointing: 0,
            devStates: ptr::null_mut(),
        }
    }
}
unsafe impl Send for stopwatch {}
impl Default for stopwatch {
    fn default() -> Self {
        Self {
            elapsedTime: 0f32,
            start: ptr::null_mut(),
            stop: ptr::null_mut(),
        }
    }
}
unsafe impl Send for aaStats {}
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
unsafe impl Send for paStats {}
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
unsafe impl Send for BTBT {}
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
unsafe impl Send for GBTBT {}
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
unsafe impl Send for iterativeSolvers {}
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
unsafe impl Send for LMMSE {}
impl Default for LMMSE {
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
unsafe impl Send for centroiding {}
impl Default for centroiding {
    fn default() -> Self {
        Self {
            _N_SIDE_LENSLET_: 0,
            N_LENSLET: 0,
            N_SOURCE: 0,
            d__c: ptr::null_mut(),
            d__cx: ptr::null_mut(),
            d__cy: ptr::null_mut(),
            d__mass: ptr::null_mut(),
            lenslet_mask: ptr::null_mut(),
            MASK_SET: 0,
            n_data: 0,
            DEV_SHARED_MEM: 0,
            DEV_MAX_THREADS: 0,
            handle: ptr::null_mut(),
            status: 0,
        }
    }
}
unsafe impl Send for pssn {}
impl Default for pssn {
    fn default() -> Self {
        Self {
            N_O: 0,
            N_O0: 0,
            n_byte: 0,
            d__O: ptr::null_mut(),
            d__O0: ptr::null_mut(),
            buffer: ptr::null_mut(),
            d__C: ptr::null_mut(),
            N_PX: 0,
            N: 0,
            d__W: ptr::null_mut(),
            N_OTF: 0,
            N_OTF2: 0,
            NN: 0,
            plan: 0,
            handle: ptr::null_mut(),
            num: 0f32,
            denom: ptr::null_mut(),
        }
    }
}
unsafe impl Send for stats {}
impl Default for stats {
    fn default() -> Self {
        Self {
            handle: ptr::null_mut(),
            status: 0,
        }
    }
}
unsafe impl Send for gpu_double {}
impl Default for gpu_double {
    fn default() -> Self {
        Self {
            dev_data: ptr::null_mut(),
            host_data: ptr::null_mut(),
            N: 0,
            nb: 0,
            S: Default::default(),
            stat: 0,
            handle: ptr::null_mut(),
            cusolverH: ptr::null_mut(),
        }
    }
}
unsafe impl Send for gpu_float {}
impl Default for gpu_float {
    fn default() -> Self {
        Self {
            dev_data: ptr::null_mut(),
            host_data: ptr::null_mut(),
            d_tau: ptr::null_mut(),
            N: 0,
            nb: 0,
            S: Default::default(),
            stat: 0,
            handle: ptr::null_mut(),
            cusolverH: ptr::null_mut(),
        }
    }
}
unsafe impl Send for conic {}
impl Default for conic {
    fn default() -> Self {
        Self {
            ref_frame: Default::default(),
            origin: Default::default(),
            d__origin: ptr::null_mut(),
            c: 0f64,
            k: 0f64,
            refractive_index: 0f64,
            even_asphere_N: 0,
            d__even_asphere_a: ptr::null_mut(),
        }
    }
}

unsafe impl Send for pyramid {}
impl Default for pyramid {
    fn default() -> Self {
        Self {
            N_PX_LENSLET: 0,
            N_SIDE_LENSLET: 0,
            modulation: 0f32,
            modulation_sampling: 0,
            camera: imaging::default(),
            alpha: 0f32,
        }
    }
}
