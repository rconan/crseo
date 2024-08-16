use active_optics::{Calib, SID};
use crseo::FromBuilder;
use skyangle::Conversion;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut calib = Calib::<crseo::gmt::GmtM2, SID>::new(active_optics::M2_N_MODE);
    calib
        .calibrate_segment_modes(1e-6)?
        .dump(format!("src/bin/calibration/onaxis_M2S{SID}_modes.pkl"))?;
    println!("onaxis_M2S{SID}_modes: {calib}");

    let mut calib = Calib::<crseo::gmt::GmtM1, SID>::new(6);
    calib
        .calibrate_rigid_body_motions([
            Some(1e-6),               // Tx
            Some(1e-6),               // Ty
            Some(1e-6),               // Tz
            Some(1f64.from_arcsec()), // Rx
            Some(1f64.from_arcsec()), // Ry
            Some(1f64.from_arcsec()), // Rz
        ])?
        .dump(format!("src/bin/calibration/onaxis_M1S{SID}_rbms.pkl"))?;
    println!("onaxis_M1S{SID}_rbms: {calib}");

    let mut calib = Calib::<crseo::gmt::GmtM2, SID>::new(active_optics::M2_N_MODE)
        .guide_star(crseo::Source::builder().size(3).on_ring(6f32.from_arcmin()));
    calib
        .calibrate_segment_modes(1e-6)?
        .dump(format!("src/bin/calibration/offaxis_M2S{SID}_modes.pkl"))?;
    println!("offaxis_M2S{SID}_modes: {calib}");

    let mut calib = Calib::<crseo::gmt::GmtM1, SID>::new(6)
        .guide_star(crseo::Source::builder().size(3).on_ring(6f32.from_arcmin()));
    calib
        .calibrate_rigid_body_motions([
            Some(1e-6),               // Tx
            Some(1e-6),               // Ty
            Some(1e-6),               // Tz
            Some(1f64.from_arcsec()), // Rx
            Some(1f64.from_arcsec()), // Ry
            Some(1f64.from_arcsec()), // Rz
        ])?
        .dump(format!("src/bin/calibration/offaxis_M1S{SID}_rbms.pkl"))?;
    println!("offaxis_M1S{SID}_rbms: {calib}");

    Ok(())
}
