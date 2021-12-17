use bincode;
use crseo::{ceo, Conversion, Gmt, Source};
use dosio::{io::jar, io::IOError, IO};
use nalgebra as na;
use serde::{Deserialize, Serialize};
use serde_pickle as pickle;
use std::{fs::File, io::BufReader, path::Path, time::Instant};

#[derive(Deserialize)]
enum OpticalSensitivities {
    Wavefront(Vec<f64>),
    TipTilt(Vec<f64>),
    SegmentTipTilt(Vec<f64>),
    SegmentPiston(Vec<f64>),
}
#[derive(Serialize, Debug)]
enum OpticalWindLoad {
    // square root time average variance [nm]
    Wavefront(f64),
    // time standard deviation [mas]
    TipTilt(Vec<f64>),
    // time standard deviation [mas]
    SegmentTipTilt(Vec<Vec<f64>>),
    // time standard deviation [nm]
    SegmentPiston(Vec<f64>),
    // square root time average variance of 21 differences [nm]
    DifferentialSegmentPiston(f64),
}

type TimeSeries = Vec<(f64, Vec<f64>)>;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new("examples");
    let (m1_rbm_ts, m2_rbm_ts): (TimeSeries, TimeSeries) = {
        let file = BufReader::with_capacity(
            100_000,
            File::open(path.join("windloading.20210225_1447_MT_mount_v202102_ASM_wind2.pkl"))?,
        );
        let data: Vec<IO<TimeSeries>> = pickle::from_reader(file)?;
        (
            std::result::Result::<TimeSeries, IOError>::from(data[jar::OSSM1Lcl::new()].clone())
                .map_err(|e| format!("{:?}", e))?,
            std::result::Result::<TimeSeries, IOError>::from(data[jar::MCM2RB6D::new()].clone())
                .map_err(|e| format!("{:?}", e))?,
        )
    };
    //println!("M1 RBM ({}): {}", m1_rbm_ts.len(), m1_rbm_ts[0].1.len());
    //println!("M2 RBM ({}): {}", m2_rbm_ts.len(), m2_rbm_ts[0].1.len());

    let skip = 30_000;
    let n_sample = m1_rbm_ts.len() - skip;
    let rbm_iter = m1_rbm_ts
        .iter()
        .zip(m2_rbm_ts.iter())
        .skip(skip)
        .flat_map(|(x1, x2)| {
            let mut u = x1.1.clone();
            let mut v = x2.1.clone();
            u.append(&mut v);
            u
        });
    let rbm = na::DMatrix::from_iterator(84, n_sample, rbm_iter);

    /*
        let mut gmt = ceo!(GMT);
        let mut src = ceo!(SOURCE);
        let now = Instant::now();
        let tt: Vec<_> = m1_rbm_ts
            .iter()
            .map(|x| x.1.clone())
            .zip(m2_rbm_ts.iter().map(|x| x.1.clone()))
            .skip(skip)
            .take(100)
            .map(|(m1_rbm, m2_rbm)| {
                gmt.update42(Some(&m1_rbm), Some(&m2_rbm), None, None);
                src.through(&mut gmt).xpupil();
                src.wfe_rms_10e(-9)
                /*src.gradients()
                .into_iter()
                .map(|x| x.to_mas())
                .collect::<Vec<f32>>()*/
            })
            .collect();
        println!("Elpased time: {:.3}s", now.elapsed().as_secs_f64());
        println!("TT: {:#?}", tt);
        pickle::to_writer(&mut File::create("examples/wavefront.pkl")?, src.phase(), true)?;
    */

    let mut metrics = vec![];
    let optical_sensitivities: [OpticalSensitivities; 4] =
        bincode::deserialize_from(BufReader::with_capacity(
            100_000,
            File::open(path.join("optical_sensitivities.rs.bin"))?,
        ))?;
    for sens in optical_sensitivities.iter() {
        match sens {
            OpticalSensitivities::Wavefront(sens) => {
                let n = sens.len() / 84;
                //println!("n: {}", n);
                let sensitivity = na::DMatrix::from_column_slice(n, 84, sens);
                //let now = Instant::now();
                let wfe_var = {
                    let n_buf = 1_000;
                    let mut buf = na::DMatrix::<f64>::zeros(n, n_buf);
                    let mut s = 0;
                    let mut var = 0f64;
                    loop {
                        if s + n_buf > n_sample {
                            s -= n_buf;
                            let n_last = n_sample - s;
                            let mut buf = na::DMatrix::<f64>::zeros(n, n_last);
                            buf.gemm(1f64, &sensitivity, &rbm.columns(s, n_last), 0f64);
                            var += buf.row_variance().as_slice().into_iter().sum::<f64>();
                            break var;
                        } else {
                            buf.gemm(1f64, &sensitivity, &rbm.columns(s, n_buf), 0f64);
                            var += buf.row_variance().as_slice().into_iter().sum::<f64>();
                        }
                        s += n_buf;
                    }
                };
                let value = 1e9 * (wfe_var / n_sample as f64).sqrt();
                metrics.push(OpticalWindLoad::Wavefront(value));
                /*println!(
                    "Wavefront: {:6.0}nm in {:.3}s", value,
                    now.elapsed().as_secs_f64()
                );*/
            }
            OpticalSensitivities::TipTilt(sens) => {
                let sensitivity = na::DMatrix::from_column_slice(2, 84, sens);
                let tip_tilt = (sensitivity * &rbm).map(|x| x.to_mas());
                let values = tip_tilt
                    .column_variance()
                    .map(|x| x.sqrt())
                    .as_slice()
                    .to_owned();
                //println!("TT: {:2.0?}mas", &values);
                metrics.push(OpticalWindLoad::TipTilt(values));
            }
            OpticalSensitivities::SegmentTipTilt(sens) => {
                let sensitivity = na::DMatrix::from_column_slice(14, 84, sens);
                let segment_tip_tilt = (sensitivity * &rbm).map(|x| x.to_mas());
                let values: Vec<_> = segment_tip_tilt
                    .column_variance()
                    .map(|x| x.sqrt())
                    .as_slice()
                    .chunks(7)
                    .map(|x| x.to_owned())
                    .collect();
                //println!("Segment TT: {:2.0?}mas", values,);
                metrics.push(OpticalWindLoad::SegmentTipTilt(values));
            }
            OpticalSensitivities::SegmentPiston(sens) => {
                let sensitivity = na::DMatrix::from_column_slice(7, 84, sens);
                let segment_piston = (sensitivity * &rbm).map(|x| x * 1e9);
                let mut v: Vec<f64> = vec![];
                for (k, row) in segment_piston.row_iter().take(6).enumerate() {
                    //println!("{}: {:?}", k, row.shape());
                    v.extend(
                        &mut segment_piston
                            .rows(k + 1, 6 - k)
                            .row_iter()
                            .flat_map(|y| (y - row).as_slice().to_owned()),
                    );
                }
                let value = (na::DMatrix::from_vec(n_sample, 21, v)
                    .column_variance()
                    .sum()
                    / n_sample as f64)
                    .sqrt();
                //println!("Diff. piston std: {:5.0}nm", value,);
                metrics.push(OpticalWindLoad::DifferentialSegmentPiston(value));
                let values = segment_piston
                    .column_variance()
                    .map(|x| x.sqrt())
                    .as_slice()
                    .to_owned();
                //println!("Piston: {:3.0?}nm ; ", &values);
                metrics.push(OpticalWindLoad::SegmentPiston(values));
            }
        }
    }
    println!("Optical Wind Loads: {:#?}", metrics);
    pickle::to_writer(
        &mut File::create(path.join("optical_wind_loads.pkl"))?,
        &metrics,
        true,
    )?;
    Ok(())
}
