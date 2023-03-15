use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

use crate::{Builder, SourceBuilder, WavefrontSensor, WavefrontSensorBuilder};

use super::{
    data_processing::{DataRef, SegmentCalibration},
    Calibration, Slopes, SlopesArray,
};

pub trait SegmentWiseSensor: WavefrontSensor {
    fn pupil_sampling(&self) -> usize;
    fn calibrate_segment(
        &mut self,
        src: Option<SourceBuilder>,
        sid: usize,
        n_mode: usize,
        pb: Option<ProgressBar>,
    ) -> SlopesArray;
    fn calibrate(&mut self, src: Option<SourceBuilder>, n_mode: usize) -> Calibration {
        (1..=7)
            .inspect(|i| println!("Calibrating segment # {i}"))
            .fold(Calibration::default(), |mut c, i| {
                c.push(self.calibrate_segment(src.clone(), i, n_mode, None));
                c
            })
    }
    fn zeroed_segment(&mut self, sid: usize, src: Option<SourceBuilder>) -> DataRef;
    // fn reset(&mut self);
    fn into_slopes(&self, data_ref: &DataRef) -> Slopes;
}

pub trait SegmentWiseSensorBuilder:
    Builder + WavefrontSensorBuilder + Clone + Copy + Send + Sized + 'static
{
    fn pupil_sampling(&self) -> usize;
    fn calibrate(self, segment: SegmentCalibration, src: SourceBuilder) -> Calibration
    where
        Self::Component: SegmentWiseSensor,
    {
        let m = MultiProgress::new();
        let mut handle = vec![];
        for sid in 1..=7 {
            let pb = if let SegmentCalibration::Modes { dof, .. } = &segment {
                let pb = m.add(ProgressBar::new(dof.n_mode() as u64));
                pb.set_style(
                    ProgressStyle::with_template(
                        "{msg} [{eta_precise}] {bar:50.cyan/blue} {pos:>7}/{len:7}",
                    )
                    .unwrap(),
                );
                pb.set_message(format!("Calibrating segment #{sid}"));
                Some(pb)
            } else {
                None
            };

            let n = unsafe { ffi::get_device_count() };
            let builder = self.clone();
            let src_builder = src.clone();
            let seg = segment.clone();
            handle.push(std::thread::spawn(move || {
                unsafe { ffi::set_device((sid - 1) as i32 % n) };
                let mut wfs = builder.build().unwrap();
                // pym.calibrate_segment(src_builder, sid, n_mode, Some(pb))
                seg.calibrate(sid, &mut wfs, src_builder, pb)
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
