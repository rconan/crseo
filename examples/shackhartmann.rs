use complot::{Plot, Scatter};
use crseo::{
    ceo, imaging::NoiseDataSheet, shackhartmann::WavefrontSensor, Builder, Diffractive, Geometric,
    ATMOSPHERE, SHACKHARTMANN,
};
use indicatif::{ProgressBar, ProgressIterator, ProgressStyle};
use skyangle::Conversion;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let n_lenslet = 48;
    let n_px_lenslet = 16;
    let pupil_sampling = n_lenslet * n_px_lenslet + 1;
    let n_px_framelet = 8;

    let mut gmt = ceo!(GMT);
    let mut src = ceo!(
        SOURCE,
        pupil_sampling = [pupil_sampling],
        magnitude = [vec![0f32]]
    );
    src.fwhm(6.);
    let now = Instant::now();
    println!("Precomputing atmospheric phase screens ...");
    let mut atm = ATMOSPHERE::new()
        .ray_tracing(26., 520, 0., 1., None, None)
        .build()?;
    let eta = now.elapsed();
    println!("... done in {}ms", eta.as_millis());

    let pitch = src.pupil_size / n_lenslet as f64;
    let mut geom_wfs = SHACKHARTMANN::<Geometric>::new()
        .lenslet_array(n_lenslet, n_px_lenslet, pitch)
        .build()?;
    let mut diff_wfs = SHACKHARTMANN::<Diffractive>::new()
        .lenslet_array(n_lenslet, n_px_lenslet, pitch)
        .detector(
            n_px_framelet,
            Some(24),
            Some(2),
            Some(NoiseDataSheet::new(1e0)),
        )
        .build()?;

    src.through(&mut gmt).xpupil();
    println!("GS WFE RMS: {}nm", src.wfe_rms_10e(-9)[0]);

    diff_wfs.calibrate(&mut src, 0.5);
    println!("# valid lenslet: {}", diff_wfs.n_valid_lenslet());

    //    geom_wfs.calibrate(&mut src, 0.5);
    geom_wfs.valid_lenslet_from(&mut diff_wfs);
    geom_wfs.set_reference_slopes(&mut src);
    println!("# valid lenslet: {}", geom_wfs.n_valid_lenslet());

    gmt.m2_segment_state(2, &[0., 0.0, 0.], &[1e-6, 0.0, 0.]);
    gmt.m2_segment_state(5, &[0., 0.0, 0.], &[0., 1e-6, 0.]);
    gmt.m2_segment_state(7, &[0., 0.0, 0.], &[1e-6, 1e-6, 0.]);

    geom_wfs.reset();
    src.through(&mut gmt).xpupil().through(&mut geom_wfs);
    geom_wfs.process();
    let geom_centroids: Vec<f32> = (&mut geom_wfs.centroids).into();

    diff_wfs.reset();
    src.through(&mut gmt).xpupil().through(&mut diff_wfs);
    diff_wfs.readout();
    diff_wfs.process();
    let diff_centroids: Vec<f32> = (&mut diff_wfs.centroids).into();
    let frame = diff_wfs.frame();
    println!(
        "Frame: {:?}/{}",
        diff_wfs.detector_resolution(),
        frame.len()
    );

    geom_centroids
        .iter()
        .zip(diff_centroids.iter())
        .enumerate()
        .map(|(k, (&g, &d))| (k as f64, vec![g.to_mas() as f64, d.to_mas() as f64]))
        .collect::<Plot>();
    let _: complot::Heatmap = ((frame.as_slice(), diff_wfs.detector_resolution()), None).into();

    Vec::<f32>::from(geom_wfs.get_data())
        .iter()
        .zip(Vec::<f32>::from(diff_wfs.get_data()).iter())
        .map(|(&g, &d)| (g.to_mas() as f64, vec![d.to_mas() as f64]))
        .collect::<Scatter>();

    src.fwhm(0.);
    gmt.reset();
    diff_wfs.reset();
    let dt = 1e-3;
    let n_step = 1000;
    let pb = ProgressBar::new(n_step);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:60.cyan/blue} {pos:>7}/{len:7}")
            .progress_chars("=|-"),
    );
    for k in (0..n_step).progress_with(pb) {
        atm.secs = k as f64 * dt;
        src.through(&mut gmt)
            .xpupil()
            .through(&mut atm)
            .through(&mut diff_wfs);
    }

    let frame = diff_wfs.frame();
    let filename = "wfs+atm.png";
    let _: complot::Heatmap = (
        (frame.as_slice(), diff_wfs.detector_resolution()),
        Some(complot::Config::new().filename(filename)),
    )
        .into();
    Ok(())
}
