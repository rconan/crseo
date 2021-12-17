use crseo::{dos::GmtOpticalModel, Builder};
use dosio::{io::jar, Dos};
use std::time::Instant;

fn main() {
    let mut gosm = GmtOpticalModel::new().build();

    let mut m2_seg_rbm = vec![vec![0f64; 6]; 7];
    m2_seg_rbm[1][3] = 1e-6;
    m2_seg_rbm[4][4] = 1e-6;
    m2_seg_rbm[6][3] = 1e-6;
    m2_seg_rbm[6][4] = 1e-6;
    let m2_rbm = jar::MCM2Lcl6D::with(m2_seg_rbm.into_iter().flatten().collect());
    //    gosm.inputs(vec![m2_rbm.clone()]).unwrap().step();
    let n_step = 30 * 100;
    let now = Instant::now();
    for _ in 0..n_step {
        let _y = gosm.in_step_out(Some(vec![m2_rbm.clone()]));
    }
    println!("Elapsed time: {}ms", now.elapsed().as_millis());
}
