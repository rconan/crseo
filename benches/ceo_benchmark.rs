use std::usize;

use criterion::*;
use crseo::{
    ceo, raytracing::*, shackhartmann::WavefrontSensorBuilder, Builder, Geometric, Gmt,
    ShackHartmann, Source, SH48,
};
use skyangle::Conversion;

#[inline]
fn sh48_ray_tracing(gmt: &mut Gmt, gs: &mut Source) {
    gs.through(gmt).xpupil();
}

#[inline]
fn sh48_wavefront_sensing(gmt: &mut Gmt, gs: &mut Source, wfs: &mut ShackHartmann<Geometric>) {
    gs.through(gmt).xpupil().through(wfs);
}

pub fn sh48_benchmark(c: &mut Criterion) {
    let mut gmt = ceo!(GmtBuilder);
    let mut gs = (1..=4)
        .map(|n_sensor| {
            SH48::<Geometric>::builder()
                .n_sensor(n_sensor)
                .guide_stars(None)
                .on_ring(6f32.from_arcmin())
                .build()
                .unwrap()
        })
        .collect::<Vec<Source>>();
    let mut wfs = (1..=4)
        .map(|n_sensor| SH48::<Geometric>::builder().n_sensor(n_sensor).build().unwrap())
        .collect::<Vec<ShackHartmann<Geometric>>>();
    let mut group = c.benchmark_group("sh48_benchmark");
    for n_sensor in 1..=4 {
        //group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(
            BenchmarkId::new("Ray tracing", n_sensor),
            &n_sensor,
            |b, &n_sensor| b.iter(|| sh48_ray_tracing(&mut gmt, &mut gs[n_sensor - 1])),
        );
        group.bench_with_input(
            BenchmarkId::new("Wavefront sensing", n_sensor),
            &n_sensor,
            |b, &n_sensor| {
                b.iter(|| {
                    sh48_wavefront_sensing(&mut gmt, &mut gs[n_sensor - 1], &mut wfs[n_sensor - 1])
                })
            },
        );
    }
    group.finish();
}

pub fn raytracing_benchmark(c: &mut Criterion) {
    let mut m1 = CONIC::builder()
        .curvature_radius(36.)
        .conic_cst(1. - 0.9982857)
        .build()
        .unwrap();
    let mut m2 = CONIC::builder()
        .curvature_radius(-4.1639009)
        .conic_cst(1. - 0.71692784)
        .conic_origin([0., 0., 20.26247614])
        .build()
        .unwrap();
    let mut rays = RAYS::builder()
        .xy(vec![0., 0.])
        .origin([0., 0., 25.])
        .build()
        .unwrap();
    c.bench_function("raytracing", |b| {
        b.iter(|| {
            rays.into_optics(&mut m1)
                .intersect(&mut m1)
                .reflect()
                .from_optics(&mut m1)
                .into_optics(&mut m2)
                .intersect(&mut m2)
                .reflect()
                .from_optics(&mut m2);
        })
    });
}

pub fn raytracing_vs_n(c: &mut Criterion) {
    let mut m1 = CONIC::builder()
        .curvature_radius(36.)
        .conic_cst(1. - 0.9982857)
        .build()
        .unwrap();
    let mut m2 = CONIC::builder()
        .curvature_radius(-4.1639009)
        .conic_cst(1. - 0.71692784)
        .conic_origin([0., 0., 20.26247614])
        .build()
        .unwrap();
    let rays_vec = (3..=6).map(|log10_n| {
        let n = 10u32.pow(log10_n as u32);
        RAYS::builder()
            .xy([0., 0.].repeat(n as usize))
            .origin([0., 0., 25.])
            .build()
            .unwrap()
    });
    let mut group = c.benchmark_group("raytracing_vs_n");
    let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);
    group.plot_config(plot_config);

    for rays in rays_vec {
        let n = rays.n_ray();
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter(|| {
                let mut rays = RAYS::builder()
                    .xy([0., 0.].repeat(n as usize))
                    .origin([0., 0., 25.])
                    .build()
                    .unwrap();
                rays.into_optics(&mut m1)
                    .intersect(&mut m1)
                    .reflect()
                    .from_optics(&mut m1)
                    .into_optics(&mut m2)
                    .intersect(&mut m2)
                    .reflect()
                    .from_optics(&mut m2);
            })
        });
    }
    group.finish();
}

criterion_group!(benches, raytracing_vs_n);
criterion_main!(benches);
