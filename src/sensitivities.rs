//use super::{error::OpticalSensitivitiesError, Matrix, WindLoadedGmtInner};
use crate::ceo;
use bincode;
use nalgebra as na;
use serde::{Deserialize, Serialize};
use skyangle::Conversion;
use std::io;
use std::{env, fs::File, io::BufReader, io::BufWriter, path::Path, time::Instant};
use thiserror::Error;

type Matrix =
    na::Matrix<f64, na::Dynamic, na::Dynamic, na::VecStorage<f64, na::Dynamic, na::Dynamic>>;

#[derive(Error, Debug)]
pub enum OpticalSensitivitiesError {
    #[error("sensitivities file not found (optical_sensitivities.rs.bin)")]
    SensitivityFile(#[from] io::Error),
    #[error("sensitivities cannot be loaded from optical_sensitivities.rs.bin")]
    SensitivityData(#[from] bincode::Error),
    #[error("CFD_REPO environment variable missing")]
    CFDREPO(#[from] env::VarError),
    #[error("segment tip-tilt sensitivity is missing")]
    SegmentTipTilt,
}

/// Optical sensitivities
///
/// Transform M1 and M2 rigid body motions into wavefront and wavefront piston and tip-tilt modes
#[derive(Serialize, Deserialize, Clone)]
pub enum OpticalSensitivities {
    /// Wavefront sensitivity [nx84] where n in the pupil resolution
    Wavefront(Vec<f64>),
    /// Exit pupil tip-tilt sensitivity [2x84]
    TipTilt(Vec<f64>),
    /// Exit pupil segment tip-tilt [14x84]
    SegmentTipTilt(Vec<f64>),
    /// Exit pupil segment piston [7x84]
    SegmentPiston(Vec<f64>),
    SegmentMask(Vec<i32>),
}
impl OpticalSensitivities {
    /// Load precomputed optical sensitivities
    ///
    /// Look in the current directory for the file: "optical_sensitivities.rs.bin"
    pub fn load() -> Result<Vec<Self>, OpticalSensitivitiesError> {
        let data_path = Path::new(".").join("optical_sensitivities.rs.bin");
        println!("loading sensitivities from {:?}", data_path);
        Ok(bincode::deserialize_from(BufReader::with_capacity(
            100_000,
            File::open(data_path)?,
        ))?)
    }
    /// Compute M2 segment tip-tilt sensitivity [14x42]
    pub fn m2_rxy(&self) -> Result<Matrix, OpticalSensitivitiesError> {
        match self {
            OpticalSensitivities::SegmentTipTilt(sens) => {
                let (_, m2_tr) = sens.split_at(14 * 42);
                Ok(na::DMatrix::from_iterator(
                    14,
                    14,
                    m2_tr
                        .chunks(14 * 3)
                        .skip(1)
                        .step_by(2)
                        .flat_map(|x| (&x[..14 * 2]).to_vec())
                        .into_iter(),
                ))
            }
            _ => Err(OpticalSensitivitiesError::SegmentTipTilt),
        }
    }
    /*
       pub fn into_optics(&self, optics_model: &WindLoadedGmtInner) -> Vec<f64> {
           //        let n_sample = optics_model.n_sample;
           let rbm = &optics_model.rbm;
           match self {
               /*OpticalSensitivities::Wavefront(sens) => {
                   let n = sens.len() / 84;
                   //println!("n: {}", n);
                   let sensitivity = na::DMatrix::from_column_slice(n, 84, sens);
                   //let now = Instant::now();
                   let wfe_var = {
                       let n_buf = 1_000;
                       let mut buf = na::DMatrix::<f64>::zeros(n, n_buf);
                       let mut s = 0;
                       let mut var = 0f64;
                       loop {
                           if s + n_buf > n_sample {
                               s -= n_buf;
                               let n_last = n_sample - s;
                               let mut buf = na::DMatrix::<f64>::zeros(n, n_last);
                               buf.gemm(1f64, &sensitivity, &rbm.columns(s, n_last), 0f64);
                               var += buf.row_variance().as_slice().into_iter().sum::<f64>();
                               break var;
                           } else {
                               buf.gemm(1f64, &sensitivity, &rbm.columns(s, n_buf), 0f64);
                               var += buf.row_variance().as_slice().into_iter().sum::<f64>();
                           }
                           s += n_buf;
                       }
                   };
                   let value = 1e9 * (wfe_var / n_sample as f64).sqrt();
                   OpticalWindLoad::Wavefront(value)
                   /*println!(
                       "Wavefront: {:6.0}nm in {:.3}s", value,
                       now.elapsed().as_secs_f64()
                   );*/
               }*/
               OpticalSensitivities::TipTilt(sens) => {
                   let sensitivity = na::DMatrix::from_column_slice(2, 84, sens);
                   let tip_tilt = (sensitivity * rbm).map(|x| x.to_mas());
                   tip_tilt.as_slice().to_owned()
               }
               OpticalSensitivities::SegmentTipTilt(sens) => {
                   let sensitivity = na::DMatrix::from_column_slice(14, 84, sens);
                   let segment_tip_tilt = (sensitivity * rbm).map(|x| x.to_mas());
                   segment_tip_tilt.as_slice().to_owned()
               }
               OpticalSensitivities::SegmentPiston(sens) => {
                   let sensitivity = na::DMatrix::from_column_slice(7, 84, sens);
                   let segment_piston = (sensitivity * rbm).map(|x| x * 1e9);
                   let mut v: Vec<f64> = vec![];
                   for (k, row) in segment_piston.row_iter().take(6).enumerate() {
                       //println!("{}: {:?}", k, row.shape());
                       v.extend(
                           &mut segment_piston
                               .rows(k + 1, 6 - k)
                               .row_iter()
                               .flat_map(|y| (y - row).as_slice().to_owned()),
                       );
                   }
                   segment_piston.as_slice().to_owned()
               }
               _ => unimplemented!(),
           }
       }
       pub fn transform(&self, optics_model: &WindLoadedGmtInner) -> OpticalWindLoad {
           let n_sample = optics_model.n_sample;
           let rbm = &optics_model.rbm;
           match self {
               OpticalSensitivities::Wavefront(sens) => {
                   let n = sens.len() / 84;
                   //println!("n: {}", n);
                   let sensitivity = na::DMatrix::from_column_slice(n, 84, sens);
                   //let now = Instant::now();
                   let wfe_var = {
                       let n_buf = 1_000;
                       let mut buf = na::DMatrix::<f64>::zeros(n, n_buf);
                       let mut s = 0;
                       let mut var = 0f64;
                       loop {
                           if s + n_buf > n_sample {
                               s -= n_buf;
                               let n_last = n_sample - s;
                               let mut buf = na::DMatrix::<f64>::zeros(n, n_last);
                               buf.gemm(1f64, &sensitivity, &rbm.columns(s, n_last), 0f64);
                               var += buf.row_variance().as_slice().into_iter().sum::<f64>();
                               break var;
                           } else {
                               buf.gemm(1f64, &sensitivity, &rbm.columns(s, n_buf), 0f64);
                               var += buf.row_variance().as_slice().into_iter().sum::<f64>();
                           }
                           s += n_buf;
                       }
                   };
                   let value = 1e9 * (wfe_var / n_sample as f64).sqrt();
                   OpticalWindLoad::Wavefront(value)
                   /*println!(
                       "Wavefront: {:6.0}nm in {:.3}s", value,
                       now.elapsed().as_secs_f64()
                   );*/
               }
               OpticalSensitivities::TipTilt(sens) => {
                   let sensitivity = na::DMatrix::from_column_slice(2, 84, sens);
                   let tip_tilt = (sensitivity * rbm).map(|x| x.to_mas());
                   let values = tip_tilt
                       .column_variance()
                       .map(|x| x.sqrt())
                       .as_slice()
                       .to_owned();
                   //println!("TT: {:2.0?}mas", &values);
                   OpticalWindLoad::TipTilt(values)
               }
               OpticalSensitivities::SegmentTipTilt(sens) => {
                   let sensitivity = na::DMatrix::from_column_slice(14, 84, sens);
                   let segment_tip_tilt = (sensitivity * rbm).map(|x| x.to_mas());
                   let values: Vec<_> = segment_tip_tilt
                       .column_variance()
                       .map(|x| x.sqrt())
                       .as_slice()
                       .chunks(7)
                       .map(|x| x.to_owned())
                       .collect();
                   //println!("Segment TT: {:2.0?}mas", values,);
                   OpticalWindLoad::SegmentTipTilt(values)
               }
               OpticalSensitivities::SegmentPiston(sens) => {
                   let sensitivity = na::DMatrix::from_column_slice(7, 84, sens);
                   let segment_piston = (sensitivity * rbm).map(|x| x * 1e9);
                   let mut v: Vec<f64> = vec![];
                   for (k, row) in segment_piston.row_iter().take(6).enumerate() {
                       //println!("{}: {:?}", k, row.shape());
                       v.extend(
                           &mut segment_piston
                               .rows(k + 1, 6 - k)
                               .row_iter()
                               .flat_map(|y| (y - row).as_slice().to_owned()),
                       );
                   }
                   let value = (na::DMatrix::from_vec(n_sample, 21, v)
                       .column_variance()
                       .sum()
                       / n_sample as f64)
                       .sqrt();
                   let values = segment_piston
                       .column_variance()
                       .map(|x| x.sqrt())
                       .as_slice()
                       .to_owned();
                   //println!("Diff. piston std: {:5.0}nm", value,);
                   //println!("Piston: {:3.0?}nm ; ", &values);
                   OpticalWindLoad::Piston([
                       PistonWindLoad::DifferentialSegmentPiston(value),
                       PistonWindLoad::SegmentPiston(values),
                   ])
               }
               OpticalSensitivities::SegmentMask(_) => OpticalWindLoad::WavefrontWoSegmentPiston(None),
           }
       }
    */
    /// Compute all the sensitivities
    ///
    /// Optionally provides an optical model or uses: [`ceo!(GMT)`](crate::GMT) and [`ceo!(SOURCE)`](crate::SOURCE)
    pub fn compute(
        model: Option<(crate::Gmt, crate::Source)>,
    ) -> std::result::Result<Vec<OpticalSensitivities>, Box<dyn std::error::Error>> {
        println!("Computing optical sensitivities ...");
        let now = Instant::now();
        let (mut gmt, mut src) = match model {
            Some(model) => model,
            None => (ceo!(GMT), ceo!(SOURCE)),
        };
        let stroke_fn = |dof| if dof < 3 { 1e-6 } else { 1f64.from_arcsec() };

        let mut tip_tilt = vec![];
        let mut segment_piston = vec![];
        let mut segment_tip_tilt = vec![];
        let mut phase = vec![];
        let n = (src.pupil_sampling * src.pupil_sampling) as usize;
        let mut amplitude = vec![true; n];
        for sid in 0..7 {
            for dof in 0..6 {
                let mut m1_rbm = vec![vec![0.; 6]; 7];
                let stroke = stroke_fn(dof);

                m1_rbm[sid][dof] = stroke;
                gmt.update(Some(&m1_rbm), None, None, None);

                src.through(&mut gmt).xpupil();
                amplitude
                    .iter_mut()
                    .zip(src.amplitude().into_iter())
                    .for_each(|(b, a)| {
                        *b = a > 0f32 && *b;
                    });
                let push_phase = src.phase().to_owned();
                let push_tip_tilt = src.gradients();
                let push_segment_piston = src.segment_piston();
                let push_segment_tip_tilt = src.segments_gradients();

                m1_rbm[sid][dof] = -stroke;
                gmt.update(Some(&m1_rbm), None, None, None);

                src.through(&mut gmt).xpupil();
                amplitude
                    .iter_mut()
                    .zip(src.amplitude().into_iter())
                    .for_each(|(b, a)| {
                        *b = a > 0f32 && *b;
                    });
                phase.extend(
                    src.phase()
                        .to_owned()
                        .into_iter()
                        .zip(push_phase.into_iter())
                        .map(|(l, r)| 0.5f64 * (r as f64 - l as f64) / stroke),
                );
                tip_tilt.extend(
                    src.gradients()
                        .into_iter()
                        .zip(push_tip_tilt.into_iter())
                        .map(|(l, r)| 0.5f64 * (r as f64 - l as f64) / stroke),
                );
                segment_piston.extend(
                    src.segment_piston()
                        .into_iter()
                        .zip(push_segment_piston.into_iter())
                        .map(|(l, r)| 0.5f64 * (r as f64 - l as f64) / stroke),
                );
                segment_tip_tilt.extend(
                    src.segments_gradients()
                        .into_iter()
                        .zip(push_segment_tip_tilt.into_iter())
                        .flat_map(|(left, right)| {
                            left.into_iter()
                                .zip(right.into_iter())
                                .map(|(l, r)| 0.5f64 * (r as f64 - l as f64) / stroke)
                                .collect::<Vec<f64>>()
                        }),
                );
            }
        }
        for sid in 0..7 {
            for dof in 0..6 {
                let mut m2_rbm = vec![vec![0.; 6]; 7];
                let stroke = stroke_fn(dof);

                m2_rbm[sid][dof] = stroke;
                gmt.update(None, Some(&m2_rbm), None, None);

                src.through(&mut gmt).xpupil();
                amplitude
                    .iter_mut()
                    .zip(src.amplitude().into_iter())
                    .for_each(|(b, a)| {
                        *b = a > 0f32 && *b;
                    });
                let push_phase = src.phase().to_owned();
                let push_tip_tilt = src.gradients();
                let push_segment_piston = src.segment_piston();
                let push_segment_tip_tilt = src.segments_gradients();

                m2_rbm[sid][dof] = -stroke;
                gmt.update(None, Some(&m2_rbm), None, None);

                src.through(&mut gmt).xpupil();
                amplitude
                    .iter_mut()
                    .zip(src.amplitude().into_iter())
                    .for_each(|(b, a)| {
                        *b = a > 0f32 && *b;
                    });
                phase.extend(
                    src.phase()
                        .to_owned()
                        .into_iter()
                        .zip(push_phase.into_iter())
                        .map(|(l, r)| 0.5f64 * (r as f64 - l as f64) / stroke),
                );
                tip_tilt.extend(
                    src.gradients()
                        .into_iter()
                        .zip(push_tip_tilt.into_iter())
                        .map(|(l, r)| 0.5f64 * (r as f64 - l as f64) / stroke),
                );
                segment_piston.extend(
                    src.segment_piston()
                        .into_iter()
                        .zip(push_segment_piston.into_iter())
                        .map(|(l, r)| 0.5f64 * (r as f64 - l as f64) / stroke),
                );
                segment_tip_tilt.extend(
                    src.segments_gradients()
                        .into_iter()
                        .zip(push_segment_tip_tilt.into_iter())
                        .flat_map(|(left, right)| {
                            left.into_iter()
                                .zip(right.into_iter())
                                .map(|(l, r)| 0.5f64 * (r as f64 - l as f64) / stroke)
                                .collect::<Vec<f64>>()
                        }),
                );
            }
        }
        let optical_sensitivities = vec![
            OpticalSensitivities::Wavefront(
                phase
                    .chunks(n)
                    .flat_map(|pp| {
                        pp.iter()
                            .zip(amplitude.iter())
                            .filter(|(_, a)| **a)
                            .map(|(p, _)| *p)
                            .collect::<Vec<f64>>()
                    })
                    .collect(),
            ),
            OpticalSensitivities::TipTilt(tip_tilt),
            OpticalSensitivities::SegmentPiston(segment_piston),
            OpticalSensitivities::SegmentTipTilt(segment_tip_tilt),
            OpticalSensitivities::SegmentMask(
                src.segment_mask()
                    .iter()
                    .zip(amplitude.iter())
                    .filter(|(_, a)| **a)
                    .map(|(p, _)| *p)
                    .collect(),
            ),
        ];
        println!(" ... done in {:.3}s", now.elapsed().as_secs_f64());
        Ok(optical_sensitivities)
    }
}
/// Sensitivities serialization into a bincode file
pub trait ToBin {
    fn to_bin(self) -> Result<Self, Box<dyn std::error::Error>>
    where
        Self: Sized;
}
impl ToBin for Vec<OpticalSensitivities> {
    /// Serializes sensitivities
    ///
    /// Saves sensitivities into the file: "optical_sensitivities.rs.bin"
    fn to_bin(self) -> Result<Self, Box<dyn std::error::Error>> {
        //let repo = env::var("CFD_REPO")?;
        let path = Path::new(".");
        bincode::serialize_into(
            BufWriter::with_capacity(
                100_000,
                File::create(path.join("optical_sensitivities.rs.bin"))?,
            ),
            &self,
        )?;
        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sens_m2_rxy() {
        /*let sensitivities = OpticalSensitivities::load()
        .or_else(|_| OpticalSensitivities::compute().unwrap().to_bin())
        .unwrap();*/
        let sensitivities = OpticalSensitivities::compute(None)
            .unwrap()
            .to_bin()
            .unwrap();
        let m2_rxy = sensitivities[3].m2_rxy().unwrap();
        println!("M2 Rxy : {:.3}", m2_rxy);
    }
}
