use std::mem;

use spade::{
    DelaunayTriangulation, FloatTriangulation, HasPosition, InsertionError, Point2, Triangulation,
};

use crate::{ModesSetError, ModesSets, ModesSetsResult};

type SurfaceTriangulation = DelaunayTriangulation<Surface>;

/// Mode surface representation
#[derive(Debug, Default, Clone)]
pub struct Surface {
    point: [f64; 2],
    pub height: f64,
}
impl HasPosition for Surface {
    type Scalar = f64;

    fn position(&self) -> Point2<Self::Scalar> {
        self.point.into()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TriangulationError {
    #[error("failed to triangulate mode #{mode} from set {set}")]
    Triangulation {
        mode: usize,
        set: usize,
        source: InsertionError,
    },
    #[error("failed to interpolate mode #{0} from set {1}")]
    Interpolation(usize, usize),
}

impl ModesSets {
    /// Interpolates the modes of set `idx` on the regular grid defined by `n_sample` and `width`
    ///
    /// `xy` iterates over the modes coordinates
    pub fn gridding_set(
        &mut self,
        idx: usize,
        xy: impl Iterator<Item = [f64; 2]> + Clone,
    ) -> ModesSetsResult<&mut Self> {
        self.check_set_index(idx)?;
        for (i, mode) in self
            .sets
            .get_mut(&idx)
            .ok_or_else(|| ModesSetError::EmptySet(idx))?
            .iter_mut()
            .enumerate()
        {
            let mut tri = SurfaceTriangulation::new();
            xy.clone()
                .zip(mode.iter_mut())
                .map(|(point, &mut height)| tri.insert(Surface { point, height }))
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| TriangulationError::Triangulation {
                    mode: i,
                    set: idx,
                    source: e,
                })?;
            let d = self.width / (self.n_sample - 1) as f64;
            let interp_mode = (0..self.n_sample * self.n_sample)
                .into_iter()
                .map(|k| {
                    let x = (k / self.n_sample) as f64 * d - 0.5 * self.width;
                    let y = (k % self.n_sample) as f64 * d - 0.5 * self.width;
                    let p = Point2::new(x, y);
                    match self.interpolation_method.clone().unwrap_or_default() {
                        crate::InterpolationMethod::Barycentric => {
                            let byc = tri.barycentric();
                            byc.interpolate(|p| p.data().height, p)
                                .or_else(|| tri.nearest_neighbor(p).map(|v| v.data().height))
                        }
                        crate::InterpolationMethod::NaturalNeighbor => {
                            let nn = tri.natural_neighbor();
                            nn.interpolate(|p| p.data().height, p)
                                .or_else(|| tri.nearest_neighbor(p).map(|v| v.data().height))
                        }
                        crate::InterpolationMethod::NaturalNeighborWithGradients => {
                            let nn = tri.natural_neighbor();
                            let grads = nn.estimate_gradients(|p| p.data().height);
                            nn.interpolate_gradient(|p| p.data().height, &grads, 1.0, p)
                                .or_else(|| tri.nearest_neighbor(p).map(|v| v.data().height))
                        }
                    }
                })
                .collect::<Option<Vec<_>>>()
                .ok_or(TriangulationError::Interpolation(i, idx))?;
            let _ = mem::replace(mode, interp_mode);
        }
        Ok(self)
    }
    /// Interpolates all the modes of all the sets on the regular grid defined by `n_sample` and `width`
    ///
    /// `xy` iterates over the modes coordinates
    pub fn gridding(
        &mut self,
        xy: impl Iterator<Item = [f64; 2]> + Clone,
    ) -> ModesSetsResult<&mut Self> {
        let keys: Vec<_> = self.sets.keys().cloned().collect();
        for idx in keys {
            self.gridding_set(idx, xy.clone())?;
        }
        Ok(self)
    }
}
