use crseo::{
    calibrations,
    dos::{GmtOpticalModel, GmtOpticalSensorModel},
    shackhartmann::Geometric as WFS_TYPE,
    Builder, Calibration, ShackHartmann, ATMOSPHERE, SH48,
};
use dosio::{io::jar, Dos};
use std::time::Instant;

fn main() {
    let wfs_blueprint = SH48::<WFS_TYPE>::new().n_sensor(1);
    let mut gosm = GmtOpticalSensorModel::<ShackHartmann<WFS_TYPE>, SH48<WFS_TYPE>>::new(
        wfs_blueprint.clone(),
        0.8,
    )
    .build();
    println!("M1 mode: {}", gosm.gmt.get_m1_mode_type());
    println!("M2 mode: {}", gosm.gmt.get_m2_mode_type());
    println!("GS band: {}", gosm.src.get_photometric_band());

    let mut gmt2wfs = Calibration::new(&gosm.gmt, &gosm.src, wfs_blueprint);
    let mirror = vec![calibrations::Mirror::M2];
    let segments = vec![vec![calibrations::Segment::Rxyz(1e-6, Some(0..2))]; 7];
    let now = Instant::now();
    gmt2wfs.calibrate(mirror, segments, Some(0.8));
    println!(
        "GTM 2 WFS calibration [{}x{}] in {}s",
        gmt2wfs.n_data,
        gmt2wfs.n_mode,
        now.elapsed().as_secs()
    );
    let poke_sum = gmt2wfs.poke.from_dev().iter().sum::<f32>();
    println!("Poke sum: {}", poke_sum);

    let mut m2_seg_rbm = vec![vec![0f64; 6]; 7];
    m2_seg_rbm[1][3] = 1e-6;
    m2_seg_rbm[4][4] = 1e-6;
    m2_seg_rbm[6][3] = 1e-6;
    m2_seg_rbm[6][4] = 1e-6;

    let m2_rbm = jar::MCM2Lcl6D::with(m2_seg_rbm.into_iter().flatten().collect());
    //    gosm.inputs(vec![m2_rbm.clone()]).unwrap().step();
    let y = gosm
        .in_step_out(Some(vec![m2_rbm.clone()]))
        .map(|x| Into::<Option<Vec<f64>>>::into(x.unwrap()[0].clone()))
        .unwrap()
        .map(|x| x.into_iter().map(|x| x as f32).collect::<Vec<f32>>())
        .unwrap();

    let a = gmt2wfs.qr().solve(&mut y.into());
    Vec::<f32>::from(a)
        .into_iter()
        .map(|x| x * 1e6)
        .collect::<Vec<f32>>()
        .chunks(2)
        .enumerate()
        .for_each(|x| println!("#{}: [{:+0.1},{:+0.1}]", 1 + x.0, x.1[0], x.1[1]));

    let mut gom = GmtOpticalModel::new().output(jar::SrcWfeRms::new()).build();
    let y = gom.in_step_out(Some(vec![m2_rbm.clone()])).unwrap();
    println!(
        "y: {:e}m",
        Into::<Option<Vec<f64>>>::into(y.unwrap()[0].clone()).unwrap()[0]
    );

    let mut gom = GmtOpticalModel::new()
        .atmosphere(Default::default())
        .output(jar::SrcWfeRms::new())
        .build();
    let y = gom.in_step_out(Some(vec![m2_rbm.clone()])).unwrap();
    println!(
        "y: {:e}m",
        Into::<Option<Vec<f64>>>::into(y.unwrap()[0].clone()).unwrap()[0]
    );

    let mut gom = GmtOpticalModel::new().output(jar::Pssn::new()).build();
    let y = gom.in_step_out(None).unwrap();
}
