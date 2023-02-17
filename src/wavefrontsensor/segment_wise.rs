use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

use crate::{Builder, SourceBuilder};

use super::{Calibration, SlopesArray};

pub trait SegmentWiseSensor {
    fn calibrate_segment(
        &mut self,
        sid: usize,
        n_mode: usize,
        pb: Option<ProgressBar>,
    ) -> SlopesArray;
    fn calibrate(&mut self, n_mode: usize) -> Calibration {
        (1..=7)
            .inspect(|i| println!("Calibrating segment # {i}"))
            .fold(Calibration::default(), |mut c, i| {
                c.push(self.calibrate_segment(i, n_mode, None));
                c
            })
    }
    fn pupil_sampling(&self) -> usize;
    fn guide_star(&self, gs: Option<SourceBuilder>) -> SourceBuilder {
        gs.unwrap_or_default().pupil_sampling(self.pupil_sampling())
    }
}

pub trait SegmentWiseSensorBuilder: Builder + Clone + Copy + Send + Sized + 'static {
    fn calibrate(self, n_mode: usize) -> Calibration
    where
        Self::Component: SegmentWiseSensor,
    {
        let m = MultiProgress::new();
        let mut handle = vec![];
        for sid in 1..=7 {
            let pb = m.add(ProgressBar::new(n_mode as u64 - 1));
            pb.set_style(
                ProgressStyle::with_template(
                    "{msg} [{eta_precise}] {bar:50.cyan/blue} {pos:>7}/{len:7}",
                )
                .unwrap(),
            );
            pb.set_message(format!("Calibrating segment #{sid}"));
            let n = unsafe { ffi::get_device_count() };
            let builder = self.clone();
            handle.push(std::thread::spawn(move || {
                unsafe { ffi::set_device((sid - 1) as i32 % n) };
                let mut pym = builder.build().unwrap();
                pym.calibrate_segment(sid, n_mode, Some(pb))
            }));
        }
        let calibration = handle.into_iter().fold(Calibration::default(), |mut c, h| {
            c.push(h.join().unwrap());
            c
        });
        m.clear().unwrap();
        calibration
    }
}
