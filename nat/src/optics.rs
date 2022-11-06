use crseo::{
    raytracing::{Conic, ConicBuilder, Rays, RaysBuilder},
    Builder, FromBuilder,
};

use crate::opd::OPD;

#[derive(Debug, thiserror::Error)]
pub enum OpticsError {
    #[error("failed to instanciate CEO object")]
    CRSEO(#[from] crseo::CrseoError),
}
pub type Result<T> = std::result::Result<T, OpticsError>;

pub struct OpticalSystem {
    optics: Vec<Conic>,
}
impl OpticalSystem {
    pub fn gmt(
        m1_origin: Option<[f64; 3]>,
        m1_euler: Option<[f64; 3]>,
        m2_origin: Option<[f64; 3]>,
        m2_euler: Option<[f64; 3]>,
    ) -> Result<Self> {
        let m1 = Conic::builder()
            .curvature_radius(36.)
            .conic_cst(1. - 0.9982857)
            .origin(m1_origin.unwrap_or([0f64; 3]))
            .euler_angles(m1_euler.unwrap_or([0f64; 3]))
            .build()?;
        let m2 = Conic::builder()
            .curvature_radius(-4.1639009)
            .conic_cst(1. - 0.71692784)
            .origin(m2_origin.unwrap_or([0., 0., 20.26247614]))
            .euler_angles(m2_euler.unwrap_or([0f64; 3]))
            .build()?;
        Ok(Self {
            optics: vec![m1, m2],
        })
    }
    pub fn trace(&mut self, rays: &mut Rays) -> OPD {
        self.optics.iter_mut().for_each(|optic| {
            optic.trace(rays);
        });
        rays.to_sphere(-5.830, 2.197173);
        rays.into()
    }
}
