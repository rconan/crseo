use std::{fs::File, path::Path};

use crseo::{Builder, FromBuilder, Gmt, Source};
use crseo_modes_set::{ModesSets, Native, Set};
use gmt_dos_systems_m1::SingularModes;

const N: usize = 256;
const L: f64 = 8.5;
const N_MODE: usize = 27;

fn main() -> anyhow::Result<()> {
    let path = Path::new("examples").join("m1_singular_modes.pkl");
    let m1_singular_modes: SingularModes =
        serde_pickle::from_reader(&mut File::open(path)?, Default::default())?;
    let segments_modes = &*m1_singular_modes;

    let mut modes_sets = ModesSets::<Native>::new(N, L, [0, 1, 2, 3, 4, 5, 6]);

    for (i, segment_modes) in segments_modes.iter().enumerate() {
        let modes = segment_modes.modes_into_mat(Some(N_MODE));
        let set = modes
            .column_iter()
            .map(|c| {
                let x = c.as_slice();
                let mean = x.iter().sum::<f64>() / x.len() as f64;
                let rms = ((x.iter().map(|x| *x - mean).map(|x| x * x).sum::<f64>())
                    / x.len() as f64)
                    .sqrt();
                // println!("{i}: {rms:.3e}");
                x.iter().map(|x| (*x - mean) / rms).collect::<Vec<_>>()
            })
            .collect::<Set>()
            .xy(segment_modes.nodes_iter().map(|n| [n[0], n[1]]));
        modes_sets.insert(i, set)?;
    }

    let mut gmt = Gmt::builder().try_m1(modes_sets, N_MODE)?.build()?;
    let mut src = Source::builder().build()?;

    let i = 15.min(N_MODE - 1);
    let c: Vec<_> = (0..7)
        .flat_map(|_| {
            let mut c = vec![0f64; N_MODE];
            c[i] = 1e-6;
            c
        })
        .collect();
    gmt.m1_modes(&c);

    src.through(&mut gmt).xpupil();
    println!("segment WFE {:.3?}micron", src.segment_wfe_10e(-6));
    let cfg = complot::Config::new().filename("examples/wavefront.png");
    complot::Heatmap::from(((src.phase().as_slice(), (512, 512)), Some(cfg)));
    Ok(())
}
