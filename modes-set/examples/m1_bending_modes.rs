//! # M1 singular modes conversion to CEO M1 modes
//! 
//! `cargo r -r --example m1_bending_modes --all-features`

use std::{fs::File, path::Path};

use crseo::{Builder, FromBuilder, Gmt, Source};
use crseo_modes_set::{ModesSets, Native};
use gmt_dos_systems_m1::SingularModes;
use interface::filing::Filing;

const N: usize = 256;
const L: f64 = 8.5;
const RAW: bool = false;
const N_MODE: usize = if RAW { 335 } else { 27 };

fn main() -> anyhow::Result<()> {
    let path = Path::new("examples").join("m1_singular_modes.pkl");
    let m1_singular_modes: SingularModes =
        serde_pickle::from_reader(&mut File::open(path)?, Default::default())?;
    // let segments_modes = &*m1_singular_modes;

    let mut modes_sets = ModesSets::<Native>::new(N, L, [0; 7]);
    if !RAW {
        modes_sets.insert_normalized_modes(&m1_singular_modes, N_MODE)
    } else {
        modes_sets.insert_raw_modes(&m1_singular_modes)
    }?;
    println!("{modes_sets}");
    modes_sets.to_path("examples/modesets.pkl")?;

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
