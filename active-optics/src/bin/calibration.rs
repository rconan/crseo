#[deny(unused_imports)]
use active_optics::{Calib, SID};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(all(feature = "m2", feature = "on-axis", feature = "modes"))]
    {
        let mut calib = Calib::<crseo::gmt::GmtM2, SID>::new(active_optics::M2_N_MODE);
        calib.calibrate_segment_modes(1e-6)?.dump("calib_m2.pkl")?;
        println!("{calib}")
    }

    #[cfg(all(feature = "m1", feature = "on-axis", feature = "rbms"))]
    {
        use skyangle::Conversion;
        let mut calib = Calib::<crseo::gmt::GmtM1, SID>::new(2);
        calib
            .calibrate_rigid_body_motions([
                None,                     // Tx
                None,                     // Ty
                None,                     // Tz
                Some(1f64.from_arcsec()), // Rx
                Some(1f64.from_arcsec()), // Ry
                None,                     // Rz
            ])?
            .dump("calib_m1_rbms.pkl")?;
        println!("{calib}")
    }

    #[cfg(all(feature = "m2", feature = "off-axis", feature = "modes"))]
    {
        use crseo::FromBuilder;
        use skyangle::Conversion;
        let mut calib = Calib::<crseo::gmt::GmtM2, SID>::new(active_optics::M2_N_MODE)
            .guide_star(crseo::Source::builder().size(3).on_ring(6f32.from_arcmin()));
        calib
            .calibrate_segment_modes(1e-6)?
            .dump("calib_offaxis_m2.pkl")?;
        println!("{calib}")
    }

    #[cfg(all(feature = "m1", feature = "off-axis", feature = "rbms"))]
    {
        use crseo::FromBuilder;
        use skyangle::Conversion;
        let mut calib = Calib::<crseo::gmt::GmtM1, SID>::new(2)
            .guide_star(crseo::Source::builder().size(3).on_ring(6f32.from_arcmin()));
        calib
            .calibrate_rigid_body_motions([
                None,                     // Tx
                None,                     // Ty
                None,                     // Tz
                Some(1f64.from_arcsec()), // Rx
                Some(1f64.from_arcsec()), // Ry
                None,                     // Rz
            ])?
            .dump("calib_offaxis_m1_rbms.pkl")?;
        println!("{calib}")
    }

    Ok(())
}
