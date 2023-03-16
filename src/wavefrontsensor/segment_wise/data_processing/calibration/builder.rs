use std::ops::Range;

use indicatif::ProgressBar;
// use serde::{Deserialize, Serialize};

use crate::{
    wavefrontsensor::{Slopes, SlopesArray},
    Builder, FromBuilder, Gmt, Propagation, SegmentWiseSensor, SourceBuilder,
};

/* #[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationBuilder {
    gmt_builder: Option<GmtBuilder>,
    src_builder: Option<SourceBuilder>,
}
impl CalibrationBuilder {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn gmt(mut self, gmt: GmtBuilder) -> Self {
        self.gmt_builder = Some(gmt);
        self
    }
    pub fn source(mut self, source: SourceBuilder) -> Self {
        self.src_builder = Some(source);
        self
    }
} */

#[derive(Debug, Clone)]
pub enum DOF {
    Modes(Vec<usize>),
    Range(Range<usize>),
}
impl IntoIterator for DOF {
    type Item = usize;

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            DOF::Modes(value) => value.into_iter(),
            DOF::Range(value) => value.collect::<Vec<usize>>().into_iter(),
        }
    }
}
impl From<Vec<usize>> for DOF {
    fn from(value: Vec<usize>) -> Self {
        DOF::Modes(value)
    }
}
impl From<Range<usize>> for DOF {
    fn from(value: Range<usize>) -> Self {
        DOF::Range(value)
    }
}
impl DOF {
    pub fn split_at(self, idx: usize) -> (DOF, DOF) {
        match self {
            DOF::Modes(val) => {
                let (left, right) = val.split_at(idx);
                (DOF::Modes(left.to_vec()), DOF::Modes(right.to_vec()))
            }
            DOF::Range(val) => (DOF::Range(val.start..idx), DOF::Range(idx..val.end)),
        }
    }
    pub fn modes(&self) -> Vec<usize> {
        match self {
            DOF::Modes(value) => value.clone(),
            DOF::Range(value) => value.clone().collect(),
        }
    }
    pub fn n_mode(&self) -> usize {
        match self {
            DOF::Modes(value) => value.len(),
            DOF::Range(Range { start, end }) => end - start,
        }
    }
}

#[derive(Clone, Debug)]
pub enum RBM {
    Txyz(Option<DOF>),
    Rxyz(Option<DOF>),
    TRxyz,
}
impl<S: Into<String>> From<S> for RBM {
    fn from(value: S) -> Self {
        let rbm: String = value.into();
        match rbm.as_str() {
            "Txyz" => RBM::Txyz(None),
            "Txy" => RBM::Txyz(Some((0..2).into())),
            "Rxyz" => RBM::Rxyz(None),
            "Rxy" => RBM::Rxyz(Some((0..2).into())),
            "TRxyz" => RBM::TRxyz,
            _ => panic!(
                r#"expected "Txyz", "Txy", "Rxyz", "Rxy" or "TRxyz" found {}"#,
                rbm
            ),
        }
    }
}

