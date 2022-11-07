use std::time::Instant;

use crseo::{
    raytracing::{Conic, Rays},
    Builder, FromBuilder,
};

fn main() -> anyhow::Result<()> {
    /*     let entrance_pupil = Mesh::new(8., 1.).build();
    let _: tri::Mesh = (
        entrance_pupil.triangle_vertex_iter(),
        Some(Config::new().filename("entrance_pupil.png")),
    )
        .into();

    let mut rays = Rays::builder()
        .xy(entrance_pupil.vertex_iter().flatten().cloned().collect())
        .origin([0., 0., 25.])
        .build()?;

    let mut sys = OpticalSystem::gmt()?;
    let now = Instant::now();
    let opd = sys.trace(&mut rays);
    println!("Rays tracing in {}micros", now.elapsed().as_micros());

    println!("{:?}", opd);

    let _: tri::Heatmap = (
        entrance_pupil.triangle_vertex_iter().zip(opd.into_iter()),
        Some(Config::new().filename("opd.png")),
    )
        .into(); */

    let mut rays = Rays::builder()
        // .xy(vertices[..70].to_vec())
        .origin([0., 0., 5.])
        .build()?;
    let mut conic = Conic::builder().build()?;
    let mut m1 = Conic::builder()
        .curvature_radius(36.)
        .conic_cst(1. - 0.9982857)
        .build()?;

    let mut m2 = Conic::builder()
        .curvature_radius(-4.1639009)
        .conic_cst(1. - 0.71692784)
        .conic_origin([0., 0., 20.26247614])
        .build()?;

    m1.trace(&mut rays);
    m2.trace(&mut rays);
    rays.to_sphere(-5.830, 2.197173);

    //sys.trace(&mut rays);
    //rays.to_z_plane(18.);
    let locs_dirs: Vec<_> = rays
        .coordinates()
        .chunks(3)
        .zip(rays.directions().chunks(3))
        .map(|(l, d)| l.iter().chain(d.iter()).cloned().collect::<Vec<f64>>())
        .collect();
    println!(
        "Rays locations/directions
    {:#?}",
        locs_dirs
    );
    let opd = rays.optical_path_difference();
    println!(
        "Opd [{:}]:
    {:?}",
        opd.len(),
        opd
    );

    Ok(())
}
