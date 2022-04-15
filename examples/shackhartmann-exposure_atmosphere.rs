//use complot::{Plot, Scatter};
use crseo::{
    ceo, imaging::NoiseDataSheet, shackhartmann::WavefrontSensor, Builder, Diffractive, ATMOSPHERE,
    ShackHartmannBuilder,
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
        .ray_tracing(26., 520, 0., 30., None, None)
        .build()?;
    let eta = now.elapsed();
    println!("... done in {}ms", eta.as_millis());

    let pitch = src.pupil_size / n_lenslet as f64;
    let mut diff_wfs = ShackHartmannBuilder::<Diffractive>::new()
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

    src.fwhm(0.);
    gmt.reset();
    let dt = 1e-3;
    for exposure in [1f64, 10f64, 30f64] {
        let n_step = (exposure / dt) as u64;
        diff_wfs.reset();
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
        diff_wfs.process();
        let centroids_error: f64 = Vec::<f32>::from(diff_wfs.get_data())
            .iter()
            .map(|&c| c.to_mas() as f64)
            .map(|c| c * c)
            .sum::<f64>()
            / diff_wfs.n_valid_lenslet() as f64;
        println!(
            "Exposure: {:2}s: {:7.3}mas",
            exposure,
            centroids_error.sqrt()
        );
    }
    /*
    let frame = diff_wfs.frame();
    let filename = "wfs+atm.png";
    let _: complot::Heatmap = (
        (frame.as_slice(), diff_wfs.detector_resolution()),
        Some(complot::Config::new().filename(filename)),
    )
        .into();*/
    Ok(())
}
