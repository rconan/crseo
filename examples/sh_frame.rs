use complot as plt;
use crseo::{
    ceo, imaging::NoiseDataSheet, shackhartmann::WavefrontSensorBuilder, Builder, Diffractive, SH48,
};
use indicatif::{ProgressBar, ProgressStyle};
use skyangle::SkyAngle;

fn main() {
    let mut gmt = ceo!(GMT);
    //let mut src = ceo!(SOURCE);
    //    let mut wfs = ceo!(SHACKHARTMANN: Diffractive);
    //    let mut wfs = ceo!(SH48: Diffractive, n_sensor = [1]);
    //    let mut src = wfs.new_guide_stars();
    let (mut wfs, mut src) = {
        let wfs_blueprint = SH48::<Diffractive>::new()
            .n_sensor(3)
            .detector_noise_specs(NoiseDataSheet::new(1e-3).read_out(1.));
        let src_blueprint = wfs_blueprint
            .guide_stars(None)
            .on_ring(SkyAngle::Arcminute(8_f32).to_radians())
            .magnitude(vec![14.; 3]);
        (
            wfs_blueprint.build().unwrap(),
            src_blueprint.build().unwrap(),
        )
    };
    let mut atm = ceo!(
        ATMOSPHERE,
        ray_tracing = [
            26.,
            401,
            SkyAngle::Arcminute(20f32).to_radians(),
            3.,
            None,
            None
        ]
    );
    //    src.fwhm(5.);

    let res = wfs.detector_resolution();
    let frame = wfs.frame();
    println!("WFS resolution: {:?}; frame: {}", res, frame.len());

    let n_sample = 1_000;
    let pb = ProgressBar::new(n_sample);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7}")
            .progress_chars("##-"),
    );
    for k in 0..n_sample {
        pb.inc(1);
        atm.secs = k as f64 * 1e-3;
        src.through(&mut gmt)
            .xpupil()
            .through(&mut atm)
            .through(&mut wfs);
        //wfs.readout();
    }
    pb.finish();

    for (i, frame) in wfs.frame().chunks(res.0 * res.1).enumerate() {
        //println!("#{}: Flux: {}", i + 1, frame.iter().sum::<f32>());
        let filename = format!("examples/wfs_frame_{}.png", i + 1);
        let _: plt::Heatmap = ((frame, res), Some(plt::Config::new().filename(filename))).into();
    }
}
