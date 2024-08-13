use crseo::{prelude::*, pssn::AtmosphereTelescopeError, PSSnEstimates};
use skyangle::Conversion;

fn main() -> anyhow::Result<()> {
    let mut src = Source::builder().build()?;
    let atm_duration = 20f32;
    let atm_n_duration = None;
    let atm_sampling = 48 * 16 + 1;
    let mut atm = Atmosphere::builder()
        .ray_tracing(
            25.5,
            atm_sampling,
            20f32.from_arcmin(),
            atm_duration,
            Some("/fsx/atmosphere/atm_15mn.bin".to_owned()),
            atm_n_duration,
        )
        .build()?;
    let mut gmt = Gmt::builder().build()?;
    let mut pssn: PSSn<AtmosphereTelescopeError> = PSSn::builder().build()?;

    for k in 0..1000 {
        atm.secs = k as f64 * 1e-1;
        src.through(&mut gmt)
            .xpupil()
            .through(&mut atm)
            .through(&mut pssn);
    }
    println!("{:?}", pssn.estimates());

    Ok(())
}
