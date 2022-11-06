use complot::{tri, Config};
use crseo::{raytracing::Rays, Builder, FromBuilder};
use linya::{Bar, Progress};
use nat::{Mesh, OpticalSystem, OPD};
use skyangle::SkyAngle;
use std::time::Instant;

fn main() -> anyhow::Result<()> {
    let rays_origin = [0., 0., 5.];
    let pupil_diameter = 25.5;
    let entrance_pupil = Mesh::new(pupil_diameter, 0.3).build();
    let _: tri::Mesh = (
        entrance_pupil.triangle_vertex_iter(),
        Some(Config::new().filename("entrance_pupil.png")),
    )
        .into();

    let mut rays = Rays::builder()
        .xy(entrance_pupil.vertex_iter().flatten().cloned().collect())
        .origin(rays_origin)
        .build()?;

    let m1_origin = Some([0., 0., 0.]);
    let m1_euler = None; //Some([0., SkyAngle::Arcsecond(6f64 * 0.1).to_radians(), 0.]);
    let m2_origin = Some([0., 0., 20.26247614]);
    let m2_euler = None; //Some([0., SkyAngle::Arcsecond(6f64 * 0.1).to_radians(), 0.]);

    let mut sys = OpticalSystem::gmt(m1_origin, m1_euler, m2_origin, m2_euler)?;
    let now = Instant::now();
    let opd = sys.trace(&mut rays);
    println!("Rays tracing in {}micros", now.elapsed().as_micros());

    let (opd_mean, opd_std) = opd.stats();
    println!("OPD mean: {:.3}nm", opd_mean * 1e9);
    println!("OPD std : {:.3}nm", opd_std * 1e9);
    opd.to_pickle("on-axis.pkl")?;

    let mut sys = OpticalSystem::gmt(m1_origin, m1_euler, m2_origin, m2_euler)?;

    for k in 0..5 {
        let mut rays = Rays::builder()
            .zenith(SkyAngle::<f64>::Arcminute((2 * k) as f64).to_radians())
            .xy(entrance_pupil.vertex_iter().flatten().cloned().collect())
            .origin(rays_origin)
            .build()?;

        let opd = sys.trace(&mut rays);
        let zern: Vec<f64> = opd.zproj(4).into_iter().map(|x| x * 1e9).collect();
        println!("Zern({}): {:+6.0?}", k * 2, zern);
        opd.to_pickle("off-axis.pkl")?;
    }

    let now = Instant::now();
    let field = Mesh::new(20f64, 0.1).build();

    let mut progress = Progress::new();
    let bar: Bar = progress.bar(field.n_vertices() as usize, "Ray tracing");

    let mut x: Vec<f64> = Vec::with_capacity(field.n_vertices());
    let mut y: Vec<f64> = Vec::with_capacity(field.n_vertices());
    let mut astigmatism_rss: Vec<f64> = Vec::with_capacity(field.n_vertices());
    let mut coma_rss: Vec<f64> = Vec::with_capacity(field.n_vertices());
    let mut spherical_rss: Vec<f64> = Vec::with_capacity(field.n_vertices());
    for xy in field.vertex_iter() {
        progress.inc_and_draw(&bar, 1);
        let (r, o) = (xy[0].hypot(xy[1]), xy[1].atan2(xy[0]));
        let mut rays = Rays::builder()
            .zenith(SkyAngle::<f64>::Arcminute(r).to_radians())
            .azimuth(o)
            .xy(entrance_pupil.vertex_iter().flatten().cloned().collect())
            .origin(rays_origin)
            .build()?;
        let opd = sys.trace(&mut rays);
        let zern: Vec<f64> = opd.zproj(4).into_iter().map(|x| x * 1e9).collect();
        x.push(xy[0]);
        y.push(xy[1]);
        astigmatism_rss.push(zern[4].hypot(zern[5]));
        coma_rss.push(zern[6].hypot(zern[7]));
        spherical_rss.push(zern[10]);
    }
    let field_a56 = OPD::new(&x, &y, &vec![], astigmatism_rss);
    field_a56.to_pickle("field_a56.pkl")?;
    let field_a78 = OPD::new(&x, &y, &vec![], coma_rss);
    field_a78.to_pickle("field_a78.pkl")?;
    let field_a11 = OPD::new(&x, &y, &vec![], spherical_rss);
    field_a11.to_pickle("field_a11.pkl")?;
    println!("Elpased time: {}ms", now.elapsed().as_millis());

    /*     let _: tri::Heatmap = (
           entrance_pupil.triangle_vertex_iter().zip(opd.into_iter()),
           Some(Config::new().filename("opd.png")),
       )
           .into();
    */
    /*     let entrance_pupil = Mesh::new(8., 1.).build();
    let vertices: Vec<f64> = entrance_pupil.vertex_iter().flatten().cloned().collect();
    // dbg!(&vertices[..70]);

    let mut rays = Rays::builder()
        .xy(vertices[..70].to_vec())
        .origin([0., 0., 5.])
        .build()?;
    println!("Chief coordinates: {:?}", rays.chief_coordinates());
    println!("Chief directions: {:?}", rays.chief_directions());

    let mut sys = OpticalSystem::gmt()?;
    let opd = sys.trace(&mut rays);

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
    println!(
        "Opd [{:}]:
    {:?}",
        opd.len(),
        opd
    ); */

    /*
        <<ray bundle functions>>=
    void bundle::to_sphere(rtd z_chief_on_axis, rtd rho_focal_plane)
    {
      fprintf(stdout, "N_BUNDLE: %d\n", N_BUNDLE);
      dim3 blockDim(1,1);
      dim3 gridDim(1,1, N_BUNDLE);
      <<reference sphere distance>>
      fprintf(stdout, "sphere distance: %f\n", d__sphere_distance);
      <<reference sphere origin>>
      vector *sphere_origin;
      sphere_origin = (vector*)malloc(sizeof(vector));
      HANDLE_ERROR( cudaMemcpy( sphere_origin, d__sphere_origin,sizeof(vector),cudaMemcpyDeviceToHost ) );
      fprintf(stdout, "sphere origin: %f , %f, %f\n", sphere_origin->x,sphere_origin->y,sphere_origin->z);
      free(sphere_origin);
      rtd *l__sphere_radius;
      HANDLE_ERROR( cudaMalloc((void**)&l__sphere_radius,
    sizeof(rtd)*N_BUNDLE ) );
      sphere_radius_kernel LLL gridDim , blockDim RRR (l__sphere_radius, d__chief_ray, 1,
                             d__sphere_origin);
      rtd *sphere_radius;
        sphere_radius = (rtd*)malloc(sizeof(rtd));
      HANDLE_ERROR( cudaMemcpy( sphere_radius, l__sphere_radius,sizeof(rtd),cudaMemcpyDeviceToHost ) );
      fprintf(stdout, "sphere radius: %f\n", sphere_radius
      );
      free(sphere_radius);
      <<chief ray optical path length>>
      <<rays optical path difference>>
    } */
    Ok(())
}
