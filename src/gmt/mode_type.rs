use std::fmt::Display;

use crseo_modes_set::{ModesSetError, ModesSets, Regular};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum ModeType {
    CeoFile(String),
    DataSet {
        n_sample: usize,
        width: f64,
        n_set: usize,
        n_mode: usize,
        s2b: [i32; 7],
        data: Box<[f64]>,
    },
    Zernike(usize),
}
impl Default for ModeType {
    fn default() -> Self {
        Self::CeoFile(String::new())
    }
}
impl Display for ModeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModeType::CeoFile(name) => write!(f, "{name}"),
            ModeType::Zernike(_) => write!(f, "zernike"),
            ModeType::DataSet {
                n_sample,
                width,
                n_set,
                n_mode,
                ..
            } => write!(
                f,
                "{n_set} set of {n_mode} segment figures ({n_sample}px,{width}m)"
            ),
        }
    }
}
impl From<&str> for ModeType {
    fn from(value: &str) -> Self {
        Self::CeoFile(value.into())
    }
}
impl From<String> for ModeType {
    fn from(value: String) -> Self {
        Self::CeoFile(value)
    }
}
impl From<&String> for ModeType {
    fn from(value: &String) -> Self {
        Self::CeoFile(value.into())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ModeTypeError {
    #[error("failed to convert ModesSet to ModeType, no modes found")]
    MissingModes,
    #[error("expected {0} samples found {1} for mode #{2} in set {3}")]
    ModeSampling(usize, usize, usize, usize),
    #[error("failed to convert ModesSets into ModeType")]
    Conversion(#[from] ModesSetError),
}

impl TryFrom<ModesSets<Regular>> for ModeType {
    type Error = ModeTypeError;

    fn try_from(modes_set: ModesSets<Regular>) -> Result<Self, Self::Error> {
        let Some(n_mode) = modes_set.max_n_mode()? else {
            return Err(ModeTypeError::MissingModes);
        };
        let ModesSets {
            n_sample,
            width,
            sets,
            segment2set: surf2mod,
            ..
        } = modes_set;
        let n_set = sets.len();
        let mut dataset: Vec<_> = sets
            .into_iter()
            .map(|(k, mut data)| {
                (0..(n_mode - data.len())).for_each(|_| data.push(vec![0f64; n_sample]));
                (k, data)
            })
            .collect();
        dataset.sort_by_key(|(k, _)| *k);
        let n2 = n_sample * n_sample;
        for (idx, set) in dataset.iter() {
            for (i, mode) in set.iter().enumerate() {
                (mode.len() == n2)
                    .ok_or_else(|| ModeTypeError::ModeSampling(n2, mode.len(), i, *idx))?;
            }
        }
        Ok(Self::DataSet {
            n_sample,
            width,
            n_set,
            n_mode,
            s2b: surf2mod,
            data: dataset
                .into_iter()
                .flat_map(|(_, v)| v.into_iter().flatten())
                .collect(),
        })
    }
}
