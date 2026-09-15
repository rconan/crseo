use gmt_dos_systems_m1::SingularModes;

use crate::{ModesSets, ModesSetsResult, Native, Set};

impl ModesSets<Native> {
    /// Inserts M1 singular modes into [ModesSets]
    pub fn insert_modes(&mut self, modes: &SingularModes, n_mode: usize) -> ModesSetsResult<()> {
        self.segment2set = [0, 1, 2, 3, 4, 5, 6];

        let segments_modes = &*modes;

        for (i, segment_modes) in segments_modes.iter().enumerate() {
            let modes = segment_modes.modes_into_mat(Some(n_mode));
            let set = modes
                .column_iter()
                .map(|c| c.as_slice().to_vec())
                .collect::<Set>()
                .xy(segment_modes.nodes_iter().map(|n| [n[0], n[1]]));
            self.insert(i, set)?;
        }
        Ok(())
    }
    /// Inserts M1 raw singular modes into [ModesSets]
    pub fn insert_raw_modes(&mut self, modes: &SingularModes) -> ModesSetsResult<()> {
        self.segment2set = [0, 1, 2, 3, 4, 5, 6];

        let segments_modes = &*modes;

        for (i, segment_modes) in segments_modes.iter().enumerate() {
            let modes = segment_modes.raw_modes_into_mat();
            let set = modes
                .column_iter()
                .map(|c| c.as_slice().to_vec())
                .collect::<Set>()
                .xy(segment_modes.nodes_iter().map(|n| [n[0], n[1]]));
            self.insert(i, set)?;
        }
        Ok(())
    }
    /// Inserts M1 singular modes into [ModesSets]
    ///
    /// The modes are normalized such as the means are zeros and standard deviations are ones
    pub fn insert_normalized_modes(
        &mut self,
        modes: &SingularModes,
        n_mode: usize,
    ) -> ModesSetsResult<()> {
        self.segment2set = [0, 1, 2, 3, 4, 5, 6];

        let segments_modes = &*modes;

        for (i, segment_modes) in segments_modes.iter().enumerate() {
            let modes = segment_modes.modes_into_mat(Some(n_mode));
            let set = modes
                .column_iter()
                .map(|c| {
                    let x = c.as_slice();
                    let mean = x.iter().sum::<f64>() / x.len() as f64;
                    let rms = ((x.iter().map(|x| *x - mean).map(|x| x * x).sum::<f64>())
                        / x.len() as f64)
                        .sqrt();
                    // println!("{i}: {rms:.3e}");
                    x.iter().map(|x| (*x - mean) / rms).collect::<Vec<_>>()
                })
                .collect::<Set>()
                .xy(segment_modes.nodes_iter().map(|n| [n[0], n[1]]));
            self.insert(i, set)?;
        }
        Ok(())
    }
}
