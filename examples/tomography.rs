use ceo::{ceo, shackhartmann::Geometric, Builder, ATMOSPHERE, LMMSE, SH48, SOURCE};
use serde_pickle as pickle;
use std::fs::File;

fn main() {
    let n_actuator = 49;
    let n_kl = 70;

    let atm_blueprint = ATMOSPHERE::new();
    let wfs_blueprint = SH48::<Geometric>::new().set_n_sensor(1);
    let gs_blueprint = wfs_blueprint.guide_stars();
    let src_blueprint = SOURCE::new().set_pupil_sampling(n_actuator);

    let mut gmt = ceo!(GMT, set_m2_n_mode = [n_kl]);
    let mut mmse_src = src_blueprint.clone().build();

    let mut gs = gs_blueprint.build();

    let mut lmmse = LMMSE::new()
        .set_atmosphere(atm_blueprint.clone())
        .set_guide_star(&gs)
        .set_mmse_star(&mmse_src)
        .set_n_side_lenslet(n_actuator - 1)
        .build();

    let mut atm = atm_blueprint.build();
    let mut wfs = wfs_blueprint.build();

    gs.through(&mut gmt).xpupil();
    wfs.calibrate(&mut gs, 0.5);

    wfs.reset();
    gs.through(&mut gmt)
        .xpupil()
        .through(&mut atm)
        .through(&mut wfs);
    wfs.process();
    let mut lmmse_phase = lmmse.get_wavefront_estimate(&mut wfs).phase_as_ptr();
    println!("# of iteration: {}", lmmse.get_n_iteration());
    mmse_src.through(&mut gmt).xpupil().through(&mut atm);
    println!("WFE RMS: {}nm", mmse_src.wfe_rms_10e(-9)[0]);
    let src_phase = mmse_src.phase().clone();
    mmse_src.sub(&mut lmmse_phase);
    println!("Residual WFE RMS: {}nm", mmse_src.wfe_rms_10e(-9)[0]);

    let kln = lmmse.calibrate_karhunen_loeve(n_kl, None, None);
    let mut kl_coefs = lmmse.get_karhunen_loeve_coefficients(&kln, Some(-1f64));

    let mut file = File::create("KL_coefs.pkl").unwrap();
    pickle::to_writer(&mut file, &kl_coefs, true).unwrap();

    let phase = Vec::<f32>::from(lmmse_phase);
    let mut file = File::create("tomography.pkl").unwrap();
    pickle::to_writer(&mut file, &(src_phase, phase), true).unwrap();

    let mut src = ceo!(SOURCE);

    src.through(&mut gmt).xpupil().through(&mut atm);
    println!("WFE RMS: {}nm", src.wfe_rms_10e(-9)[0]);
    let src_phase: Vec<f32> = src.phase_as_ptr().into();
    let mut file = File::create("SRC_phase.pkl").unwrap();
    pickle::to_writer(&mut file, &src_phase, true).unwrap();

    gmt.set_m2_modes(&mut kl_coefs);
    src.through(&mut gmt).xpupil().through(&mut atm);
    println!("KL residual WFE RMS: {}nm", src.wfe_rms_10e(-9)[0]);

    let kl_phase: Vec<f32> = src.phase_as_ptr().into();
    let mut file = File::create("KL_phase.pkl").unwrap();
    pickle::to_writer(&mut file, &kl_phase, true).unwrap();
}
