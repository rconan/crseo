mod builder;
pub use builder::SegmentPistonSensorBuilder;
pub mod processing;

use crate::{cu::Single, imaging::Frame, Cu, FromBuilder, Propagation};

pub struct SegmentPistonSensor {
    _c_: ffi::segmentPistonSensor,
    malloc_dft: bool,
    middle_mask_width: Option<f64>,
}

impl FromBuilder for SegmentPistonSensor {
    type ComponentBuilder = SegmentPistonSensorBuilder;
}

impl Propagation for SegmentPistonSensor {
    fn propagate(&mut self, src: &mut crate::Source) {
        let src_ptr = src.as_raw_mut_ptr();
        unsafe {
            if self.malloc_dft {
                if let Some(middle_mask_width) = self.middle_mask_width {
                    self._c_.propagate1(src_ptr, middle_mask_width as f32)
                } else {
                    self._c_.propagate(src_ptr)
                }
            } else {
                self._c_.propagate_alt(src_ptr)
            }
        }
    }

    fn time_propagate(&mut self, _secs: f64, _src: &mut crate::Source) {
        todo!()
    }
}

impl SegmentPistonSensor {
    pub fn n_camera_frame(&self) -> usize {
        self._c_.camera.N_FRAME as usize
    }
    pub fn n_fft_frame(&self) -> usize {
        self._c_.FFT.N_FRAME as usize
    }
    pub fn fft(&mut self) -> &mut Self {
        unsafe {
            self._c_.fft();
        }
        self
    }
    pub fn fft_frame(&mut self) -> Frame {
        let n = self._c_.FFT.N_SIDE_LENSLET * self._c_.FFT.N_PX_CAMERA;
        let mut cu = Cu::<Single>::vector((n.pow(2)) as usize);
        cu.from_ptr(self._c_.FFT.d__frame);
        Frame {
            dev: cu,
            n_px_camera: self._c_.FFT.N_PX_CAMERA as usize,
            resolution: n as usize,
            n_frame: 1,
        }
    }
    pub fn frame(&self) -> Frame {
        let resolution = self._c_.camera.N_PX_CAMERA * self._c_.camera.N_SIDE_LENSLET;
        let mut cu = Cu::<Single>::vector((resolution.pow(2)) as usize);
        cu.from_ptr(self._c_.camera.d__frame);
        Frame {
            dev: cu,
            n_px_camera: self._c_.camera.N_PX_CAMERA as usize,
            resolution: resolution as usize,
            n_frame: 1,
        }
    }
    pub fn reset(&mut self) -> &mut Self {
        unsafe {
            self._c_.camera.reset();
            self._c_.FFT.reset();
        }
        self
    }
    pub fn camera_reset(&mut self) -> &mut Self {
        unsafe {
            self._c_.camera.reset();
        }
        self
    }
    pub fn fft_reset(&mut self) -> &mut Self {
        unsafe {
            self._c_.FFT.reset();
        }
        self
    }
    pub fn frame_size(&self) -> usize {
        (self._c_.camera.N_PX_CAMERA * self._c_.camera.N_SIDE_LENSLET) as usize
    }
    pub fn fft_size(&self) -> usize {
        (self._c_.FFT.N_SIDE_LENSLET * self._c_.FFT.N_PX_CAMERA) as usize
    }
    pub fn n_source(&self) -> usize {
        self._c_.N_GS as usize
    }
}

