use complot::{Config, Plot};
use crseo::{ceo, dos};
use std::time::Instant;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let n_px = 769;
    let mut gmt = ceo!(GMT);
    let mut src = ceo!(SOURCE, pupil_sampling = [n_px]);
    let cfd_case = "b2019_0z_0az_os_7ms";
    let duration = 5000; // Dome seeing is sampled a 5Hz
    let rate = 10; // Set the sampling rate to `rate` x 5Hz
    let mut ds = dos::DomeSeeing::new(
        "us-west-2",
        "gmto.modeling",
        "Baseline2020",
        &cfd_case,
        duration,
        Some(rate),
    );
    ds.get_keys().await?.load_opd().await?;

    println!("Dome seeing sampling time: {:.4}s", ds.sampling_time);
    let mut data_1 = vec![];
    let now = Instant::now();
    while let Some(_) = ds.next() {
        src.through(&mut gmt).xpupil().through(&mut ds);
        data_1.push((ds.current_time, vec![src.wfe_rms_10e(-6)[0] as f64]));
    }
    println!(
        "Elapsed time: {:.3}s",
        now.elapsed().as_millis() as f64 * 1e-3
    );
    println!(
        "Time range: {:.3?}",
        (data_1[0].0, data_1.last().unwrap().0)
    );

    let _: Plot = (
        data_1.into_iter(),
        Some(Config::new().filename("opd_timeseries.svg")),
    )
        .into();

    Ok(())
}
