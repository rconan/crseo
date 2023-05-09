use std::time::Instant;

use crseo::{Builder, GmtBuilder, SourceBuilder};

fn main() {
    let mut src = SourceBuilder::default().build().unwrap();
    let mut gmt = GmtBuilder::default().build().unwrap();
    let now = Instant::now();
    src.through(&mut gmt).xpupil();
    println!("Ray tracing in {}ms", now.elapsed().as_millis());
    println!("WFE RMS: {}", src.wfe_rms_10e(-9)[0]);
}
