use complot::{Axis, Config, Plot};
use crseo::{dos::GmtOpticalModel, Builder, GMT, SOURCE};
use dosio::{ios, Dos};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let sim_duration = 60f64;
    let sampling_rate = 1e3;
    let m1_n_mode = 332;
    let soak_delta_temperature = 10f64;
    let n_px = 16 * 48 + 1;
    let mut gom = GmtOpticalModel::new()
        .gmt(GMT::new())
        .source(SOURCE::new().pupil_sampling(n_px))
        .output(ios!(SrcSegmentWfeRms))
        .dome_seeing("b2019_0z_0az_os_7ms", sim_duration, sampling_rate, None)
        .await?
        .build()?;
    gom.gmt.a1 = (0..7)
        .flat_map(|_| {
            let mut a1 = vec![0f64; m1_n_mode];
            a1[m1_n_mode - 3] = 1f64;
            a1[m1_n_mode - 2] = 1f64;
            a1[m1_n_mode - 1] = soak_delta_temperature;
            a1
        })
        .collect();
    gom.gmt.reset();

    gom.in_step_out(None)?;

    let _: complot::Heatmap = (
        (
            gom.src
                .phase()
                .iter()
                .map(|&x| x as f64 * 1e6)
                .collect::<Vec<f64>>()
                .as_slice(),
            (n_px, n_px),
        ),
        None,
    )
        .into();

    Ok(())
}
