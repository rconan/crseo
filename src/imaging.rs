use crate::builders::ImagingBuilder;
use crate::{Cu, FromBuilder};

use super::Propagation;
use super::Source;
use ffi::{dev2host, imaging};
use serde::Deserialize;
use serde::Serialize;
use std::f32;
use std::fmt::Display;

use crate::cu::Single;

/// Lenslet array specifications
///
/// Default properties:
///  - # of lenslet     : 1
///  - lenslet pitch    : 25.5m
///  - # pixel/lenslet  : 511px
#[derive(Debug, Clone, PartialEq, Copy, Serialize, Deserialize)]
pub struct LensletArray {
    pub n_side_lenslet: usize,
    pub n_px_lenslet: usize,
    pub d: f64,
}
impl Default for LensletArray {
    fn default() -> Self {
        LensletArray {
            n_side_lenslet: 1,
            n_px_lenslet: 511,
            d: 25.5,
        }
    }
}
impl LensletArray {
    /// Sets the size of the square lenslet array
    ///
    /// The lenslet pitch is re-computed as well.
    pub fn n_side_lenslet(self, n_side_lenslet: usize) -> Self {
        let d = self.d * self.n_side_lenslet as f64 / n_side_lenslet as f64;
        Self {
            n_side_lenslet,
            d,
            ..self
        }
    }
    /// Sets the number of pixel per lenslet
    pub fn n_px_lenslet(self, n_px_lenslet: usize) -> Self {
        Self {
            n_px_lenslet,
            ..self
        }
    }
    /// Sets the lenslet pitch
    pub fn pitch(self, d: f64) -> Self {
        Self { d, ..self }
    }
}

