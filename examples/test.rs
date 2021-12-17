use crseo::ceo;

fn main() {
    let mut gmt = ceo!(GMT);
    let mut src = ceo!(SOURCE);
    src.through(&mut gmt).xpupil();
    println!("WFE RMS: {}", src.wfe_rms_10e(-9)[0]);
}
