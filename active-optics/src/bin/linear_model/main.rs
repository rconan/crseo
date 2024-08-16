use active_optics::{Calib, SID};
use complot::{Config, Heatmap};
use crseo::gmt::{GmtM1, GmtM2};
use faer::mat::from_column_major_slice;
use skyangle::Conversion;
use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut calib_m2_modes =
        Calib::<GmtM2, SID>::load(format!("src/bin/calibration/onaxis_M2S{SID}_modes.pkl"))?;
    println!("{calib_m2_modes}");

    let mut calib_m1_rbms =
        Calib::<GmtM1, SID>::load(format!("src/bin/calibration/onaxis_M1S{SID}_rbms.pkl"))?;
    println!("{calib_m1_rbms}");

    // calib_m2_modes.match_areas(&mut calib_m1_rbms);
    calib_m1_rbms.match_areas(&mut calib_m2_modes);
    println!("{calib_m2_modes}");
    println!("{calib_m1_rbms}");

    let m1_to_m2 = &calib_m2_modes.pseudoinverse() * &calib_m1_rbms;
    println!("M1->M2 ({},{})", m1_to_m2.nrows(), m1_to_m2.ncols());

    // let a = [100f64.from_mas(), 0.];
    // let b = &m1_to_m2 * from_column_major_slice::<f64>(&a, 2, 1);
    // b.col(0).iter().enumerate().for_each(|(i, x)| {
    //     if x.abs() > 1e-9 {
    //         println!("{:3}: {:8.1}", i + 1, x * 1e9)
    //     }
    // });

    let mut calib_offaxis_m2_modes =
        Calib::<GmtM2, SID>::load(format!("src/bin/calibration/offaxis_M2S{SID}_modes.pkl"))?;
    println!("{calib_offaxis_m2_modes}");

    let mut calib_offaxis_m1_rbms =
        Calib::<GmtM1, SID>::load(format!("src/bin/calibration/offaxis_M1S{SID}_rbms.pkl"))?;
    println!("{calib_offaxis_m1_rbms}");

    calib_offaxis_m2_modes.match_areas(&mut calib_offaxis_m1_rbms);
    println!("{calib_offaxis_m2_modes}");
    println!("{calib_offaxis_m1_rbms}");

    let m1_to_agws =
        &calib_offaxis_m1_rbms.mat_ref() - calib_offaxis_m2_modes.mat_ref() * &m1_to_m2;
    println!("M1->AGWS ({},{})", m1_to_agws.nrows(), m1_to_agws.ncols());

    m1_to_agws.col_iter().enumerate().for_each(|(c_i, c)| {
        let c0 = calib_offaxis_m1_rbms.unmask(c.iter());
        c0.chunks(calib_offaxis_m2_modes.src_mask_len())
            .enumerate()
            .for_each(|(i, x)| {
                let n = calib_offaxis_m2_modes.src_mask_square_len();
                let filename = format!("src/bin/linear_model/m1_to_agws_col{c_i}src{i}.png");
                let _: Heatmap = ((x, (n, n)), Some(Config::new().filename(&filename))).into();
                let thumbnail = format!("src/bin/linear_model/m1_to_agws_col{c_i}src{i}_.png");
                Command::new("/usr/bin/convert")
                    .arg(filename)
                    .arg("-resize")
                    .arg("15%")
                    .arg(thumbnail)
                    .output()
                    .expect("failed to execute process");
            });
    });

    Ok(())
}
