use active_optics::Calib;
use complot::{Config, Heatmap};
use crseo::{Builder, FromBuilder, Gmt, Source};
use skyangle::Conversion;
use std::{env, f32::consts::PI, sync::LazyLock};

const SID: u8 = 1;

static M2_N_MODE: LazyLock<usize> = LazyLock::new(|| {
    env::var("M2_N_MODE")
        .map(|x| x.parse().unwrap())
        .unwrap_or(66)
});

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut gmt = Gmt::builder().m2_n_mode(*M2_N_MODE).build()?;
    let mut src = Source::builder()
        .size(2)
        .zenith_azimuth(vec![0., 6f32.from_arcmin()], vec![0., PI / 2.])
        .build()?;

    src.through(&mut gmt).xpupil();
    let phase0 = src.phase().clone();
    let amplitude0 = src.amplitude();

    gmt.m1_segment_state(1, &[0., -20e-6, 0.], &[0f64.from_mas(), 0., 0.]);
    src.through(&mut gmt).xpupil();

    let n = src.pupil_sampling();
    let phase: Vec<_> = src
        .phase()
        .iter()
        .zip(&phase0)
        .map(|(x, y)| (x - y))
        .collect();
    dbg!(phase.len());
    let _: Heatmap = (
        (&phase[..n * n], (n, n)),
        Some(Config::new().filename("on-axis_wavefront.png")),
    )
        .into();
    let _: Heatmap = (
        (&phase[n * n..], (n, n)),
        Some(Config::new().filename("off-axis_wavefront.png")),
    )
        .into();

    let calib = Calib::<SID>::load("calib_m2.pkl")?;
    let pinv = calib.pseudoinverse();

    let dphase: Vec<f64> = phase.iter().take(n * n).map(|x| -*x as f64).collect();
    let a = &pinv * calib.apply_mask(dphase.as_slice());
    a.iter().enumerate().for_each(|(i, x)| {
        if x.abs() > 1e-9 {
            println!("{:3}: {:8.1}", i + 1, x * 1e9)
        }
    });

    gmt.m2_segment_modes(SID, a.as_slice());
    src.through(&mut gmt).xpupil();
    let mask = amplitude0
        .into_iter()
        .zip(src.amplitude().into_iter())
        .map(|(x, y)| x * y > 0.);
    let phase: Vec<_> = src
        .phase()
        .iter()
        .zip(&phase0)
        .zip(mask)
        .map(|((x, y), m)| if m { (x - y) * 1e9 } else { 0. })
        .collect();

    let _: Heatmap = (
        (&phase[..n * n], (n, n)),
        Some(Config::new().filename("on-axis_m2-corrected_wavefront.png")),
    )
        .into();
    let _: Heatmap = (
        (&phase[n * n..], (n, n)),
        Some(Config::new().filename("off-axis_m2-corrected_wavefront.png")),
    )
        .into();

    Ok(())
}
