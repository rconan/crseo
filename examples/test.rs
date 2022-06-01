use crseo::{SourceBuilder,GmtBuilder,Builder};

fn main() {
    let mut src = SourceBuilder::default().build().unwrap();
    let mut gmt = GmtBuilder::default().build().unwrap();
    src.through(&mut gmt).xpupil();
    println!("WFE RMS: {}", src.wfe_rms_10e(-9)[0]);
}
