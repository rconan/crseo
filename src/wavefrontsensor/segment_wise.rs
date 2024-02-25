use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};

use crate::{Builder, SourceBuilder, WavefrontSensor, WavefrontSensorBuilder};

pub mod data_processing;
pub mod differential_piston_sensor;
pub mod geom_shack;
pub mod phase_sensor;
pub mod piston_sensor;
pub mod pyramid;
use data_processing::{Calibration, DataRef, SegmentCalibration, Slopes, SlopesArray};

use super::Pyramid;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub enum GmtSegmentation {
    #[default]
    Complete,
    Partial(Vec<u8>),
    Outers,
    Center,
}
impl<T: Into<u8>> From<Vec<T>> for GmtSegmentation {
    fn from(value: Vec<T>) -> Self {
        Self::Partial(value.into_iter().map(|value| value.into()).collect())
    }
}
impl From<String> for GmtSegmentation {
    fn from(value: String) -> Self {
        match value.to_lowercase().as_str() {
            "outers" => Self::Outers,
            "center" => Self::Center,
            _ => unimplemented!(
                r#"Conversion to GmtSegmentation: expected "outers" or "center", found {value}"#
            ),
        }
    }
}
impl GmtSegmentation {
    pub fn iter(&self) -> GmtSegmentationIter {
        GmtSegmentationIter(match self {
            GmtSegmentation::Complete => Box::new(1..=7),
            GmtSegmentation::Partial(sids) => Box::new(sids.clone().into_iter()),
            GmtSegmentation::Outers => Box::new(1..7),
            GmtSegmentation::Center => Box::new(Some(7).into_iter()),
        })
    }
    pub fn n_segment(&self) -> usize {
        match self {
            GmtSegmentation::Complete => 7,
            GmtSegmentation::Partial(sids) => sids.len(),
            GmtSegmentation::Outers => 6,
            GmtSegmentation::Center => 1,
        }
    }
}

pub struct GmtSegmentationIter(Box<dyn Iterator<Item = u8>>);

impl Iterator for GmtSegmentationIter {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

/// Detector frame
#[derive(Default, Debug)]
pub struct Frame<T = f32> {
    pub resolution: (usize, usize),
    pub value: Vec<T>,
}

impl From<&Pyramid> for Frame<f32> {
    fn from(pym: &Pyramid) -> Self {
        Self {
            resolution: pym.camera_resolution(),
            value: pym.frame(),
        }
    }
}

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
    fn frame(&self) -> Frame<f32> {
        Default::default()
    }
}

pub trait SegmentWiseSensorBuilder:
    Builder + WavefrontSensorBuilder + Clone + Send + Sized + 'static
{
    fn pupil_sampling(&self) -> usize;
    fn calibrate(self, segment: SegmentCalibration, src_builder: SourceBuilder) -> Calibration
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
            let seg = segment.clone();
            let seg_src_builder = src_builder.clone();
            handle.push(std::thread::spawn(move || {
                unsafe { ffi::set_device((sid - 1) as i32 % n) };
                let mut wfs = builder.build().unwrap();
                // pym.calibrate_segment(src_builder, sid, n_mode, Some(pb))
                seg.calibrate(sid, &mut wfs, seg_src_builder.clone(), pb)
            }));
        }
        let mut calibration = handle.into_iter().fold(Calibration::default(), |mut c, h| {
            c.push(h.join().unwrap());
            c
        });
        m.clear().unwrap();
        calibration.src = src_builder;
        calibration
    }
}
