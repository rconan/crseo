use complot::{tri, Config};
use crseo::RAYS;
use nat::Mesh;

fn main() -> anyhow::Result<()> {
    let entrance_pupil = Mesh::new(8., 0.25).build();
    let _: tri::Mesh = (
        entrance_pupil.triangle_vertex_iter(),
        Some(Config::new().filename("entrance_pupil.png")),
    )
        .into();

    let mut rays = RAYS::new()
        .xy(entrance_pupil.vertex_iter().flatten().collect())
        .origin([0., 0., 25.])
        .build()?;
}
