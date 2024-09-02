use std::{f32::consts::PI, iter::repeat};

use serde::{Deserialize, Serialize};

use super::SegmentPistonSensor;

const O: [f32; 12] = [
    0.,
    -PI / 3.,
    0.,
    -PI / 3.,
    PI / 3.,
    -PI / 3.,
    PI / 3.,
    -PI / 3.,
    PI / 3.,
    0.,
    PI / 3.,
    0.,
];

#[derive(Debug, Clone, Serialize)]
pub struct Fftlet {
    x: Vec<f32>,
    y: Vec<f32>,
    image: Vec<f32>,
}

impl Fftlet {
    pub fn intercept(&self) -> f32 {
        let (s, sy) = self
            .x
            .iter()
            .zip(self.y.iter())
            .zip(self.image.iter())
            .fold((0f32, 0f32), |(mut s, mut sy), ((x, y), i)| {
                s += i;
                sy += i * y * x.signum();
                (s, sy)
            });
        sy / s
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct DispersedFringeSensor {
    data: Vec<Vec<f32>>,
    n: usize,
    threshold: Option<f32>,
    mask_radius: Option<f32>,
    intercept: Vec<f32>,
    reference: Option<Vec<f32>>,
}

impl DispersedFringeSensor {
    pub fn set_reference(&mut self, dfs: &DispersedFringeSensor) -> &mut Self {
        self.reference = Some(dfs.intercept.clone());
        self
    }
    pub fn threshold(self, t: f64) -> Self {
        Self {
            threshold: Some(t as f32),
            ..self
        }
    }
}

impl From<&mut SegmentPistonSensor> for DispersedFringeSensor {
    fn from(sps: &mut SegmentPistonSensor) -> Self {
        let mut frame = sps.fft();
        let n = frame.resolution;
        let q = n / 4;
        let data = Vec::<f32>::from(&mut frame);
        let mut chop_data = vec![];
        for i in 0..4 {
            for j in 0..3 {
                chop_data.push(
                    data.chunks(n)
                        .skip(i * q)
                        .take(q)
                        .flat_map(|data| {
                            data.iter().skip(j * q).take(q).cloned().collect::<Vec<_>>()
                        })
                        .collect::<Vec<_>>(),
                )
            }
        }
        Self {
            data: chop_data,
            n: q,
            threshold: Some(0.2),
            mask_radius: Some(0.05),
            ..Default::default()
        }
    }
}

impl DispersedFringeSensor {
    pub fn flux(&self) -> Vec<f32> {
        self.data.iter().map(|data| data.iter().sum()).collect()
    }

    pub fn xy(&self, i: usize) -> impl Iterator<Item = (f32, f32)> {
        let n = self.n;

        let x = (0..n)
            .flat_map(move |i| repeat(i).take(n))
            .map(move |x| (x as f32 - 0.5 * (n - 1) as f32) / (n - 1) as f32);
        let y = (0..n)
            .cycle()
            .take(n * n)
            .map(move |x| (x as f32 - 0.5 * (n - 1) as f32) / (n - 1) as f32);

        x.zip(y).map(move |(x, y)| {
            let (so, co) = O[i].sin_cos();
            (co * x - so * y, so * x + co * y)
        })
    }
    pub fn fftlet(&self, i: usize, radius: Option<f32>, threshold: Option<f32>) -> Fftlet {
        // let flux = self.flux()[i];
        let ((x, y), image): ((Vec<f32>, Vec<f32>), Vec<f32>) = if let Some(r) = radius {
            self.xy(i)
                .zip(self.data[i].iter())
                .filter_map(|((x, y), data)| {
                    if x.hypot(y) > r {
                        Some(((x, y), data))
                    } else {
                        None
                    }
                })
                .unzip()
        } else {
            (
                self.xy(i).unzip(),
                self.data[i].iter().map(|i| *i).collect(),
            )
        };
        if let Some(t) = threshold {
            let max_intensity = image
                .iter()
                .max_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap()
                * t;
            let ((x, y), image): ((Vec<f32>, Vec<f32>), Vec<f32>) = x
                .into_iter()
                .zip(y.into_iter())
                .zip(image.into_iter())
                .filter_map(|((x, y), image)| {
                    if image > max_intensity {
                        Some(((x, y), image))
                    } else {
                        None
                    }
                })
                .unzip();
            Fftlet { x, y, image }
        } else {
            Fftlet { x, y, image }
        }
    }
    pub fn intercept(&mut self) -> &mut Self {
        self.intercept = (0..12)
            .map(|i| {
                let fftlet = self.fftlet(i, self.mask_radius, self.threshold);
                fftlet.intercept()
            })
            .collect();
        if let Some(r) = &self.reference {
            self.intercept
                .iter_mut()
                .zip(r.iter())
                .for_each(|(i, r)| *i -= r);
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use std::{error::Error, fs::File, io::BufWriter, time::Instant};

    use crate::{gmt::MirrorGetSet, Builder, FromBuilder, Gmt, SegmentPistonSensor, Source};

    use super::DispersedFringeSensor;

    #[test]
    fn dfs_tz() -> Result<(), Box<dyn Error>> {
        let mut gmt = Gmt::builder().build()?;
        let src_builder = Source::builder().band("J");

        let mut sps = SegmentPistonSensor::builder()
            .src(src_builder.clone())
            .nyquist_factor(3.)
            .build()?;

        let mut src = src_builder.build()?;

        src.through(&mut gmt).xpupil().through(&mut sps);
        let mut dfs0 = DispersedFringeSensor::from(&mut sps).threshold(0.01);
        dfs0.intercept();

        // serde_pickle::to_writer(&mut File::create("DFS0.pkl")?, &dfs0, Default::default())?;

        let mut data = vec![];

        let now = Instant::now();
        for tz in -10..11 {
            let mut tr_xyz = [0f64; 6];
            tr_xyz[2] = tz as f64 * 1e-6;
            gmt.m1.set_rigid_body_motions(7, &tr_xyz);

            src.through(&mut gmt).xpupil().through(sps.reset());
            let mut dfs = DispersedFringeSensor::from(&mut sps).threshold(0.01);
            dfs.set_reference(&dfs0).intercept();

            data.push((tz, dfs));
        }
        println!("elasped {:.3?}", now.elapsed());
        let mut buffer = BufWriter::new(File::create("DFS_tz.pkl")?);
        serde_pickle::to_writer(&mut buffer, &data, Default::default())?;
        Ok(())
    }

    #[test]
    fn fftlet() -> Result<(), Box<dyn Error>> {
        let mut gmt = Gmt::builder().build()?;
        let src_builder = Source::builder().band("J");

        let mut sps = SegmentPistonSensor::builder()
            .src(src_builder.clone())
            .nyquist_factor(3.)
            .build()
            .unwrap();

        let mut src = src_builder.build()?;

        let mut tr_xyz = [0f64; 6];
        tr_xyz[2] = -10e-6;
        //tr_xyz[4] = tz as f64 * 100f64.from_mas();
        gmt.m1.set_rigid_body_motions(1, &tr_xyz);

        src.through(&mut gmt).xpupil().through(&mut sps);

        let mut dfs: DispersedFringeSensor = (&mut sps).into();

        println!("{:+6.4?}", dfs.intercept());

        let fftlet = dfs.fftlet(10, Some(0.05), Some(0.2));

        let mut buffer = BufWriter::new(File::create("fftlet.pkl")?);
        serde_pickle::to_writer(&mut buffer, &fftlet, Default::default())?;

        let mut buffer = BufWriter::new(File::create("dfs.pkl")?);
        serde_pickle::to_writer(&mut buffer, &dfs, Default::default())?;

        Ok(())
    }
}
