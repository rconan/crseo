use crseo::{
    ceo,
    cu::Single,
    shackhartmann::{Geometric, WavefrontSensor, WavefrontSensorBuilder},
    Builder, Cu, AtmosphereBuilder, LinearMinimumMeanSquareErrorBuilder, SH48, SourceBuilder,
};
use nalgebra as na;
use serde_pickle as pickle;
use skyangle::SkyAngle;
use std::fs::File;
use std::time::Instant;

fn main() {
    let n_actuator = 49;
    let n_kl = 70;

    let atm_blueprint = AtmosphereBuilder::builder();
    let wfs_blueprint = SH48::<Geometric>::builder(); //.n_sensor(1);
    let gs_blueprint = wfs_blueprint
        .guide_stars(None)
        .on_ring(SkyAngle::Arcminute(6f32).to_radians());
    let src_blueprint = SourceBuilder::builder().pupil_sampling(n_actuator);

    let mut gmt = ceo!(GmtBuilder, m2_n_mode = [n_kl]);
    let mut mmse_src = src_blueprint.clone().build().unwrap();

    let mut gs = gs_blueprint.build().unwrap();

    let mut lmmse = LinearMinimumMeanSquareErrorBuilder::builder()
        .atmosphere(atm_blueprint.clone())
        .guide_star(&gs)
        .mmse_star(&mmse_src)
        .n_side_lenslet(n_actuator - 1)
        .build()
        .unwrap();

    let mut atm = atm_blueprint.build().unwrap();
    let mut wfs = wfs_blueprint.build().unwrap();

    gs.through(&mut gmt).xpupil();
    wfs.calibrate(&mut gs, 0.5);

    wfs.reset();
    gs.through(&mut gmt)
        .xpupil()
        .through(&mut atm)
        .through(&mut wfs);
    wfs.process();
    let now = Instant::now();
    let mut lmmse_phase = lmmse.get_wavefront_estimate(&mut wfs).phase_as_ptr();
    let et = now.elapsed().as_secs_f64();
    println!("# of iteration: {}", lmmse.get_n_iteration());
    mmse_src.through(&mut gmt).xpupil().through(&mut atm);
    println!("WFE RMS: {:.0}nm", mmse_src.wfe_rms_10e(-9)[0]);
    let src_phase = mmse_src.phase().clone();
    mmse_src.sub(&mut lmmse_phase);
    println!(
        "Residual WFE RMS: {:.0}nm in {:.3}s (Toeplitz)",
        mmse_src.wfe_rms_10e(-9)[0],
        et
    );

    let tomo_recon = {
        let f = File::open("/home/rconan/Documents/GMT/Notes/GLAO/Marcos/tomo_recon.pkl").unwrap();
        let data: Vec<f64> = pickle::from_reader(f).unwrap();
        na::DMatrix::from_row_slice(n_actuator * n_actuator, wfs.n_centroids as usize, &data)
    };
    let c = na::DVector::from_vec(wfs.centroids.clone().into()).map(|x| x as f64);
    let now = Instant::now();
    let mut mvm_phase: Cu<Single> = (tomo_recon * c)
        .map(|x| x as f32)
        .as_slice()
        .to_owned()
        .into();
    let et = now.elapsed().as_secs_f64();
    mmse_src.through(&mut gmt).xpupil().through(&mut atm);
    mmse_src.sub(&mut mvm_phase);
    println!(
        "Residual WFE RMS: {:.0}nm in {:.3}s (MVM)",
        mmse_src.wfe_rms_10e(-9)[0],
        et
    );

    let phase = Vec::<f32>::from(lmmse_phase);
    let phase_est = Vec::<f32>::from(mvm_phase);
    let mut file = File::create("tomography.pkl").unwrap();
    pickle::to_writer(&mut file, &(src_phase, phase, phase_est), true).unwrap();

    /*
    let kln = lmmse.calibrate_karhunen_loeve(n_kl, None, None);
    let mut kl_coefs = lmmse.get_karhunen_loeve_coefficients(&kln, Some(-1f64));

    let mut file = File::create("KL_coefs.pkl").unwrap();
    pickle::to_writer(&mut file, &kl_coefs, true).unwrap();

    let mut src = ceo!(SOURCE);

    src.through(&mut gmt).xpupil().through(&mut atm);
    println!("WFE RMS: {}nm", src.wfe_rms_10e(-9)[0]);
    let src_phase: Vec<f32> = src.phase_as_ptr().into();
    let mut file = File::create("SRC_phase.pkl").unwrap();
    pickle::to_writer(&mut file, &src_phase, true).unwrap();

    gmt.m2_modes(&mut kl_coefs);
    src.through(&mut gmt).xpupil().through(&mut atm);
    println!("KL residual WFE RMS: {}nm", src.wfe_rms_10e(-9)[0]);

    let kl_phase: Vec<f32> = src.phase_as_ptr().into();
    let mut file = File::create("KL_phase.pkl").unwrap();
    pickle::to_writer(&mut file, &kl_phase, true).unwrap();
    */
}
