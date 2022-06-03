use crseo::prelude::*;
use std::time::Instant;

#[test]
fn sh24_calibration() -> anyhow::Result<()> {
    let mut src = Source::builder().build()?;
    let mut gmt = Gmt::builder().build()?;
    let mut wfs = SH24::<Geometric>::new().build()?;

    src.through(&mut gmt).xpupil();
    wfs.calibrate(&mut src, 0.5);

    let mut gmt2wfs = Calibration::new(&gmt, &src, SH24::<Geometric>::new());
    let specs = vec![
        Some(vec![(
            calibrations::Mirror::M2,
            vec![calibrations::Segment::Rxyz(1e-6, Some(0..2))]
        )]);
        7
    ];
    let now = Instant::now();
    gmt2wfs.calibrate(
        specs,
        calibrations::ValidLensletCriteria::OtherSensor(&mut wfs),
    );
    println!(
        "GMT 2 WFS calibration [{}x{}] in {}s",
        gmt2wfs.n_data,
        gmt2wfs.n_mode,
        now.elapsed().as_secs()
    );
    Ok(())
}
