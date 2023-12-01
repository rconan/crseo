use crseo::{
    wavefrontsensor::{LensletArray, Pyramid, PyramidBuilder},
    Builder, FromBuilder, Gmt, WavefrontSensor, WavefrontSensorBuilder,
};
use std::io::{self, Write};

pub struct PyramidPiston {
    pub mask: (Vec<bool>, Vec<bool>),
    sxy0: (Vec<f32>, Vec<f32>),
    calibration: nalgebra::DMatrix<f32>,
    pseudo_inverse: nalgebra::DMatrix<f32>,
}

impl PyramidPiston {
    pub fn new<'a>(
        pym: PyramidBuilder,
        masks: impl Iterator<Item = Option<&'a nalgebra::DMatrix<bool>>>,
    ) -> anyhow::Result<Self> {
        let mut gmt = Gmt::builder().m2("ASM_DDKLs_S7OC04184_675kls", 1).build()?;
        let mut src = pym.guide_stars(None).build()?;
        let mut pym = pym.build()?;

        let LensletArray { n_side_lenslet, .. } = pym.lenslet_array;

        let mask = masks.filter_map(|mask| mask.map(|x| x.clone())).fold(
            vec![true; n_side_lenslet * n_side_lenslet],
            |mut a, m| {
                a.iter_mut().zip(m.iter()).for_each(|(a, m)| {
                    *a = *a & !m;
                });
                a
            },
        );

        src.through(&mut gmt).xpupil().through(&mut pym);
        let (sx0, sy0) = pym.processing();
        pym.reset();

        let mut stdout = io::stdout().lock();
        let mut poke_matrix = vec![];
        let stroke0 = 25e-9;
        let mut m2_segment_coefs = vec![0f64; 1];

        let n_segment = 6;

        print!("Piston calibration: ");
        for sid in 1..=n_segment {
            stdout.write(format!("{sid} ").as_bytes())?;
            stdout.flush()?;

            m2_segment_coefs[0] = stroke0;
            gmt.m2_segment_modes(sid, &m2_segment_coefs);
            pym.reset();
            src.through(&mut gmt).xpupil().through(&mut pym);
            let (mut sx, mut sy) = pym.processing();

            m2_segment_coefs[0] = -stroke0;
            gmt.m2_segment_modes(sid, &m2_segment_coefs);
            pym.reset();
            src.through(&mut gmt).xpupil().through(&mut pym);
            let (_sx, _sy) = pym.processing();

            let q = (0.5 / stroke0) as f32;
            sx -= _sx;
            sx *= q;
            poke_matrix.push(sx.as_slice().to_vec());

            sy -= _sy;
            sy *= q;
            poke_matrix.push(sy.as_slice().to_vec());

            m2_segment_coefs[0] = 0f64;
            gmt.m2_segment_modes(sid, &m2_segment_coefs);
        }
        println!("");

        let mut sx_mask = vec![false; n_side_lenslet * n_side_lenslet];
        let mut sy_mask = vec![false; n_side_lenslet * n_side_lenslet];

        for sxy in poke_matrix.chunks(2) {
            let max = sxy[0]
                .iter()
                .zip(&mask)
                .filter_map(|(sx, m)| if *m { Some(sx) } else { None })
                .chain(
                    sxy[1]
                        .iter()
                        .zip(&mask)
                        .filter_map(|(sy, m)| if *m { Some(sy) } else { None }),
                )
                .map(|v: &f32| v.abs())
                .max_by(|x, y| x.partial_cmp(y).unwrap())
                .unwrap();

            sxy[0]
                .iter()
                .zip(mask.iter().zip(sx_mask.iter_mut()))
                .filter_map(|(v, (m, pm))| if *m { Some((v, pm)) } else { None })
                .for_each(|(v, pm)| {
                    if v.abs() > 0.65 * max {
                        *pm = true;
                    }
                });

            sxy[1]
                .iter()
                .zip(mask.iter().zip(sy_mask.iter_mut()))
                .filter_map(|(v, (m, pm))| if *m { Some((v, pm)) } else { None })
                .for_each(|(v, pm)| {
                    if v.abs() > 0.65 * max {
                        *pm = true;
                    }
                });
        }

        let sx0: Vec<f32> = sx0
            .into_iter()
            .zip(&sx_mask)
            .filter_map(|(v, m)| if *m { Some(*v) } else { None })
            .collect();
        let sy0: Vec<f32> = sy0
            .into_iter()
            .zip(&sy_mask)
            .filter_map(|(v, m)| if *m { Some(*v) } else { None })
            .collect();

        let mut calibration = vec![];
        for sxy in poke_matrix.chunks(2) {
            let sx = sxy[0]
                .iter()
                .zip(&sx_mask)
                .filter_map(|(v, m)| if *m { Some(*v) } else { None });
            calibration.extend(sx);
            let sy = sxy[1]
                .iter()
                .zip(&sy_mask)
                .filter_map(|(v, m)| if *m { Some(*v) } else { None });
            calibration.extend(sy);
        }

        let n_segment = n_segment as usize;
        let mat = nalgebra::DMatrix::<f32>::from_column_slice(
            calibration.len() / n_segment,
            n_segment,
            &calibration,
        );

        let svd = mat.clone().svd(false, false);
        dbg!(&svd.singular_values);

        let pseudo_inverse = mat.clone().pseudo_inverse(0.).unwrap();

        Ok(Self {
            mask: (sx_mask, sy_mask),
            sxy0: (sx0, sy0),
            calibration: mat,
            pseudo_inverse,
        })
    }

    pub fn piston(&self, pym: &mut Pyramid) -> Vec<f32> {
        let sxy = pym.processing();
        let data = sxy
            .0
            .into_iter()
            .zip(&self.mask.0)
            .filter_map(|(v, m)| if *m { Some(*v) } else { None })
            .zip(&self.sxy0.0)
            .map(|(s, s0)| s - s0)
            .chain(
                sxy.1
                    .into_iter()
                    .zip(&self.mask.1)
                    .filter_map(|(v, m)| if *m { Some(*v) } else { None })
                    .zip(&self.sxy0.1)
                    .map(|(s, s0)| s - s0),
            );
        let piston =
            &self.pseudo_inverse * nalgebra::DVector::from_iterator(self.calibration.nrows(), data);
        piston.as_slice().to_vec()
    }
}
