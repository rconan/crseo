use crseo::ceo;

fn main() {
    let mut gmt = ceo!(GmtBuilder);
    let mut src = ceo!(SourceBuilder);
    src.through(&mut gmt).xpupil();
    println!("WFE RMS: {}", src.wfe_rms_10e(-9)[0]);
}