impl Drop for SegmentPistonSensor {
    fn drop(&mut self) {
        unsafe {
            if self.malloc_dft {
                self._c_.cleanup();
            } else {
                self._c_.cleanup_alt();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{error::Error, fs::File, io::BufWriter};

    use complot::{Config, Heatmap};

    use crate::{gmt::MirrorGetSet, Builder, FromBuilder, Gmt, Source};

    use super::SegmentPistonSensor;

    #[test]
    fn sps0() {
        let mut gmt = Gmt::builder().build().unwrap();
        let mut src = Source::builder().build().unwrap();

        let mut sps = SegmentPistonSensor::builder()
            .nyquist_factor(3.)
            .build()
            .unwrap();

        src.through(&mut gmt).xpupil().through(&mut sps);

        let mut frame = sps.frame();
        dbg!(frame.n_px_camera);
        let hframe: Vec<f32> = (&mut frame).into();
        dbg!(hframe.len());

        dbg!(hframe.iter().sum::<f32>());

        let mut fft = sps.fft().fft_frame();

        let _: Heatmap = (
            (src.phase().as_slice(), (512, 512)),
            Some(Config::new().filename("phase.png")),
        )
            .into();

        let _: Heatmap = (
            (hframe.as_slice(), dbg!(frame.roi())),
            Some(Config::new().filename("sps-frame.png")),
        )
            .into();

        let _: Heatmap = (
            (
                Vec::<f32>::from(&mut fft)
                    .iter()
                    .map(|x| x.cbrt())
                    .collect::<Vec<_>>()
                    .as_slice(),
                dbg!(fft.roi()),
            ),
            Some(Config::new().filename("sps-fft.png")),
        )
            .into();
    }

    #[test]
    fn sps_tz() -> Result<(), Box<dyn Error>> {
        let mut gmt = Gmt::builder().build().unwrap();
        let mut src = Source::builder().build().unwrap();

        let mut sps = SegmentPistonSensor::builder()
            .nyquist_factor(3.)
            .build()
            .unwrap();

        let mut tr_xyz = [0f64; 6];
        let p = 8usize;
        tr_xyz[2] = p as f64 * 1e-6;
        gmt.m1.set_rigid_body_motions(1, &tr_xyz);

        src.through(&mut gmt).xpupil().through(&mut sps);

        let mut frame = sps.frame();
        let hframe: Vec<f32> = (&mut frame).into();

        let mut buffer = BufWriter::new(File::create(format!("sps-frame-{p}microntz.pkl"))?);
        serde_pickle::to_writer(&mut buffer, &hframe, Default::default())?;

        // let mut fft = sps.fft();

        // let _: Heatmap = (
        //     (src.phase().as_slice(), (512, 512)),
        //     Some(Config::new().filename("phase-tz.png")),
        // )
        //     .into();

        // let _: Heatmap = (
        //     (hframe.as_slice(), dbg!(frame.roi())),
        //     Some(Config::new().filename("sps-frame-2microntz.png")),
        // )
        //     .into();

        // let _: Heatmap = (
        //     (
        //         Vec::<f32>::from(&mut fft)
        //             .iter()
        //             .map(|x| x.cbrt())
        //             .collect::<Vec<_>>()
        //             .as_slice(),
        //         dbg!(fft.roi()),
        //     ),
        //     Some(Config::new().filename("sps-fft-2microntz.png")),
        // )
        //     .into();
        Ok(())
    }

    #[test]
    fn sps_ty() {
        let mut gmt = Gmt::builder().build().unwrap();
        let mut src = Source::builder().build().unwrap();

        let mut sps = SegmentPistonSensor::builder()
            .nyquist_factor(3.)
            .build()
            .unwrap();

        let mut tr_xyz = [0f64; 6];
        tr_xyz[1] = 4e-6;
        gmt.m1.set_rigid_body_motions(1, &tr_xyz);

        src.through(&mut gmt).xpupil().through(&mut sps);

        let mut frame = sps.frame();
        let hframe: Vec<f32> = (&mut frame).into();

        let mut fft = sps.fft().fft_frame();

        let _: Heatmap = (
            (src.phase().as_slice(), (512, 512)),
            Some(Config::new().filename("phase-ty.png")),
        )
            .into();

        let _: Heatmap = (
            (hframe.as_slice(), dbg!(frame.roi())),
            Some(Config::new().filename("sps-frame-ty.png")),
        )
            .into();

        let _: Heatmap = (
            (
                Vec::<f32>::from(&mut fft)
                    .iter()
                    .map(|x| x.cbrt())
                    .collect::<Vec<_>>()
                    .as_slice(),
                dbg!(fft.roi()),
            ),
            Some(Config::new().filename("sps-fft-ty.png")),
        )
            .into();
    }

    #[test]
    fn sps_tx() {
        let mut gmt = Gmt::builder().build().unwrap();
        let mut src = Source::builder().build().unwrap();

        let mut sps = SegmentPistonSensor::builder()
            .nyquist_factor(3.)
            .build()
            .unwrap();

        let mut tr_xyz = [0f64; 6];
        tr_xyz[0] = 4e-6;
        gmt.m1.set_rigid_body_motions(1, &tr_xyz);

        src.through(&mut gmt).xpupil().through(&mut sps);

        let mut frame = sps.frame();
        let hframe: Vec<f32> = (&mut frame).into();

        let mut fft = sps.fft().fft_frame();

        let _: Heatmap = (
            (src.phase().as_slice(), (512, 512)),
            Some(Config::new().filename("phase-tx.png")),
        )
            .into();

        let _: Heatmap = (
            (hframe.as_slice(), dbg!(frame.roi())),
            Some(Config::new().filename("sps-frame-tx.png")),
        )
            .into();

        let _: Heatmap = (
            (
                Vec::<f32>::from(&mut fft)
                    .iter()
                    .map(|x| x.cbrt())
                    .collect::<Vec<_>>()
                    .as_slice(),
                dbg!(fft.roi()),
            ),
            Some(Config::new().filename("sps-fft-tx.png")),
        )
            .into();
    }
}
