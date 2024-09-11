use criterion::*;
use crseo::{Builder, FromBuilder, Gmt, Source};

#[inline]
fn ray_tracing_fn(src: &mut Source, gmt: &mut Gmt) {
    src.through(gmt).xpupil();
    // src.wfe_rms_10e(-9);
}

pub fn ray_tracing(c: &mut Criterion) {
    let mut src = Source::builder().build().unwrap();
    let mut gmt = Gmt::builder().build().unwrap();
    c.bench_function("ray tracing", |b| {
        b.iter(|| ray_tracing_fn(&mut src, &mut gmt))
    });
}

criterion_group!(benches, ray_tracing);
criterion_main!(benches);
