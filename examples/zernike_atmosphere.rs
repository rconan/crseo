use crseo::{Atmosphere, Builder, FromBuilder, Source};
use matio_rs::{MatFile, Save, Set};

fn main() -> anyhow::Result<()> {
    let mut atm = Atmosphere::builder()
        .single_turbulence_layer(0f32, Some(7.), Some(0.))
        .build()?;
    let n_xy = 101;
    let side = 2f64;
    let mut src = Source::builder()
        .pupil_size(side)
        .pupil_sampling(n_xy)
        .build()?;
    let coefficients: Vec<_> = (0..1000)
        .flat_map(|i| {
            let t = 0.01f64 * i as f64;
            let ps = atm.get_phase_screen(&mut src, t, (side, n_xy), None);
            zernike::projection(&ps, 11, n_xy)
        })
        .collect();
    MatFile::save("zernike_coefficients")?.var("coefs", &coefficients);
    Ok(())
}
