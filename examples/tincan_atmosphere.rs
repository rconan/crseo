use crseo::{Builder, ATMOSPHERE};
use log;
use skyangle::Conversion;
fn main() {
    env_logger::init();

    let sim_duration = 400f64;
    let atm_duration = 20f32;
    let atm_n_duration = Some((sim_duration / atm_duration as f64).ceil() as i32);
    let atm_sampling = 48 * 16 + 1;
    atm_n_duration.map(|atm_n_duration| {
        log::info!(
            "Atmosphere duration: {}x{}={}s",
            atm_duration,
            atm_n_duration,
            atm_duration * atm_n_duration as f32,
        )
    });

    let _ = ATMOSPHERE::new()
        .remove_turbulence_layer(0)
        .ray_tracing(
            25.5,
            atm_sampling,
            20f32.from_arcmin(),
            atm_duration,
            Some("ns-opm-im_atm.sh48.bin".to_string()),
            atm_n_duration,
        )
        .build()
        .unwrap();
}
