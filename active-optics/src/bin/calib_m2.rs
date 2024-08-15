use active_optics::{Calib, M2_N_MODE, SID};
use crseo::{Builder, FromBuilder, Gmt, Source};
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut gmt = Gmt::builder().m2_n_mode(M2_N_MODE).build()?;
    gmt.keep(&[SID as i32]);
    let mut src = Source::builder().build()?;

    src.through(&mut gmt).xpupil();
    // let phase0 = src.phase().clone();
    let amplitude0 = src.amplitude();
    let mask: Vec<_> = amplitude0.iter().map(|x| *x > 0.).collect();
    let area0 = mask.iter().filter(|&&x| x).count();
    dbg!(area0);

    let stroke = 1e-6;
    let mut a = vec![0f64; M2_N_MODE];
    let mut calib: Vec<f64> = Vec::new();
    let now = Instant::now();
    for i in 0..M2_N_MODE {
        a[i] = stroke;
        gmt.m2_segment_modes(SID, &a);
        src.through(&mut gmt).xpupil();
        let area = src.amplitude().iter().filter(|&&x| x > 0.).count();
        if area != area0 {
            panic!("Expected area={}, found {}", area0, area);
        }
        let push = src.phase().clone();

        a[i] *= -1.;
        gmt.m2_segment_modes(SID, &a);
        src.through(&mut gmt).xpupil();
        let area = src.amplitude().iter().filter(|&&x| x > 0.).count();
        if area != area0 {
            panic!("Expected area={}, found {}", area0, area);
        }

        let pushpull = push
            .iter()
            .zip(src.phase().iter())
            .zip(&mask)
            .filter(|&(_, &m)| m)
            .map(|((x, y), _)| 0.5 * (x - y) as f64 / stroke);
        calib.extend(pushpull);

        a[i] = 0.;
    }
    println!("Elapsed: {:?}", now.elapsed());

    dbg!(calib.len());

    let calib = Calib::<SID>::new(M2_N_MODE, calib, mask);
    calib.dump("calib_m2.pkl")?;

    Ok(())
}