#[derive(Clone, Debug)]
pub enum SegmentCalibration {
    Modes {
        name: String,
        dof: DOF,
        mirror: Mirror,
    },
    RBM {
        stroke: f64,
        rbm: RBM,
        mirror: Mirror,
    },
}
impl SegmentCalibration {
    pub fn slip_at(self, idx: usize) -> Option<(SegmentCalibration, SegmentCalibration)> {
        let SegmentCalibration::Modes { name, dof, mirror } = self else {return None};
        let (left, right) = dof.split_at(idx);
        Some((
            SegmentCalibration::Modes {
                name: name.clone(),
                dof: left,
                mirror: mirror.clone(),
            },
            SegmentCalibration::Modes {
                name,
                dof: right,
                mirror,
            },
        ))
    }
    pub fn modes<S, D, M>(name: S, dof: D, mirror: M) -> Self
    where
        S: Into<String>,
        D: Into<DOF>,
        M: Into<Mirror>,
    {
        SegmentCalibration::Modes {
            name: name.into(),
            dof: dof.into(),
            mirror: mirror.into(),
        }
    }
    pub fn rbm<R, M>(rbm: R, mirror: M) -> Self
    where
        R: Into<RBM>,
        M: Into<Mirror>,
    {
        SegmentCalibration::RBM {
            stroke: 1e-6,
            rbm: rbm.into(),
            mirror: mirror.into(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Mirror {
    M1,
    M2,
}
impl<S: Into<String>> From<S> for Mirror {
    fn from(value: S) -> Self {
        let m: String = value.into();
        match m.as_str() {
            "M1" => Mirror::M1,
            "M2" => Mirror::M2,
            _ => panic!(r#"expected "M1" or "M2", found {}"#, m),
        }
    }
}

impl SegmentCalibration {
    pub fn calibrate<W>(
        &self,
        sid: usize,
        wfs: &mut W,
        src_builder: SourceBuilder,
        pb: Option<ProgressBar>,
    ) -> SlopesArray
    where
        W: SegmentWiseSensor + Propagation,
    {
        let data_ref = wfs.zeroed_segment(sid, Some(src_builder.clone()));
        let mut src = src_builder.build().unwrap();
        let mut slopes = vec![];
        let slopes = match self {
            SegmentCalibration::Modes { name, dof, mirror } => {
                let l = 1 + dof
                    .clone()
                    .into_iter()
                    .last()
                    .expect("expect some modes, found none");
                let mut gmt = match mirror {
                    Mirror::M1 => Gmt::builder().m1(name, l),
                    Mirror::M2 => Gmt::builder().m2(name, l),
                }
                .build()
                .unwrap();

                gmt.keep(&[sid as i32]);

                let o2p = (2. * std::f64::consts::PI / src.wavelength()) as f32;

                for kl_mode in dof.clone() {
                    pb.as_ref().map(|pb| pb.inc(1));
                    gmt.reset();
                    let kl_a0 = 1e-6;
                    match mirror {
                        Mirror::M1 => gmt.m1_modes_ij(sid - 1, kl_mode, kl_a0),
                        Mirror::M2 => gmt.m2_modes_ij(sid - 1, kl_mode, kl_a0),
                    };
                    src.through(&mut gmt).xpupil();
                    let opd = src.phase().clone();
                    let opd_minmax =
                        opd.iter()
                            .fold((f32::INFINITY, f32::NEG_INFINITY), |(min, max), value| {
                                (
                                    if *value < min { *value } else { min },
                                    if *value > max { *value } else { max },
                                )
                            });
                    let phase_minmax = (o2p * opd_minmax.0, o2p * opd_minmax.1);
                    // println!("𝜑 minmax: {:?}", phase_minmax);
                    let kl_coef = 1e-2 * kl_a0 as f32 / phase_minmax.0.abs().max(phase_minmax.1);
                    // println!("KL coef.:{:e}", kl_coef);

                    match mirror {
                        Mirror::M1 => gmt.m1_modes_ij(sid - 1, kl_mode, kl_coef as f64),
                        Mirror::M2 => gmt.m2_modes_ij(sid - 1, kl_mode, kl_coef as f64),
                    };
                    src.through(&mut gmt).xpupil().through(wfs);
                    // let slopes_push = Slopes::from((&data_ref, &*wfs));
                    let slopes_push: Slopes = wfs.into_slopes(&data_ref);
                    wfs.reset();

                    match mirror {
                        Mirror::M1 => gmt.m1_modes_ij(sid - 1, kl_mode, -kl_coef as f64),
                        Mirror::M2 => gmt.m2_modes_ij(sid - 1, kl_mode, -kl_coef as f64),
                    };
                    src.through(&mut gmt).xpupil().through(wfs);
                    // let slopes_pull = Slopes::from((&data_ref, wfs));
                    let slopes_pull = wfs.into_slopes(&data_ref);
                    wfs.reset();

                    slopes.push((slopes_push - slopes_pull) / (2. * kl_coef));
                }
                pb.as_ref().map(|pb| pb.finish());
                slopes
            }
            SegmentCalibration::RBM {
                stroke,
                rbm,
                mirror,
            } => {
                let mut gmt = Gmt::builder().build().unwrap();
                gmt.keep(&[sid as i32]);
                let dof = match rbm {
                    RBM::Txyz(dof) | RBM::Rxyz(dof) => {
                        dof.clone().unwrap_or(DOF::Range(0..3)).into_iter()
                    }
                    RBM::TRxyz => (0..6).collect::<Vec<usize>>().into_iter(),
                };
                for i in dof {
                    pb.as_ref().map(|pb| pb.inc(1));
                    gmt.reset();

                    let mut tr = [0f64; 6];

                    match rbm {
                        RBM::Txyz(_) => tr[i] = *stroke,
                        RBM::Rxyz(_) => tr[i + 3] = *stroke,
                        RBM::TRxyz => tr[i] = *stroke,
                    }

                    match mirror {
                        Mirror::M1 => gmt.m1_segment_state(sid as i32, &tr[..3], &tr[3..]),
                        Mirror::M2 => gmt.m2_segment_state(sid as i32, &tr[..3], &tr[3..]),
                    };
                    src.through(&mut gmt).xpupil().through(wfs);
                    // let slopes_push = Slopes::from((&data_ref, &*wfs));
                    let slopes_push: Slopes = wfs.into_slopes(&data_ref);
                    wfs.reset();

                    match rbm {
                        RBM::Txyz(_) => tr[i] = *stroke,
                        RBM::Rxyz(_) => tr[i + 3] = *stroke,
                        RBM::TRxyz => tr[i] = *stroke,
                    }

                    match mirror {
                        Mirror::M1 => gmt.m1_segment_state(sid as i32, &tr[..3], &tr[3..]),
                        Mirror::M2 => gmt.m2_segment_state(sid as i32, &tr[..3], &tr[3..]),
                    };
                    src.through(&mut gmt).xpupil().through(wfs);
                    // let slopes_pull = Slopes::from((&data_ref, wfs));
                    let slopes_pull = wfs.into_slopes(&data_ref);
                    wfs.reset();

                    slopes.push((slopes_push - slopes_pull) / (2. * *stroke as f32));
                }
                pb.as_ref().map(|pb| pb.finish());
                slopes
            }
        };
        (data_ref, slopes).into()
    }
}
