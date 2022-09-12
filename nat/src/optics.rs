use crseo::{Conic, Rays, CONIC};

#[derive(Debug, thiserror::Error)]
pub enum OpticsError {
    CRSEO(#[from] crseo::Error),
}

pub struct OpticalSystem {
    optics: Vec<Conic>,
}
impl OpticalSystem {
    pub fn gmt() -> Self {
        let mut m1 = CONIC::new()
            .curvature_radius(36.)
            .conic_cst(1. - 0.9982857)
            .build()
            .unwrap();
        let mut m2 = CONIC::new()
            .curvature_radius(-4.1639009)
            .conic_cst(1. - 0.71692784)
            .conic_origin([0., 0., 20.26247614])
            .build()
            .unwrap();
        Self {
            optics: vec![m1, m2],
        }
    }
    pub fn trace(&mut self, rays: &mut Rays) {
        self.optics.iter_mut().maps(|optic| {
            rays.into_optics(optic)
                .intersect(optic)
                .reflect()
                .from_optics(optic)
        })
    }
}
