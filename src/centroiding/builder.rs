use super::Centroiding;
use crate::imaging::ImagingBuilder;
use crate::Builder;

/// Centroiding builder
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CentroidingBuilder {
    n_lenslet: usize,
    n_sensor: usize,
    data_units: f64,
}

impl CentroidingBuilder {
    pub fn n_lenslet(mut self, n_lenslet: usize) -> Self {
        self.n_lenslet = n_lenslet;
        self
    }
    pub fn n_sensor(mut self, n_sensor: usize) -> Self {
        self.n_sensor = n_sensor;
        self
    }
}

impl Default for CentroidingBuilder {
    fn default() -> Self {
        Self {
            n_lenslet: 1,
            n_sensor: 1,
            data_units: 1.,
        }
    }
}

impl From<&ImagingBuilder> for CentroidingBuilder {
    fn from(value: &ImagingBuilder) -> Self {
        Self {
            n_lenslet: value.lenslet_array.n_side_lenslet,
            n_sensor: value.n_sensor as usize,
            ..Default::default()
        }
    }
}

impl Builder for CentroidingBuilder {
    type Component = Centroiding;

    fn build(self) -> crate::Result<Self::Component> {
        let mut cmpt = Centroiding {
            _c_: Default::default(),
            _c_mask_: Default::default(),
            n_lenslet_total: 0,
            n_centroids: 0,
            units: 1f32,
            flux: vec![],
            valid_lenslets: vec![],
            n_valid_lenslet: vec![],
            centroids: vec![],
            xy_mean: None,
        };
        cmpt.n_lenslet_total = self.n_lenslet * self.n_lenslet;
        let n = cmpt.n_lenslet_total * self.n_sensor;
        cmpt.n_valid_lenslet = vec![cmpt.n_lenslet_total; self.n_sensor];
        cmpt.n_centroids = 2 * n;
        unsafe {
            cmpt._c_.setup(self.n_lenslet as i32, self.n_sensor as i32);
            cmpt._c_mask_.setup(n as i32);
        }
        cmpt.flux = vec![0.0; n];
        cmpt.centroids = vec![0.0; cmpt.n_centroids];
        cmpt.units = self.data_units as f32;
        cmpt.valid_lenslets(None, Some(vec![1i8; n]));
        Ok(cmpt)
    }
}
