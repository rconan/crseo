use crseo::wavefrontsensor::LensletArray;
use crseo::{
    atmosphere,
    wavefrontsensor::{
        Calibration, GmtSegmentation, PistonSensor, Pyramid, PyramidCalibration,
        SegmentCalibration, Stroke, TruncatedPseudoInverse,
    },
    Atmosphere, Builder, FromBuilder, Gmt, SegmentWiseSensor, SegmentWiseSensorBuilder,
    WavefrontSensor, WavefrontSensorBuilder,
};

use nalgebra as na;
use nanorand::{Rng, WyRand};
use std::time::Instant;
use std::{fmt::Display, fs::File, path::Path};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Segment {
    sid: u8,
    n_mode: usize,
    mask: na::DMatrix<bool>,
    calibration: na::DMatrix<f32>,
}
impl Display for Segment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Segment #{} with {} modes, calibration: {:?}, mask: {:?}",
            self.sid,
            self.n_mode,
            self.calibration.shape(),
            self.mask.shape(),
        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Mirror {
    segments: Vec<Segment>,
    piston_mask: (Vec<bool>, Vec<bool>),
}
impl Display for Mirror {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for segment in &self.segments {
            writeln!(f, "{segment}")?;
        }
        writeln!(
            f,
            "Piston mask: [{},{}]",
            self.piston_mask.0.len(),
            self.piston_mask.1.len()
        )
    }
}

mod pym;
pub use pym::PyramidPiston;

#[allow(dead_code)]
enum CalibrationMode {
    Load,
    Save,
}

