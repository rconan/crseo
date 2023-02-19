mod builder;
pub use builder::GeomShackBuilder;
mod geom_shack;
pub use geom_shack::GeomShack;

#[cfg(test)]
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
}
