use crseo::{
    Builder, FromBuilder, PSSn, PSSnEstimates, Propagation, Source, gmt::ZernikeGmt,
    pssn::TelescopeError,
};

const N_PX: usize = 1024;

fn main() -> anyhow::Result<()> {
    let src_b = Source::builder().band("Vs").pupil_sampling(N_PX);
    let mut gmt = ZernikeGmt::builder()
        .m1_radial_order(0)
        .m2_radial_order(4)
        .build()?;
    println!("{gmt}");
    let mut pssn = PSSn::<TelescopeError>::builder()
        .source(src_b.clone())
        .build()?;
    println!("{pssn}");
    let mut src = src_b.build()?;
    println!("{src}");

    let mut eval_pssn = |j: usize, za: f64| {
        let mut a = vec![0f64; 7 * gmt.m2.n_mode];
        a.chunks_mut(gmt.m2.n_mode).for_each(|a| a[j] = za * 1e-9);
        gmt.m2_modes(&a);

        src.through(&mut gmt).xpupil().through(&mut pssn);
        pssn.peek();
        println!(
            "Zernike #{j} RMS={:.1}nm - WFE RMS={:.0}nm - PSSN={:.5}",
            za,
            src.wfe_rms_10e(-9)[0],
            pssn.estimates()[0]
        );
    };
    eval_pssn(5, 51.);
    eval_pssn(7, 24.5);
    eval_pssn(9, 37.);

    Ok(())
}
