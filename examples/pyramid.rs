use crseo::{wavefrontsensor::Pyramid, Atmosphere, Builder, FromBuilder, Gmt, Source};

fn main() -> anyhow::Result<()> {
    let n_lenslet = 90;
    let n_mode = 150;

    let builder = Pyramid::builder().n_lenslet(n_lenslet).modulation(8., 101);
    let mut pym = builder.clone().build().unwrap();

    let mut slopes_mat = pym.calibrate(n_mode);
    slopes_mat.pseudo_inverse().unwrap();

    let mut gmt = Gmt::builder().m2("Karhunen-Loeve", n_mode).build().unwrap();
    let mut src = Source::builder()
        .pupil_sampling(pym.pupil_sampling())
        .build()
        .unwrap();
    src.rotate_rays(0.5 * std::f64::consts::FRAC_PI_6);

    let mut atm = Atmosphere::builder().build()?;

    let mut buffer = vec![0f64; 7 * n_mode];
    let gain = 0.5;

    for i in 0..25 {
        pym.reset();
        src.through(&mut gmt)
            .xpupil()
            .through(&mut atm)
            .through(&mut pym);
        println!("#{:03}: WFE RMS: {:4.0?}nm", i, src.wfe_rms_10e(-9));

        let coefs = (&slopes_mat * &pym).unwrap();
        // dbg!(&coefs);

        buffer
            .chunks_mut(n_mode)
            .zip(coefs.chunks(n_mode - 1))
            .for_each(|(b, c)| {
                b.iter_mut()
                    .skip(1)
                    .zip(c)
                    .for_each(|(b, c)| *b -= gain * *c as f64)
            });
        // dbg!(&buffer);
        gmt.m2_modes(&buffer);
    }

    Ok(())
}
