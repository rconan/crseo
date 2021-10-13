use complot::{Combo, Complot, Config, Kind};
use crseo::{ceo, dos};
use indicatif::ProgressBar;
use std::time::Instant;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let n_px = 769;
    let mut gmt = ceo!(GMT);
    let mut src = ceo!(SOURCE, pupil_sampling = [n_px]);
    let cfd_case = "b2019_0z_0az_os_7ms";
    let duration = 20; // Dome seeing is sampled a 5Hz, so the time range is [0,4)s
    let rate = 1; // Set the sampling rate to `rate` x 5Hz
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
    let bar = ProgressBar::new(ds.n_step as u64);
    let now = Instant::now();
    while let Some(_) = ds.next() {
        bar.inc(1);
        src.through(&mut gmt).xpupil().through(&mut ds);

        data_1.push((ds.current_time, vec![src.wfe_rms_10e(-9)[0] as f64]));
    }
    bar.finish();
    println!(
        "Elapsed time: {:.3}s",
        now.elapsed().as_millis() as f64 * 1e-3
    );
    println!(
        "Time range: {:.3?}",
        (data_1[0].0, data_1.last().unwrap().0)
    );

    let _: complot::Heatmap = (
        (
            src.phase()
                .iter()
                .map(|&x| x as f64 * 1e6)
                .collect::<Vec<f64>>()
                .as_slice(),
            (n_px, n_px),
        ),
        None,
    )
        .into();

    ds.reset(Some(2));
    println!("Dome seeing sampling time: {:.4}s", ds.sampling_time);
    let mut data_2 = vec![];
    let bar = ProgressBar::new(ds.n_step as u64);
    let now = Instant::now();
    while let Some(_) = ds.next() {
        bar.inc(1);
        src.through(&mut gmt).xpupil().through(&mut ds);

        data_2.push((ds.current_time, vec![src.wfe_rms_10e(-9)[0] as f64]));
    }
    bar.finish();
    println!(
        "Elapsed time: {:.3}s",
        now.elapsed().as_millis() as f64 * 1e-3
    );
    println!(
        "Time range: {:.3?}",
        (data_2[0].0, data_2.last().unwrap().0)
    );

    ds.reset(Some(10));
    println!("Dome seeing sampling time: {:.4}s", ds.sampling_time);
    let mut data = vec![];
    let bar = ProgressBar::new(ds.n_step as u64);
    let now = Instant::now();
    while let Some(_) = ds.next() {
        bar.inc(1);
        src.through(&mut gmt).xpupil().through(&mut ds);

        data.push((ds.current_time, vec![src.wfe_rms_10e(-9)[0] as f64]));
    }
    bar.finish();
    println!(
        "Elapsed time: {:.3}s",
        now.elapsed().as_millis() as f64 * 1e-3
    );
    println!("Time range: {:.3?}", (data[0].0, data.last().unwrap().0));

    let mut cfg = Config::new().filename("opd_sampling.svg");
    cfg.auto_range(vec![&data_2, &data_1, &data]);
    let _: Combo = From::<Complot>::from((
        vec![
            Box::new(data_2.into_iter()),
            Box::new(data_1.into_iter()),
            Box::new(data.into_iter()),
        ],
        vec![Kind::Scatter, Kind::Scatter, Kind::Plot],
        Some(cfg),
    ));

    Ok(())
}
