pub mod builder;
use crate::builders::SourceBuilder;
pub use builder::GeomShackBuilder;
mod geom_shack;
pub use geom_shack::GeomShack;

pub use super::data_processing;
use crate::{SegmentWiseSensorBuilder, WavefrontSensor, WavefrontSensorBuilder};

impl WavefrontSensorBuilder for GeomShackBuilder {
    fn guide_stars(&self, gs: Option<SourceBuilder>) -> SourceBuilder {
        gs.unwrap_or_default()
            .size(self.n_gs as usize)
            .pupil_sampling(self.pupil_sampling())
    }
}

impl WavefrontSensor for GeomShack {
    fn calibrate(&mut self, _src: &mut crate::Source, _threshold: f64) {
        todo!()
    }

    fn reset(&mut self) {
        unsafe {
            self._c_.reset();
        }
    }

    fn process(&mut self) {
        todo!()
    }

    fn readout(&mut self) {
        todo!()
    }

    fn data(&mut self) -> Vec<f64> {
        GeomShack::data(self)
            .into_iter()
            .map(|x| x as f64)
            .collect()
    }

    fn frame(&self) -> Option<Vec<f32>> {
        todo!()
    }

    fn n_frame(&self) -> usize {
        todo!()
    }

    fn valid_lenslet_from(&mut self, _wfs: &mut dyn WavefrontSensor) {
        todo!()
    }

    fn valid_lenslet(&mut self) -> &mut ffi::mask {
        todo!()
    }

    fn n_valid_lenslet(&mut self) -> Vec<usize> {
        todo!()
    }

    fn left_multiply(&self, calibration: &super::Calibration) -> Option<Vec<f32>> {
        calibration * self
    }
}

/* #[cfg(test)]
mod tests {
    use std::fs::File;

    use super::*;
    use crate::{
        wavefrontsensor::{Mirror, SegmentCalibration, DOF},
        Builder, FromBuilder, Gmt, SegmentWiseSensor, Source,
    };

    #[test]
    fn geom_shack() {
        let n_side_lenslet = 50;
        let mut gmt = Gmt::builder().build().unwrap();
        let mut wfs = GeomShack::builder()
            .lenslet(n_side_lenslet, 16)
            .build()
            .unwrap();
        let mut src = Source::builder()
            .pupil_sampling(wfs.pupil_sampling())
            .build()
            .unwrap();
        src.through(&mut gmt).xpupil().through(&mut wfs);

        let _: complot::Heatmap = (
            (
                src.phase().as_slice(),
                (wfs.pupil_sampling(), wfs.pupil_sampling()),
            ),
            Some(complot::Config::new().filename("phase.png")),
        )
            .into();

        let data = wfs.data();
        dbg!(data.len());
        serde_pickle::to_writer(
            &mut File::create("geom_shack_data.pkl").unwrap(),
            &data,
            Default::default(),
        )
        .unwrap();

        let calib = wfs.calibrate_segment(None, 1, 15, None);
        println!("{calib}");
        serde_pickle::to_writer(
            &mut File::create("geom_shack_calibration.pkl").unwrap(),
            &calib,
            Default::default(),
        )
        .unwrap();

        let sc = SegmentCalibration::Modes {
            name: "Karhunen-Loeve".to_string(),
            dof: DOF::Range(1..15),
            mirror: Mirror::M2,
            keep: true,
        };

        let calib2 = sc.calibrate(1, &mut wfs, Source::builder(), None);
        println!("{calib2}");
        serde_pickle::to_writer(
            &mut File::create("geom_shack_calibration2.pkl").unwrap(),
            &calib,
            Default::default(),
        )
        .unwrap();
        // assert_eq!(calib, calib2);
    }

    #[test]
    fn calibrate_rbm() {
        let builder = GeomShack::builder().size(2).lenslet(48, 8);
        let src_builder = builder.guide_stars(Some(Source::builder()));
        let mut wfs = builder.build().unwrap();
        let seg = SegmentCalibration::rbm("TRxyz", "M1");
        let mut c = seg.calibrate(1, &mut wfs, src_builder, None);
        println!("{}", c.interaction_matrix());
        c.pseudo_inverse(None).unwrap();
    }
}
 */
