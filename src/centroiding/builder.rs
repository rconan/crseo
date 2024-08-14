use crate::Builder;

use super::Centroiding;

/// Centroiding builder
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CentroidingBuilder {
    n_lenslet: usize,
    data_units: f64,
}

impl Default for CentroidingBuilder {
    fn default() -> Self {
        Self {
            n_lenslet: 1,
            data_units: 1.,
        }
    }
}

impl Builder for CentroidingBuilder {
    type Component = Centroiding;

    fn build(self) -> crate::Result<Self::Component> {
        let mut cmpt = Centroiding {
            _c_: Default::default(),
            _c_mask_: Default::default(),
            n_lenslet_total: 0u32,
            n_centroids: 0u32,
            units: 1f32,
            flux: vec![],
            valid_lenslets: vec![],
            n_valid_lenslet: 0u32,
            centroids: vec![],
        };
        cmpt.n_lenslet_total = (self.n_lenslet * self.n_lenslet) as u32;
        cmpt.n_centroids = 2 * cmpt.n_lenslet_total;
        cmpt.n_valid_lenslet = cmpt.n_lenslet_total;
        unsafe {
            cmpt._c_.setup(self.n_lenslet as i32, 1);
            cmpt._c_mask_.setup(cmpt.n_lenslet_total as i32);
        }
        cmpt.flux = vec![0.0; cmpt.n_lenslet_total as usize];
        cmpt.centroids = vec![0.0; cmpt.n_centroids as usize];
        cmpt.units = self.data_units as f32;
        Ok(cmpt)
    }
}