/// Detector noise model specifications
///
/// Default properties:
///  - # pixel/framelet         : 512px
///  - DFT padding factor (osf) : 2
#[derive(Debug, Clone, PartialEq, Copy, Serialize, Deserialize)]
pub struct Detector {
    pub n_px_framelet: usize,
    pub n_px_imagelet: Option<usize>,
    pub osf: usize,
    pub noise_specs: Option<NoiseDataSheet>,
}
impl Default for Detector {
    fn default() -> Self {
        Detector {
            n_px_framelet: 512,
            n_px_imagelet: None,
            osf: 2,
            noise_specs: None,
        }
    }
}
impl Detector {
    pub fn n_px_framelet(self, n_px_framelet: usize) -> Self {
        Self {
            n_px_framelet,
            ..self
        }
    }
    pub fn n_px_imagelet(self, n_px_imagelet: usize) -> Self {
        Self {
            n_px_imagelet: Some(n_px_imagelet),
            ..self
        }
    }
    pub fn osf(self, osf: usize) -> Self {
        Self { osf, ..self }
    }
    pub fn noise_specs(self, noise_specs: NoiseDataSheet) -> Self {
        Self {
            noise_specs: Some(noise_specs),
            ..self
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
/// Detector noise specifications
pub struct NoiseDataSheet {
    /// Exposure time \[s\]
    pub exposure_time: f64,
    /// Read-out noise rms (# of photo-electron per pixel)
    pub rms_read_out_noise: f64,
    /// Number of background photons per frame
    pub n_background_photon: f64,
    /// Noise excess factor
    pub noise_factor: f64,
}
impl NoiseDataSheet {
    /// Creates a new `NoiseDataSheet` with only photon noise for the given exposure time
    pub fn new(exposure_time: f64) -> Self {
        Self {
            exposure_time,
            ..Default::default()
        }
    }
    /// Creates a new `NoiseDataSheet` with `rms_read_out_noise`
    pub fn read_out(self, rms_read_out_noise: f64) -> Self {
        NoiseDataSheet {
            rms_read_out_noise,
            ..self
        }
    }
    /// Creates a new `NoiseDataSheet` with `n_background_photon`
    pub fn background(self, n_background_photon: f64) -> Self {
        NoiseDataSheet {
            n_background_photon,
            ..self
        }
    }
    /// Creates a new `NoiseDataSheet` with `n_background_photon`
    pub fn excess_noise(self, noise_factor: f64) -> Self {
        NoiseDataSheet {
            noise_factor,
            ..self
        }
    }
}
impl Default for NoiseDataSheet {
    /// Creates a new `NoiseDataSheet` with `rms_read_out_noise`=0, `n_background_photon`=0 and `noise_factor`=1
    fn default() -> Self {
        NoiseDataSheet {
            exposure_time: 1f64,
            rms_read_out_noise: 0f64,
            n_background_photon: 0f64,
            noise_factor: 1f64,
        }
    }
}

#[derive(Clone, Default)]
pub struct Frame {
    pub dev: Cu<Single>,
    pub n_px_camera: usize,
    pub resolution: usize,
    pub n_frame: usize,
    pub idx: usize,
}

// impl Default for Frame {
//     fn default() -> Self {
//         Self {
//             dev: Cu::new(),
//             n_px_camera: Default::default(),
//             resolution: Default::default(),
//             n_frame: Default::default(),
//         }
//     }
// }
impl From<&mut Frame> for Vec<f32> {
    fn from(value: &mut Frame) -> Self {
        value.dev.from_dev()
    }
}

impl From<&Frame> for Vec<f32> {
    fn from(value: &Frame) -> Self {
        let mut dev = Cu::<Single>::vector(value.dev.size());
        dev.from_ptr(value.dev.as_ptr() as *mut _);
        dev.from_dev()
    }
}

impl From<Frame> for Vec<f32> {
    fn from(mut value: Frame) -> Self {
        value.dev.from_dev()
    }
}

impl From<Vec<f32>> for Frame {
    fn from(value: Vec<f32>) -> Self {
        let n = value.len();
        let sqrt_n = (n as f64).sqrt() as usize;
        assert_eq!(n, sqrt_n * sqrt_n, "the frame is not square");
        Self {
            dev: Cu::<Single>::from(value),
            n_px_camera: sqrt_n,
            resolution: sqrt_n,
            n_frame: 1,
            idx: 1,
        }
    }
}

impl Frame {
    pub fn new(data: Vec<f32>, resolution: usize, n_px_camera: usize) -> Self {
        Self {
            dev: Cu::<Single>::from(data),
            n_px_camera,
            resolution,
            n_frame: 1,
            idx: 1,
        }
    }
    pub fn is_empty(&self) -> bool {
        self.idx == 0
    }
    pub fn roi(&self) -> (usize, usize) {
        (self.resolution, self.resolution)
    }
}

/// An optical imager with a detector
///
/// The optical imager is a square lenslet array which focal plane lies on the detector.
/// The detector continuously integrates the images formed on the detector until it is explicitly reset
pub struct Imaging {
    pub(crate) _c_: imaging,
    pub(crate) dft_osf: usize,
    /// lenslet flux threshold
    pub fluxlet_threshold: f64,
}
impl FromBuilder for Imaging {
    type ComponentBuilder = ImagingBuilder;
}
impl Imaging {
    /// Creates a new `Imaging`
    pub fn new() -> Imaging {
        Imaging {
            _c_: Default::default(),
            dft_osf: 1,
            fluxlet_threshold: 0.,
        }
    }
    /*     /// Set `Imaging` parameters
    ///
    /// * `n_sensor` - the number of `Imaging` sensor
    /// * `n_side_lenslet` - the size of the square lenslet array
    /// * `n_px_lenslet` - the number of pixel per lenslet, for a total resolution of (`n_side_lenslet`X`n_px_lenslet`+1)^2
    /// * `dft_osf` - the discrete Fourier transform oversampling factor
    /// * `n_px_imagelet` - the sampling of a lenslet focal plane image
    /// * `binning` - binning factor of a imagelet
    pub fn build(
        &mut self,
        n_sensor: i32,
        n_side_lenslet: i32,
        n_px_lenslet: i32,
        dft_osf: i32,
        n_px_imagelet: i32,
        binning: i32,
    ) -> &mut Self {
        unsafe {
            self._c_.setup3(
                n_px_lenslet,
                n_side_lenslet,
                dft_osf,
                n_px_imagelet,
                binning,
                n_sensor,
            );
        }
        self.dft_osf = dft_osf;
        self
    } */
    pub fn __ceo__(&self) -> &imaging {
        &self._c_
    }
    /// Transfer the frame from the GPU to the host
    pub fn frame_transfer(&mut self, frame: &mut Vec<f32>) -> &mut Self {
        unsafe {
            dev2host(
                frame.as_mut_ptr(),
                self._c_.d__frame,
                self.resolution() * self.resolution() * self._c_.N_SOURCE,
            );
        }
        self
    }
    /// Returns a pointer to the frame on the device
    pub fn frame(&self) -> Frame {
        let mut cu = Cu::<Single>::vector(
            (self.resolution() * self.resolution() * self._c_.N_SOURCE) as usize,
        );
        cu.from_ptr(self._c_.d__frame);
        Frame {
            dev: cu,
            n_px_camera: self._c_.N_PX_CAMERA as usize,
            resolution: self.resolution() as usize,
            n_frame: self._c_.N_SOURCE as usize,
            idx: self._c_.N_FRAME as usize,
        }
    }
    /// Resets the detector frame to zero
    pub fn reset(&mut self) -> &mut Self {
        unsafe {
            self._c_.reset();
        }
        self
    }
    /// Returns the detector resolution
    pub fn resolution(&self) -> i32 {
        self._c_.N_PX_CAMERA * self._c_.N_SIDE_LENSLET
    }
    /// Return the number of frames that have been summed since the last reset
    pub fn n_frame(&self) -> u32 {
        self._c_.N_FRAME as u32
    }
    /// Reads out the detector by adding noise to the frame if a `NoiseDataSheet` is passed and the intensity is scaled according to the detector `exposure` time \[s]\
    pub fn readout(
        &mut self,
        exposure: f64,
        detector_noise_properties: Option<NoiseDataSheet>,
    ) -> &mut Self {
        detector_noise_properties.map_or_else(
            || (),
            |p| unsafe {
                self._c_.readout1(
                    exposure as f32,
                    p.rms_read_out_noise as f32,
                    p.n_background_photon as f32,
                    p.noise_factor as f32,
                );
            },
        );
        self
    }
    /* /// Sets the pixel scale
    pub fn set_pixel_scalep(&mut self, src: &mut Source) -> &mut Self {
        self._c_.pixel_scale = (src.wavelength() / src.pupil_size / self.dft_osf as f64) as f32
            * (self._c_.N_SIDE_LENSLET * self._c_.BIN_IMAGE) as f32;
        self
    } */
    /// Returns the detector pixel scale
    pub fn pixel_scale(&self, src: &Source) -> f32 {
        (src.wavelength() / src.pupil_size / self.dft_osf as f64) as f32
            * (self._c_.N_SIDE_LENSLET * self._c_.BIN_IMAGE) as f32
    }
    /// Returns the lenslet field-of-view
    pub fn field_of_view(&self, src: &Source) -> f32 {
        self.pixel_scale(src) * (self._c_.N_PX_CAMERA as f32)
    }
    /// Sets the detector pointing direction error relative to the guide star
    ///
    /// The error is given in cartesian coordinates and in radians units
    pub fn pointing(
        &mut self,
        xy: &[(f64, f64)],
        src: &Source, // mut zen: Vec<f32>,
                      // mut azi: Vec<f32>,
                      // pixel_scale: f64,
    ) -> &mut Self {
        unsafe {
            assert_eq!(xy.len(), src.size as usize);
            let za = src.zenith.iter().zip(&src.azimuth);
            let (mut zen, mut azi): (Vec<_>, Vec<_>) = xy
                .iter()
                .zip(za)
                .map(|((x, y), (&z, a))| {
                    let (sa, ca) = a.sin_cos();
                    let (x, y) = (*x as f32 + z * ca, *y as f32 + z * sa);
                    (x.hypot(y), y.atan2(x))
                })
                .unzip();
            self._c_.pixel_scale = (src.wavelength() / src.pupil_size / self.dft_osf as f64) as f32
                * (self._c_.N_SIDE_LENSLET) as f32;
            self._c_.absolute_pointing = 1;
            self._c_
                .set_pointing_direction(zen.as_mut_ptr(), azi.as_mut_ptr());
        }
        self
    }
    pub fn n_guide_star(&self) -> i32 {
        self._c_.N_SOURCE
    }
}
impl Display for Imaging {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "Imaging (x{}): {}x{} lenslets, {}x{} pixels, osf {}, {}x{} binning",
            self._c_.N_SOURCE,
            self._c_.N_SIDE_LENSLET,
            self._c_.N_SIDE_LENSLET,
            self._c_.N_PX_CAMERA,
            self._c_.N_PX_CAMERA,
            self.dft_osf,
            self._c_.BIN_IMAGE,
            self._c_.BIN_IMAGE
        )
    }
}
impl Drop for Imaging {
    /// Frees CEO memory before dropping `Imaging`
    fn drop(&mut self) {
        unsafe {
            self._c_.cleanup();
        }
    }
}
impl Propagation for Imaging {
    /// Fourier propagates the wavefront to the focal plane onto the detector
    fn propagate(&mut self, src: &mut Source) {
        unsafe {
            self._c_.propagate(src.as_raw_mut_ptr());
        }
    }
    fn time_propagate(&mut self, _secs: f64, src: &mut Source) {
        self.propagate(src)
    }
}

/* #[cfg(test)]
/// Imaging tests
mod tests {
    use super::{Imaging, NoiseDataSheet};
    use crate::{ceo, Centroiding, Conversion, Source};

    #[test]
    /// Test the intensity per lenslet
    fn imaging_flux() {
        let pupil_size = 25.5f64;
        let n_side_lenslet = 20;
        let n_px_lenslet = 32;
        let pupil_sampling = n_side_lenslet * n_px_lenslet + 1;
        let lenslet_size = (pupil_size / n_side_lenslet as f64) as f32;
        let mut gmt = ceo!(GmtBuilder);
        let mut src = Source::new(1, pupil_size, pupil_sampling);
        src.build("V", vec![0f32], vec![0f32], vec![18f32]);
        let mut sensor = Imaging::new();
        sensor.build(1, n_side_lenslet, n_px_lenslet, 2, 2 * n_px_lenslet, 1);
        let mut cog = Centroiding::new();
        cog.build(n_side_lenslet as u32, None);

        src.through(&mut gmt).xpupil().through(sensor.reset());
        cog.process(&sensor, None);
        let fluxlet = cog
            .lenslet_flux()
            .iter()
            .cloned()
            .fold(-f32::INFINITY, f32::max);
        println!("Light collecting area: {}", src.light_collecting_area());
        println!("Sensor lenslet flux: {}", fluxlet);
        let fluxlet_expected = src.n_photon()[0] * lenslet_size * lenslet_size;
        println!("Lenslet expected flux: {}", fluxlet_expected);
        assert!((fluxlet - fluxlet_expected).abs() / fluxlet_expected < 1e-1);
    }

    #[test]
    fn imaging_noise_photon() {
        let pupil_size = 25.5f64;
        let n_side_lenslet = 40;
        let n_px_lenslet = 16;
        let pupil_sampling = n_side_lenslet * n_px_lenslet + 1;
        let lenslet_size = (pupil_size / n_side_lenslet as f64) as f32;
        let mut gmt = ceo!(GmtBuilder);
        let mut src = Source::new(1, pupil_size, pupil_sampling);
        src.build("V", vec![0f32], vec![0f32], vec![16f32]);
        let fwhm_px = 8f64;
        src.fwhm(fwhm_px);
        let mut sensor = Imaging::new();
        sensor.build(1, n_side_lenslet, n_px_lenslet, 2, 2 * n_px_lenslet, 1);
        let p = sensor.pixel_scale(&mut src) as f64;

        let mut cog0 = Centroiding::new();
        cog0.build(n_side_lenslet as u32, None);
        src.through(&mut gmt).xpupil().through(&mut sensor);
        let nv = cog0.process(&sensor, None).valid_lenslets(Some(0.9), None);
        println!("Valid lenslet #: {}", nv);

        let mut cog = Centroiding::new();
        cog.build(n_side_lenslet as u32, Some(p));
        src.through(&mut gmt).xpupil().through(&mut sensor);
        sensor.readout(1f64, Some(NoiseDataSheet::default()));
        let s = cog
            .process(&sensor, Some(&cog0))
            .grab()
            .valids(Some(&cog0.valid_lenslets));
        println!("Valid slopes #: {}", s.len());
        println!("Pixel scale: {}mas", p.to_mas());
        let m = s.iter().sum::<f32>() / s.len() as f32;
        let v = s.iter().map(|x| (x - m).powi(2)).sum::<f32>() / s.len() as f32;
        println!("Centroid rms error: {}", (v.sqrt() as f64).to_mas());

        let fluxlet = cog
            .lenslet_flux()
            .iter()
            .cloned()
            .fold(-f32::INFINITY, f32::max);
        let fluxlet_expected = src.n_photon()[0] * lenslet_size * lenslet_size;
        println!("expected flux: {}", fluxlet_expected);
        println!("flux ratio: {}", fluxlet / fluxlet_expected);
        //let fwhm = 1.03*src.wavelength()/lenslet_size as f64;
        let fwhm = p * fwhm_px;
        //println!("FWHM: {}mas",fwhm.to_mas());
        println!("FWHM: {}mas", fwhm.to_mas());
        //let v_expected = fwhm.powi(2)/(2f64*2f64.ln()*fluxlet_expected as f64);
        let v_expected = fwhm.powi(2) / (8f64 * 2f64.ln() * fluxlet_expected as f64);
        //println!("Expected centroid rms error: {}",v_expected.sqrt().to_mas());
        println!(
            "Expected centroid rms error: {}",
            v_expected.sqrt().to_mas()
        );

        assert!((v.sqrt() as f64 - v_expected.sqrt()).abs() < 1f64);
    }

    #[test]
    fn imaging_noise_readout() {
        let n_side_lenslet = 40;
        let n_px_lenslet = 16;

        let mut sensor = Imaging::new();
        sensor.build(1, n_side_lenslet, n_px_lenslet, 2, 2 * n_px_lenslet, 1);

        sensor
            .reset()
            .readout(1f64, Some(NoiseDataSheet::new(1.).read_out(1f64)));
        let n = sensor.resolution().pow(2);
        let mut frame = vec![0f32; n as usize];
        sensor.frame_transfer(&mut frame);

        let m = frame.iter().sum::<f32>() / n as f32;
        let v = frame.iter().map(|x| (x - m).powi(2)).sum::<f32>() / n as f32;
        println!("RON: {}", v.sqrt());
        assert!((1f32 - v.sqrt()).abs() < 1e-2);
    }

    #[test]
    fn imaging_noise_background() {
        let n_side_lenslet = 40;
        let n_px_lenslet = 16;

        let mut sensor = Imaging::new();
        sensor.build(1, n_side_lenslet, n_px_lenslet, 2, 2 * n_px_lenslet, 1);

        let n = sensor.resolution().pow(2);
        let nbg_px = 1000f64;
        let n_background_photon = n as f64 * nbg_px;
        println!("Background photon #: {n_background_photon}");
        sensor.reset().readout(
            1f64,
            Some(NoiseDataSheet::new(1.).background(n_background_photon)),
        );
        let mut frame = vec![0f32; n as usize];
        sensor.frame_transfer(&mut frame);

        let mut m = frame.iter().sum::<f32>() / n as f32;
        let mut v = frame.iter().map(|x| (x - m).powi(2)).sum::<f32>() / n as f32;
        m /= n as f32;
        v /= n as f32;
        println!("background photon: [{},{}]", m, v);
        assert!((m as f64 - nbg_px).abs() / nbg_px < 2e-2);
        assert!((v as f64 - nbg_px).abs() / nbg_px < 2e-2);
    }
    #[test]
    fn imaging_pointing() {
        let pupil_size = 25.5f64;
        let n_side_lenslet = 1;
        let n_px_lenslet = 511;
        let pupil_sampling = n_side_lenslet * n_px_lenslet + 1;
        //        let lenslet_size = (pupil_size / n_side_lenslet as f64) as f32;
        let mut gmt = ceo!(GmtBuilder);
        let mut src = Source::new(1, pupil_size, pupil_sampling);
        src.build("V", vec![0f32], vec![0f32], vec![18f32]);
        let mut sensor = Imaging::new();
        sensor.build(1, n_side_lenslet, n_px_lenslet, 2, 128, 1);
        let p = src.wavelength() / pupil_size / 2f64;
        println!("Pixel scale: {:.3e}mas", p.to_mas());

        let mut cog0 = Centroiding::new();
        cog0.build(n_side_lenslet as u32, None);
        src.through(&mut gmt).xpupil().through(&mut sensor);
        cog0.process(&sensor, None);
        println!("s: {:?}", cog0.grab().centroids);

        let mut frame = vec![0f32; sensor.resolution().pow(2) as usize];
        sensor.frame_transfer(&mut frame);
        let f0 = frame.iter().sum::<f32>();

        let mut cog = Centroiding::new();
        cog.build(n_side_lenslet as u32, Some(p));

        let z = p as f32 * 10.0;
        sensor.pointing(vec![z], vec![0.0], p);
        src.through(&mut gmt).xpupil().through(sensor.reset());
        sensor.frame_transfer(&mut frame);
        let f = frame.iter().sum::<f32>();
        println!("Flux ratio: {}", f / f0);

        cog.process(&sensor, Some(&cog0));
        let s = &cog.grab().centroids;
        println!("s: [{},{}]", (s[0] as f64).to_mas(), (s[1] as f64).to_mas());
        assert!(((z + s[0]) as f64).to_mas() < 1f64);
    }

    #[test]
    fn imaging_exposure() {
        let pupil_size = 25.5f64;
        let n_side_lenslet = 1;
        let n_px_lenslet = 511;
        let pupil_sampling = n_side_lenslet * n_px_lenslet + 1;
        //        let lenslet_size = (pupil_size / n_side_lenslet as f64) as f32;
        let mut gmt = ceo!(GmtBuilder);
        let mut src = Source::new(1, pupil_size, pupil_sampling);
        src.build("V", vec![0f32], vec![0f32], vec![18f32]);
        let mut sensor = Imaging::new();
        sensor.build(1, n_side_lenslet, n_px_lenslet, 2, 128, 1);

        let mut frame = vec![0f32; sensor.resolution().pow(2) as usize];
        src.through(&mut gmt).xpupil().through(&mut sensor);
        sensor.frame_transfer(&mut frame);
        let f0 = frame.iter().sum::<f32>();

        src.through(&mut gmt).xpupil().through(&mut sensor);
        sensor.frame_transfer(&mut frame);
        let f = frame.iter().sum::<f32>();
        println!("Flux ratio: {}", f / f0);
        assert_eq!((f / f0) as usize, 2);

        src.through(&mut gmt).xpupil().through(sensor.reset());
        sensor.frame_transfer(&mut frame);
        let f = frame.iter().sum::<f32>();
        println!("Flux ratio: {}", f / f0);
        assert_eq!((f / f0) as usize, 1);

        sensor.reset();
        for _ in 0..10 {
            src.through(&mut gmt).xpupil().through(&mut sensor);
        }
        sensor.frame_transfer(&mut frame);
        let f = frame.iter().sum::<f32>();
        println!("Flux ratio: {}", (f / f0) as usize);
        assert_eq!((f / f0) as usize, 10);
    }
}
 */
