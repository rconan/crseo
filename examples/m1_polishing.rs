use crseo::ceo;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let m1_n_mode = 28;
    let mut gmt = ceo!(
        GMT,
        m1 = ["m1_eigen-modes-27_raw-polishing", m1_n_mode],
        m1_default_state = [(0..7)
            .flat_map(|_| {
                let mut a1 = vec![0f64; m1_n_mode];
                a1[m1_n_mode - 1] = 1f64;
                a1
            })
            .collect()]
    );
    let mut src = ceo!(SOURCE);

    /*
        let mut a1: Vec<_> = (0..7)
            .flat_map(|k| {
                let mut a1 = vec![0f64; m1_n_mode - 1];
                a1[k] = 1e-5;
                a1
            })
            .collect();
        gmt.m1_modes(&mut a1);
    */
    src.through(&mut gmt).xpupil();
    println!("WFE RMS: {:?}nm", src.wfe_rms_10e(-9));
    let phase: Vec<_> = src.phase().iter().map(|&x| x as f64 * 1e6).collect();
    let _: complot::Heatmap = ((phase.as_slice(), (512, 512)), None).into();
    Ok(())
}
