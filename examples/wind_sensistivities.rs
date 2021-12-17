use crseo::{ceo, Conversion};
use dosio::{io::jar, io::IOError, IO};
use serde_pickle as pickle;
use std::fs::File;
use std::io::BufReader;

type TimeSeries = Vec<(f64, Vec<f64>)>;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (m1_rbm_ts, m2_rbm_ts): (TimeSeries, TimeSeries) = {
        let file = BufReader::with_capacity(
            100_000,
            File::open("examples/windloading.20210225_1447_MT_mount_v202102_ASM_wind2.pkl")?,
        );
        let data: Vec<IO<TimeSeries>> = pickle::from_reader(file)?;
        (
            std::result::Result::<TimeSeries, IOError>::from(data[jar::OSSM1Lcl::new()].clone())
                .map_err(|e| format!("{:?}", e))?,
            std::result::Result::<TimeSeries, IOError>::from(data[jar::MCM2RB6D::new()].clone())
                .map_err(|e| format!("{:?}", e))?,
        )
    };
    println!("M1 RBM ({}): {}", m1_rbm_ts.len(), m1_rbm_ts[0].1.len());
    println!("M2 RBM ({}): {}", m2_rbm_ts.len(), m2_rbm_ts[0].1.len());

    let mut gmt = ceo!(GMT);
    let mut src = ceo!(SOURCE);
    let tt: Vec<_> = m1_rbm_ts
        .into_iter()
        .map(|x| x.1)
        .zip(m2_rbm_ts.into_iter().map(|x| x.1))
        .take(5)
        .map(|(m1_rbm, m2_rbm)| {
            gmt.update42(Some(&m1_rbm), Some(&m2_rbm), None, None);
            src.through(&mut gmt).xpupil();
            src.gradients()
                .into_iter()
                .map(|x| x.to_mas())
                .collect::<Vec<f32>>()
        })
        .collect();
    println!("TT: {:#?}",tt);
    Ok(())
}
