use std::time::Instant;

use crseo::{
    wavefrontsensor::{PhaseSensor, SegmentCalibration, Stroke},
    Atmosphere, Builder, FromBuilder, Gmt, SegmentWiseSensor, SegmentWiseSensorBuilder,
    WavefrontSensor, WavefrontSensorBuilder,
};

fn main() -> anyhow::Result<()> {
    let n_lenslet = 92;
    let n_mode = 250;

    let builder = PhaseSensor::builder().lenslet(n_lenslet, 8);

    let now = Instant::now();
    let stroke0 = 25e-9;
    let mut slopes_mat = builder.calibrate(
        SegmentCalibration::modes(
            "Karhunen-Loeve",
            0..n_mode,
            "M2",
            Stroke::RadialOrder(stroke0),
        ),
        builder.guide_stars(None),
    );
    println!(
        "M2 {}modes/segment calibrated in {}s",
        n_mode,
        now.elapsed().as_secs()
    );
    slopes_mat.pseudo_inverse(None).unwrap();

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

/*
M2 250modes/segment calibrated in 101s
#000: WFE RMS [nm]: [1221] [ 725, 1079,  943, 1134, 1092, 1040, 1018]
#001: WFE RMS [nm]: [ 617] [ 366,  543,  472,  577,  557,  521,  529]
#002: WFE RMS [nm]: [ 321] [ 202,  284,  248,  303,  294,  271,  293]
#003: WFE RMS [nm]: [ 185] [ 135,  166,  149,  175,  171,  160,  189]
#004: WFE RMS [nm]: [ 130] [ 113,  121,  114,  123,  122,  118,  148]
#005: WFE RMS [nm]: [ 111] [ 107,  107,  104,  105,  106,  106,  134]
#006: WFE RMS [nm]: [ 106] [ 105,  103,  101,  100,  101,  102,  129]
#007: WFE RMS [nm]: [ 105] [ 105,  102,  101,   99,  100,  102,  128]
#008: WFE RMS [nm]: [ 104] [ 105,  102,  101,   98,   99,  102,  127]
#009: WFE RMS [nm]: [ 104] [ 104,  102,  101,   98,   99,  102,  127]
 */
