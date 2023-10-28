use crseo::wavefrontsensor::LensletArray;
use crseo::{
    wavefrontsensor::{Calibration, Pyramid, SegmentCalibration, Stroke},
    Atmosphere, Builder, FromBuilder, Gmt, SegmentWiseSensor, SegmentWiseSensorBuilder,
    WavefrontSensor, WavefrontSensorBuilder,
};

use nanorand::{Rng, WyRand};
use std::fs::File;
use std::time::Instant;

mod pym;
pub use pym::PyramidPiston;

fn main() -> anyhow::Result<()> {
    let n_lenslet = 92;
    let n_side_lenslet = n_lenslet;
    let n_mode = 450;

    let builder = Pyramid::builder()
        .lenslet_array(LensletArray {
            n_side_lenslet: n_lenslet,
            n_px_lenslet: 10,
            d: 0f64,
        })
        .modulation(2., 64);

    let now = Instant::now();
    let stroke0 = 25e-9;
    let mut slopes_mat = builder.calibrate(
        SegmentCalibration::modes(
            "ASM_DDKLs_S7OC04184_675kls",
            1..n_mode,
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
    // println!("{slopes_mat}");

    serde_pickle::to_writer(
        &mut File::create(format!("slopes_mat-{n_mode}.pkl"))?,
        &slopes_mat,
        Default::default(),
    )?;

    // let mut slopes_mat: Calibration = serde_pickle::from_reader(File::open("slopes_mat-250.pkl")?)?; //, Default::default())?;
    slopes_mat.pseudo_inverse(None).unwrap();

    let pymp = PyramidPiston::new(builder, slopes_mat.masks())?;

    serde_pickle::to_writer(
        &mut File::create("piston_mask.pkl")?,
        &pymp.mask,
        Default::default(),
    )?;

    /*
    let m: Vec<_> = slopes_mat
        .mask()
        .into_iter()
        .filter_map(|mask| mask.map(|m| m.as_slice().to_vec()))
        .collect();
    serde_pickle::to_writer(
        &mut File::create("poke_mat_masks.pkl")?,
        &m,
        Default::default(),
    )?;
    let sxy0 = slopes_mat.reference_slopes();
    serde_pickle::to_writer(
        &mut File::create("poke_mat_sxy0.pkl")?,
        &sxy0,
        Default::default(),
    )?;

    for (i, mask) in slopes_mat.mask().enumerate() {
        if let Some(mask) = mask {
            let mat = mask.map(|x| if x { 1f64 } else { 0f64 });
            let _: complot::Heatmap = (
                (mat.as_slice(), (n_lenslet, n_lenslet)),
                Some(complot::Config::new().filename(format!("mask#{i}.png"))),
            )
                .into();
        }
    } */

    let mut gmt = Gmt::builder()
        .m2("ASM_DDKLs_S7OC04184_675kls", n_mode)
        .build()
        .unwrap();
    let mut src = builder.guide_stars(None).build().unwrap();
    let mut pym = builder.build().unwrap();

    let mut buffer = vec![0f64; 7 * n_mode];

    /*     dbg!(pym.frame().len());

    src.through(&mut gmt).xpupil().through(&mut pym);

    /*     let mut buffer = vec![0f64; 7 * n_mode];
    buffer[3] = 25e-9;
    gmt.m2_modes(&buffer); */

    let mut m2_segment_coefs = vec![0f64; n_mode];
    gmt.reset();
    m2_segment_coefs[5] = stroke0;
    gmt.m2_segment_modes(1, &m2_segment_coefs);
    m2_segment_coefs[8] = stroke0;
    gmt.m2_segment_modes(3, &m2_segment_coefs);
    m2_segment_coefs[12] = stroke0;
    gmt.m2_segment_modes(7, &m2_segment_coefs);

    pym.reset();
    src.through(&mut gmt).xpupil().through(&mut pym);
    println!("{:?}", src.segment_wfe_rms_10e(-9));

    let n = src.pupil_sampling();
    let _: complot::Heatmap = (
        (src.phase().as_slice(), (n, n)),
        Some(complot::Config::new().filename("opd.png")),
    )
        .into();

    let mut coefs = (&slopes_mat * &pym).unwrap();
    coefs
        .as_mut_slice()
        .chunks_mut(n_mode-1)
        .map(|v| {
            v.iter_mut().for_each(|v| {
                *v = *v / stroke0 as f32;
            });
            v
        })
        .enumerate()
        .for_each(|(i, c)| {
            println!("SID #{}\n{:.3?}", i + 1, c);
        });

    let _: complot::Heatmap = (
        (pym.frame().as_slice(), (n_lenslet * 4, n_lenslet * 4)),
        Some(complot::Config::new().filename("pym.png")),
    )
        .into(); */

    let mut atm = Atmosphere::builder().build()?;

    let gain = 1.;
    let piston_gain = 0.5;

    let mut piston_dist = vec![0f32; 7];
    let mut rng = WyRand::new();
    piston_dist.iter_mut().take(7).for_each(|x| {
        *x = (2. * rng.generate::<f32>() - 1.) * 1500e-9;
    });
    println!(
        "Piston dist: {:4.0?}",
        piston_dist.iter().map(|x| x * 1e9).collect::<Vec<_>>()
    );

    for i in 0..150 {
        pym.reset();
        src.through(&mut gmt)
            .xpupil()
            // .add_piston(&piston_dist)
            .through(&mut atm)
            .through(&mut pym);
        println!(
            "#{:03}: WFE RMS [nm]: {:4.0?} {:4.0?}",
            i,
            src.wfe_rms_10e(-9),
            src.segment_wfe_10e(-9),
        );

        /*         let (sx0, sy0) = pym.data();

        let _: complot::Heatmap = (
            (sx0.as_slice(), (n_side_lenslet, n_side_lenslet)),
            Some(complot::Config::new().filename("pym-sx0.png")),
        )
            .into();
        let _: complot::Heatmap = (
            (sy0.as_slice(), (n_side_lenslet, n_side_lenslet)),
            Some(complot::Config::new().filename("pym-sy0.png")),
        )
            .into(); */

        /*         let slopes: Vec<_> = slopes_mat
            .iter()
            .map(|sm| pym.into_slopes(&sm.data_ref))
            .collect();
        serde_pickle::to_writer(
            &mut File::create("slopes.pkl")?,
            &slopes,
            Default::default(),
        )?; */

        if i == 0 {
            let _: complot::Heatmap = (
                (
                    src.phase().as_slice(),
                    (pym.pupil_sampling(), pym.pupil_sampling()),
                ),
                Some(complot::Config::new().filename("opd.png")),
            )
                .into();
        }
        let coefs = (&slopes_mat * &pym).unwrap();

        buffer
            .chunks_mut(n_mode)
            .zip(coefs.chunks(n_mode - 1))
            .for_each(|(b, c)| {
                b.iter_mut()
                    .skip(1)
                    .zip(c)
                    .for_each(|(b, c)| *b -= gain * *c as f64)
            });
        if i >= 0 {
            let piston = pymp.piston(&mut pym);
            // println!(
            //     "Piston: {:4.0?}",
            //     piston.iter().map(|x| x * 1e9).collect::<Vec<_>>()
            // );
            // dbg!(&piston);
            buffer
                .chunks_mut(n_mode)
                .zip(&piston)
                .for_each(|(b, p)| b[0] -= piston_gain * *p as f64);
        }
        // dbg!(buffer.chunks(n_mode).map(|x| x[0]).collect::<Vec<_>>());
        gmt.m2_modes(&buffer);
    }

    let _: complot::Heatmap = (
        (
            src.phase().as_slice(),
            (pym.pupil_sampling(), pym.pupil_sampling()),
        ),
        Some(complot::Config::new().filename("opd.png")),
    )
        .into();

    Ok(())
}
