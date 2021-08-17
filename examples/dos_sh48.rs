use crseo::Calibration;
use crseo::{
    calibrations,
    dos::{GmtOpticalModel, GmtOpticalSensorModel},
    shackhartmann::Diffractive as WFS_TYPE,
    shackhartmann::Geometric,
    Builder, CrseoError, ShackHartmann, SH48,
};
use dosio::{ios, Dos};
use std::time::Instant;

fn main() -> std::result::Result<(), CrseoError> {
    let wfs_blueprint = SH48::<WFS_TYPE>::new().n_sensor(1);
    let mut gosm = GmtOpticalSensorModel::<ShackHartmann<WFS_TYPE>, SH48<WFS_TYPE>>::new()
        .sensor(wfs_blueprint.clone())
        .build()?;
    gosm.src.fwhm(6f64);
    println!("M1 mode: {}", gosm.gmt.get_m1_mode_type());
    println!("M2 mode: {}", gosm.gmt.get_m2_mode_type());
    println!("GS band: {}", gosm.src.get_photometric_band());

    let mut gmt2wfs = Calibration::new(&gosm.gmt, &gosm.src, SH48::<Geometric>::new().n_sensor(1));
    let mirror = vec![calibrations::Mirror::M2];
    let segments = vec![vec![calibrations::Segment::Rxyz(1e-6, Some(0..2))]; 7];
    let now = Instant::now();
    gmt2wfs.calibrate(
        mirror,
        segments,
        calibrations::ValidLensletCriteria::OtherSensor(&mut gosm.sensor),
    );
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

    let m2_rbm = ios!(MCM2Lcl6D(m2_seg_rbm.into_iter().flatten().collect()));
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

    let mut gom = GmtOpticalModel::new().output(ios!(SrcWfeRms)).build()?;
    let y_wo_atm = gom.in_step_out(Some(vec![m2_rbm.clone()])).unwrap();

    let mut gom = GmtOpticalModel::new()
        .atmosphere(Default::default())
        .output(ios!(SrcWfeRms))
        .build()?;
    let y_w_atm = gom.in_step_out(Some(vec![m2_rbm.clone()])).unwrap();
    println!(
        "WFE RMS [nm] without and with atmosphere: {:.0}/{:.0}",
        Into::<Option<Vec<f64>>>::into(y_wo_atm.unwrap()[0].clone()).unwrap()[0] * 1e9,
        Into::<Option<Vec<f64>>>::into(y_w_atm.unwrap()[0].clone()).unwrap()[0] * 1e9
    );

    let mut gom = GmtOpticalModel::new().output(ios!(Pssn)).build()?;
    let y = gom.in_step_out(Some(vec![m2_rbm])).unwrap();

    println!(
        "PSSn : {:.4}",
        Into::<Option<Vec<f64>>>::into(y.unwrap()[0].clone()).unwrap()[0]
    );

    Ok(())
}
