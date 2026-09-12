use std::{collections::HashMap, fmt::Display};

#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ModesSet {
    pub n_sample: usize,
    pub width: f64,
    pub sets: HashMap<usize, Vec<Vec<f64>>>,
    pub surf2mode: [i32; 7],
}
impl Display for ModesSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "GMT Modes Set (N={}px, L={}m): {:?} -> {:?}",
            self.n_sample,
            self.width,
            self.n_mode(),
            self.surf2mode
        )
    }
}
impl ModesSet {
    pub fn new(n_sample: usize, width: f64, surf2mod: impl Into<[i32; 7]>) -> Self {
        Self {
            n_sample,
            width,
            surf2mode: surf2mod.into(),
            ..Default::default()
        }
    }
    pub fn insert(&mut self, idx: usize, mode: impl Into<Vec<f64>>) -> &mut Self {
        self.sets.entry(idx).or_insert(vec![]).push(mode.into());
        self
    }
    pub fn n_mode(&self) -> Vec<usize> {
        let mut n_mode = vec![];
        for i in 1.. {
            if let Some(modes) = self.sets.get(&i) {
                n_mode.push(modes.len());
            } else {
                break;
            }
        }
        n_mode
    }
    pub fn max_n_mode(&self) -> Option<usize> {
        self.n_mode()
            .iter()
            .max_by(|x, y| x.partial_cmp(y).unwrap())
            .map(|n| *n)
    }
}
