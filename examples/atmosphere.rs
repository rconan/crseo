use crseo::{ceo, Builder, AtmosphereBuilder};
use serde_pickle as pkl;
use skyangle::Conversion;
use std::fs::File;

fn main() {
    env_logger::init();

    let mut gmt = ceo!(GmtBuilder);
    let mut src = ceo!(SourceBuilder);

    let sim_sampling_frequency = 1000_usize;

    const CFD_RATE: usize = 1;
    const CFD_DELAY: usize = 10; // seconds
    let cfd_sampling_frequency = sim_sampling_frequency / CFD_RATE;

    const M1_RATE: usize = 10;
    assert_eq!(sim_sampling_frequency / M1_RATE, 100); // Hz

    const SH48_RATE: usize = 30000;
    assert_eq!(SH48_RATE / sim_sampling_frequency, 30); // Seconds

    const FSM_RATE: usize = 5;
    assert_eq!(sim_sampling_frequency / FSM_RATE, 200); // Hz

    type D = Vec<f64>;

    let sim_duration = (CFD_DELAY + 30 * SH48_RATE / sim_sampling_frequency) as f64;

    let atm_duration = 20f32;
    let atm_n_duration = Some((sim_duration / atm_duration as f64).ceil() as i32);
    let atm_sampling = 48 * 16 + 1;
    let atm_builder = AtmosphereBuilder::builder().ray_tracing(
        25.5,
        atm_sampling,
        20f32.from_arcmin(),
        atm_duration,
        Some("atm_15mn.bin".to_owned()),
        atm_n_duration,
    );
    println!("{atm_builder:#?}");

    let mut atm = atm_builder.build().unwrap();
    src.through(&mut gmt).xpupil().through(&mut atm);
    let wfe_rms = src.wfe_rms_10e(-9);
    println!("{wfe_rms:?}");

    /*
        let mut atm_1 = ATMOSPHERE::new()
            .single_turbulence_layer(0f32, None, None)
            .build()
            .unwrap();
        let mut atm_2 = ATMOSPHERE::new()
            .single_turbulence_layer(0f32, None, None)
            .ray_tracing(25.5, 512, 0., 1., Some("atm_2.bin".to_owned()), None)
            .build()
            .unwrap();

        let dump = |data: &Vec<f32>, filename: &str| {
            let mut file = File::create(filename).unwrap();
            pkl::to_writer(&mut file, data, true).unwrap();
        };
        dump(
            &(src.through(&mut gmt).xpupil().through(&mut atm_1).phase()),
            "atm_1.pkl",
        );
        dump(
            &(src.through(&mut gmt).xpupil().through(&mut atm_2).phase()),
            "atm_2.pkl",
        );
    */
    /*
    let mut atm = ceo!(ATMOSPHERE);
    let dt = 10_f64;
    for k in 0..10 {
        atm.secs = k as f64 * dt;
        src.through(&mut gmt).xpupil().through(&mut atm);
        println!(
            "T: {:02}s -> WFE RMS: {:.0}nm",
            atm.secs,
            src.wfe_rms_10e(-9)[0]
        );
    }
    */
}
