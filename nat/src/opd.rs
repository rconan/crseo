use std::{fs::File, path::Path};

use crseo::raytracing::Rays;
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum OpdError {
    #[error("failed to create a new file")]
    File(#[from] std::io::Error),
    #[error("failed to serialize OPD to Pickle")]
    Pickle(#[from] serde_pickle::Error),
}
pub type Result<T> = std::result::Result<T, OpdError>;

#[derive(Serialize)]
pub struct OPD {
    x: Vec<f64>,
    y: Vec<f64>,
    z: Vec<f64>,
    opd: Vec<f64>,
}
impl From<&mut Rays> for OPD {
    fn from(rays: &mut Rays) -> Self {
        let (x, (y, z)): (Vec<f64>, (Vec<f64>, Vec<f64>)) = rays
            .coordinates()
            .chunks(3)
            .map(|xyz| (xyz[0], (xyz[1], xyz[2])))
            .unzip();
        let opd = rays.optical_path_difference();
        Self { x, y, z, opd }
    }
}
impl OPD {
    pub fn new(x: &Vec<f64>, y: &Vec<f64>, z: &Vec<f64>, opd: Vec<f64>) -> Self {
        Self {
            x: x.to_owned(),
            y: y.to_owned(),
            z: z.to_owned(),
            opd,
        }
    }
    pub fn stats(&self) -> (f64, f64) {
        let opd_mean: f64 = self.opd.iter().cloned().sum::<f64>() / self.opd.len() as f64;
        let opd_std = (self
            .opd
            .iter()
            .map(|x| x - opd_mean)
            .map(|x| x * x)
            .sum::<f64>()
            / self.opd.len() as f64)
            .sqrt();
        (opd_mean, opd_std)
    }
    pub fn to_pickle<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        serde_pickle::to_writer(&mut File::create(path.as_ref())?, self, Default::default())?;
        Ok(())
    }
    pub fn zproj(&self, n: usize) -> Vec<f64> {
        // let j_max = index.iter().fold(usize::MIN, |a, &b| a.max(b)) as f64;
        // let n = (0.5 * ((9. + 8. * (j_max - 1.)).sqrt() - 3.)).round();
        let (j, n, m) = zernike::jnm(n as u32 + 1);
        let (mut r, o): (Vec<f64>, Vec<f64>) = self
            .x
            .iter()
            .zip(&self.y)
            .map(|(x, y)| (x.hypot(*y), y.atan2(*x)))
            .unzip();
        let ir_max = r.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b)).recip();
        r.iter_mut().for_each(|r| *r *= ir_max);
        j.into_iter()
            .zip(n.into_iter())
            .zip(m.into_iter())
            .map(|((j, n), m)| {
                let (p, n) = r
                    .iter()
                    .zip(&o)
                    .map(|(&r, &o)| zernike::zernike(j, n, m, r, o))
                    .zip(&self.opd)
                    .fold((0f64, 0f64), |(mut a, mut b), (z, &o)| {
                        a += o * z;
                        b += z * z;
                        (a, b)
                    });
                p / n
            })
            .collect()
    }
}
