use bincode;
use crseo::{ceo, Gmt, Source};
use serde::Serialize;
use skyangle::Conversion;
use std::fs::File;
use std::io::BufWriter;

#[derive(Serialize)]
enum OpticalSensitivities {
    Wavefront(Vec<f64>),
    TipTilt(Vec<f64>),
    SegmentTipTilt(Vec<f64>),
    SegmentPiston(Vec<f64>),
}

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut gmt = ceo!(GMT);
    let mut src = ceo!(SOURCE);
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
                    *b = if a > 0f32 && *b { true } else { false };
                });
            let push_phase = src.phase().to_owned();
            let push_tip_tilt = src.gradients();
            let push_segment_piston = src.segment_piston();
            let push_segment_tip_tilt = src.segment_gradients();

            m1_rbm[sid][dof] = -stroke;
            gmt.update(Some(&m1_rbm), None, None, None);

            src.through(&mut gmt).xpupil();
            amplitude
                .iter_mut()
                .zip(src.amplitude().into_iter())
                .for_each(|(b, a)| {
                    *b = if a > 0f32 && *b { true } else { false };
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
                src.segment_gradients()
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
                    *b = if a > 0f32 && *b { true } else { false };
                });
            let push_phase = src.phase().to_owned();
            let push_tip_tilt = src.gradients();
            let push_segment_piston = src.segment_piston();
            let push_segment_tip_tilt = src.segment_gradients();

            m2_rbm[sid][dof] = -stroke;
            gmt.update(None, Some(&m2_rbm), None, None);

            src.through(&mut gmt).xpupil();
            amplitude
                .iter_mut()
                .zip(src.amplitude().into_iter())
                .for_each(|(b, a)| {
                    *b = if a > 0f32 && *b { true } else { false };
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
                src.segment_gradients()
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
    let optical_sensitivities = [
        OpticalSensitivities::Wavefront(
            phase
                .chunks(n)
                .flat_map(|pp| {
                    pp.into_iter()
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
    ];
    bincode::serialize_into(
        BufWriter::with_capacity(
            100_000,
            File::create("examples/optical_sensitivities.rs.bin")?,
        ),
        &optical_sensitivities,
    )?;
    Ok(())
}
