use gmt_dos_systems_m1_modes_lib::SingularModes;

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
    ) -> ModesSetsResult<Vec<Vec<f64>>> {
        self.insert_scaled_modes(modes, n_mode, false)
    }
    /// Inserts M1 singular modes into [ModesSets]
    ///
    /// The modes are scaled such as the standard deviations are ones, the means are kept:
    /// the modes are the surfaces of the forces of the modes divided by the standard deviations
    pub fn insert_unit_rms_modes(
        &mut self,
        modes: &SingularModes,
        n_mode: usize,
    ) -> ModesSetsResult<Vec<Vec<f64>>> {
        self.insert_scaled_modes(modes, n_mode, true)
    }
    fn insert_scaled_modes(
        &mut self,
        modes: &SingularModes,
        n_mode: usize,
        keep_mean: bool,
    ) -> ModesSetsResult<Vec<Vec<f64>>> {
        self.segment2set = [0, 1, 2, 3, 4, 5, 6];

        let segments_modes = &*modes;

        let mut rms = vec![];
        for (i, segment_modes) in segments_modes.iter().enumerate() {
            let modes = segment_modes.modes_into_mat(Some(n_mode));
            let (set, set_rms): (Vec<Vec<f64>>, Vec<f64>) = modes
                .column_iter()
                .map(|c| {
                    let x = c.as_slice();
                    let mean = x.iter().sum::<f64>() / x.len() as f64;
                    let rms = ((x.iter().map(|x| *x - mean).map(|x| x * x).sum::<f64>())
                        / x.len() as f64)
                        .sqrt();
                    // println!("{i}: {rms:.3e}");
                    let offset = if keep_mean { 0f64 } else { mean };
                    (
                        x.iter().map(|x| (*x - offset) / rms).collect::<Vec<_>>(),
                        rms,
                    )
                })
                .unzip();
            let set = Set::from_iter(set.into_iter())
                .xy(segment_modes.nodes_iter().map(|n| [n[0], n[1]]));
            self.insert(i, set)?;
            rms.push(set_rms);
        }
        Ok(rms)
    }
}
