use std::time::Instant;

use crseo::{Builder, FromBuilder, Gmt, Source};

#[test]
fn main() {
    let mut src = Source::builder().build().unwrap();
    let mut gmt = Gmt::builder().build().unwrap();
    let now = Instant::now();
    src.through(&mut gmt).xpupil();
    println!("Ray tracing in {}ms", now.elapsed().as_millis());
    let wfe = src.wfe_rms_10e(-9)[0];
    println!("WFE RMS: {}", wfe);
    assert!((wfe - 0.7966555).abs() < 1e-3);
}