fn main() -> anyhow::Result<()> {
    let calib_mode = CalibrationMode::Load;

    let n_lenslet = 92;
    let n_mode = 450;

    let mut builder = Pyramid::builder()
        .lenslet_array(LensletArray {
            n_side_lenslet: n_lenslet,
            n_px_lenslet: 10,
            d: 0f64,
        })
        .modulation(2., 64);

    let mut slopes_mat: Calibration = match calib_mode {
        CalibrationMode::Save => {
            let now = Instant::now();
            let stroke0 = 25e-9;
            let mut slopes_mat = builder.clone().calibrate(
                SegmentCalibration::modes(
                    "ASM_DDKLs_S7OC04184_675kls",
                    1..n_mode,
                    "M2",
                    Stroke::RadialOrder(stroke0),
                ),
                builder.clone().guide_stars(None),
            );
            println!(
                "M2 {}modes/segment calibrated in {}s",
                n_mode,
                now.elapsed().as_secs()
            );
            // println!("{slopes_mat}");

            serde_pickle::to_writer(
                &mut File::create(format!("slopes_mat-{n_mode}_no-truss.pkl"))?,
                &slopes_mat,
                Default::default(),
            )?;

            slopes_mat
        }
        CalibrationMode::Load => {
            serde_pickle::from_reader(
                File::open(format!("slopes_mat-{n_mode}_no-truss.pkl"))?,
                Default::default(),
            )?
            //, Default::default())?;
        }
    };
    let mut n_sv = vec![None; 7];
    n_sv[6] = Some(18);
    println!(
        "Condition numbers: {:?}",
        slopes_mat.condition_number(Some(n_sv))
    );
    let mut truncation = vec![None; 7];
    truncation[6] = Some(TruncatedPseudoInverse::EigenValues(18));
    slopes_mat.pseudo_inverse(None).unwrap();

    builder.piston_sensor(&slopes_mat, GmtSegmentation::Outers)?;

    let mut gmt = Gmt::builder()
        .m2("ASM_DDKLs_S7OC04184_675kls", n_mode)
        .build()
        .unwrap();
    let src_builder = builder.guide_stars(None);
    let mut src = src_builder.clone().build().unwrap();
    let mut pym = builder.clone().build().unwrap();

    let mut buffer = vec![0f64; 7 * n_mode];

    // let mut atm = Atmosphere::builder().build()?;
    let mut atm = Atmosphere::builder()
        .ray_tracing(
            atmosphere::RayTracing::default()
                .duration(5f64)
                .filepath("/home/ubuntu/projects/grsim/ngao/data/atmosphere.bin"),
        )
        .build()?;

    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("pyramid");
    /*
    let mirror: Mirror = serde_pickle::from_reader(
        &File::open(path.join("py_calibration_segments.pkl"))?,
        Default::default(),
    )?;
    // let reconstructor: na::DMatrix<f32> =
    //     serde_pickle::from_reader(&File::open(path.join("pym_recon.pkl"))?, Default::default())?;
    let reconstructor: na::DMatrix<f32> = serde_pickle::from_reader(
        &File::open(path.join("pym_constrained_recon.pkl"))?,
        Default::default(),
    )?; */

    gmt.reset();
    src.through(&mut gmt).xpupil().through(&mut pym);
    let sxy0 = pym.processing();
    pym.reset();
    let pym_calibration = PyramidCalibration::new(
        sxy0,
        path.join("py_calibration_segments.pkl"),
        path.join("pym_constrained_recon.pkl"),
    )
    .unwrap();

    /*     let cum_mask = mirror.segments.iter().skip(1).fold(
        mirror.segments[0].mask.clone_owned(),
        |mut mask, segment| {
            mask.iter_mut()
                .zip(segment.mask.iter())
                .for_each(|(m1, mi)| *m1 = *m1 || *mi);
            mask
        },
    );
    let mut h_filter = cum_mask.into_iter().cycle();
    let mut p_filter = mirror
        .piston_mask
        .0
        .iter()
        .chain(mirror.piston_mask.1.iter())
        .cycle();

    let sxy0: Vec<_> = sx0
        .iter()
        .chain(sy0.iter())
        .zip(h_filter.by_ref())
        .filter_map(|(s, f)| f.then_some(*s))
        .chain(
            sx0.iter()
                .chain(sy0.iter())
                .zip(p_filter.by_ref())
                .filter_map(|(s, f)| f.then_some(*s)),
        )
        .collect(); */

    let gain = 0.5;

    let mut piston_dist = vec![0f32; 7];
    let mut rng = WyRand::new();
    // piston_dist.iter_mut().take(7).for_each(|x| {
    //     *x = (2. * rng.generate::<f32>() - 1.) * 25e-9;
    // });
    piston_dist[0] = 300e-9;
    println!(
        "Piston dist: {:4.0?}",
        piston_dist.iter().map(|x| x * 1e9).collect::<Vec<_>>()
    );

    let mut dp = vec![0f64; 7];
    let mut dp_counter = 0;

    for i in 0..1000 {
        pym.reset();
        // atm.secs = 1e-3 * i as f64;
        src.through(&mut gmt)
            .xpupil()
            // .add_piston(&piston_dist)
            .through(&mut atm)
            .through(&mut pym);
        let seg_wfe = src.segment_wfe_10e(-9);
        let p7 = seg_wfe[6].0;
        if i > 50 {
            dp.iter_mut()
                .zip(seg_wfe.iter())
                .for_each(|(dp, (p, _))| *dp += (*p - p7));
            dp_counter += 1;
        }
        println!(
            "#{:03}: WFE RMS [nm]: {:4.0?} {:4.0?}",
            i,
            src.wfe_rms_10e(-9),
            seg_wfe.into_iter().map(|(p, w)| (p, w)).collect::<Vec<_>>(),
        );

        if i % 100 == 0 {
            let _: complot::Heatmap = (
                (
                    src.phase().as_slice(),
                    (pym.pupil_sampling(), pym.pupil_sampling()),
                ),
                Some(complot::Config::new().filename("opd.png")),
            )
                .into();
        }
        let mut coefs = (&pym_calibration * &pym).unwrap();
        /*         let h_filter = cum_mask.into_iter().cycle();
        let p_filter = mirror
            .piston_mask
            .0
            .iter()
            .chain(mirror.piston_mask.1.iter())
            .cycle(); */
        /*         let (sx, sy) = pym.processing();
        let sxy: Vec<_> = sx
            .iter()
            .chain(sy.iter())
            .zip(h_filter.by_ref())
            .filter_map(|(s, f)| f.then_some(*s))
            .chain(
                sx.iter()
                    .chain(sy.iter())
                    .zip(p_filter.by_ref())
                    .filter_map(|(s, f)| f.then_some(*s)),
            )
            .zip(&sxy0)
            .map(|(s, s0)| s - *s0)
            .collect();
        let mut coefs = (&reconstructor * na::DVector::from_column_slice(&sxy))
            .as_slice()
            .to_vec(); */
        // coefs.insert(6 * n_mode, 0f32);

        if dp_counter == 10 {
            dp.iter_mut().for_each(|dp| *dp /= dp_counter as f64);
            coefs.chunks_mut(n_mode).zip(&dp).for_each(|(p, dp)| {
                if (*dp).abs() > 250f64 {
                    p[0] -= -*dp as f32 * 1e-9;
                }
            });
            println!("{:4.0?}", dp.iter().map(|x| x).collect::<Vec<_>>());
            dp_counter = 0;
            dp.fill(0f64);
        }

        println!(
            "{:4.2?}",
            coefs.chunks(n_mode).map(|x| x[0] * 1e9).collect::<Vec<_>>()
        );

        buffer
            .iter_mut()
            .zip(&coefs)
            .for_each(|(b, c)| *b -= gain * *c as f64);
        let piston = buffer.chunks(n_mode).map(|x| x[0]).sum::<f64>() / 7f64;
        buffer.chunks_mut(n_mode).for_each(|x| x[0] -= piston);

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
