use std::time::Instant;

use crseo::{
    wavefrontsensor::{PhaseSensor, SegmentCalibration},
    Atmosphere, Builder, FromBuilder, Gmt, SegmentWiseSensor, SegmentWiseSensorBuilder,
    WavefrontSensor, WavefrontSensorBuilder,
};

fn main() -> anyhow::Result<()> {
    let n_lenslet = 92;
    let n_mode = 250;

    let builder = PhaseSensor::builder().lenslet(n_lenslet, 16);

    let now = Instant::now();
    let mut slopes_mat = builder.calibrate(
        SegmentCalibration::modes("Karhunen-Loeve", 0..n_mode, "M2"),
        builder.guide_stars(None),
    );
    println!(
        "M2 {}modes/segment calibrated in {}s",
        n_mode,
        now.elapsed().as_secs()
    );
    slopes_mat.pseudo_inverse().unwrap();

    let mut gmt = Gmt::builder().m2("Karhunen-Loeve", n_mode).build().unwrap();
    let mut src = builder.guide_stars(None).build().unwrap();
    let mut wfs = builder.build().unwrap();

    let mut atm = Atmosphere::builder().build()?;

    let mut buffer = vec![0f64; 7 * n_mode];
    let gain = 0.5;

    for i in 0..10 {
        src.through(&mut gmt)
            .xpupil()
            .through(&mut atm)
            .through(&mut wfs);
        println!(
            "#{:03}: WFE RMS [nm]: {:4.0?} {:4.0?}",
            i,
            src.wfe_rms_10e(-9),
            src.segment_wfe_rms_10e(-9)
        );

        let coefs = (&slopes_mat * &wfs).unwrap();
        // dbg!(coefs.len());
        wfs.reset();

        buffer
            .iter_mut()
            .zip(&coefs)
            .for_each(|(b, c)| *b -= gain * *c as f64);
        // dbg!(&buffer);
        gmt.m2_modes(&buffer);
    }

    let _: complot::Heatmap = (
        (
            src.phase().as_slice(),
            (wfs.pupil_sampling(), wfs.pupil_sampling()),
        ),
        Some(complot::Config::new().filename("opd.png")),
    )
        .into();

    Ok(())
}
