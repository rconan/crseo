//! # DOS Interface
//!
//! The `dos` module implements the `Dos` trait from the `dosio` crate for 2 new structure: `GmtOpticalModelInner` and `GMTOpticalSensorModelInner`.
//! Both structure are created with their respective Builders i.e. `GmtOpticalModel` and `GmtOpticalSensorModel`.
pub mod dome_seeing;
pub mod sensor;
pub use dome_seeing::DomeSeeing;
pub use sensor::GmtOpticalSensorModel;

use crate::{
    pssn::TelescopeError, Atmosphere, Builder, Gmt, PSSn, Source, ATMOSPHERE, GMT, PSSN, SOURCE,
};
use dosio::{io::IO, ios, DOSIOSError, Dos};

pub struct CfdDataBase {
    region: String,
    bucket: String,
    path: String,
}
impl Default for CfdDataBase {
    fn default() -> Self {
        Self {
            region: "us-west-2".to_string(),
            bucket: "gmto.modeling".to_string(),
            path: "Baseline2020".to_string(),
        }
    }
}

/// GMT Optical Model
#[derive(Default)]
pub struct GmtOpticalModel {
    gmt: GMT,
    src: SOURCE,
    atm: Option<ATMOSPHERE>,
    dome_seeing: Option<DomeSeeing>,
    outputs: Vec<IO<()>>,
    pssn: Option<PSSN<TelescopeError>>,
}
impl GmtOpticalModel {
    /// Creates a new GMT optical model
    ///
    /// Creates a default model based on the default parameters for [GMT] and [SOURCE]
    pub fn new() -> Self {
        Default::default()
    }
    /// Sets the GMT model
    pub fn gmt(self, gmt: GMT) -> Self {
        Self { gmt, ..self }
    }
    /// Sets the `Source` model
    pub fn source(self, src: SOURCE) -> Self {
        Self { src, ..self }
    }
    /// Sets the [atmosphere](ATMOSPHERE) template
    pub fn atmosphere(self, atm: ATMOSPHERE) -> Self {
        Self {
            atm: Some(atm),
            ..self
        }
    }
    /// Sets the dome seeing model
    pub async fn dome_seeing(
        self,
        cfd_case: &str,
        duration: f64,
        sampling_rate: f64,
        cfd_data_base: Option<CfdDataBase>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let CfdDataBase {
            region,
            bucket,
            path,
        } = cfd_data_base.unwrap_or_default();
        let cfd_duration = (duration * 5f64).ceil() as usize;
        let cfd_rate = sampling_rate as usize / 5;
        let mut ds = DomeSeeing::new(
            &region,
            &bucket,
            &path,
            cfd_case,
            cfd_duration,
            Some(cfd_rate),
        );
        ds.get_keys().await?.load_opd().await?;
        Ok(Self {
            dome_seeing: Some(ds),
            ..self
        })
    }
    /// Sets the output type
    ///
    /// The output type is one of the [dosio::io::IO] type
    pub fn output(self, data: IO<()>) -> Self {
        let mut outputs = self.outputs;
        match data {
            IO::Pssn { .. } => {
                outputs.push(data);
                Self {
                    outputs,
                    pssn: Some(PSSN::<TelescopeError>::new()),
                    ..self
                }
            }
            _ => {
                outputs.push(data);
                Self { outputs, ..self }
            }
        }
    }
    /// Builds a new GMT optical model
    pub fn build(self) -> crate::Result<GmtOpticalModelInner> {
        Ok(GmtOpticalModelInner {
            gmt: self.gmt.build()?,
            src: self.src.clone().build()?,
            atm: match self.atm {
                Some(atm) => Some(atm.build()?),
                None => None,
            },
            dome_seeing: self.dome_seeing,
            pssn: match self.pssn {
                Some(pssn) => Some(pssn.source(&(self.src.build()?)).build()?),
                None => None,
            },
            outputs: self.outputs,
        })
    }
}

/// GMT Optical Model CEO Interface
///
/// The [GmtOpticalModelInner] structure is the interface between CEO and DOS.
/// The propagation through the optical system happened each time the [Self::next()] method of the [Iterator] trait is invoked.
/// The states of the GMT M1 and M2 segments are set with the `OSSM1Lcl` and `MCM2Lcl6D` variant of the `IO` type of the `dosio` module that are passed to the [Self::inputs()] method of the `Dos` trait.
/// Model outputs are collected with the [Self::outputs()] method of the `Dos` trait.
/// Valid `dosio::io::IO` output variants are:
///  - `SrcWfeRms` : returns the wavefront standard error in the telescope  exit pupil
///  - `Pssn` : returns the PSSn value at the wavelength of the source
pub struct GmtOpticalModelInner {
    pub gmt: Gmt,
    pub src: Source,
    pub atm: Option<Atmosphere>,
    pub dome_seeing: Option<DomeSeeing>,
    pub pssn: Option<PSSn<TelescopeError>>,
    pub outputs: Vec<IO<()>>,
}
impl Iterator for GmtOpticalModelInner {
    type Item = ();
    fn next(&mut self) -> Option<Self::Item> {
        self.src.through(&mut self.gmt).xpupil();
        if let Some(ds) = &mut self.dome_seeing {
            ds.next();
            self.src.through(ds);
        }
        if let Some(atm) = &mut self.atm {
            self.src.through(atm);
        }
        if let Some(pssn) = &mut self.pssn {
            self.src.through(pssn);
        }
        Some(())
    }
}
impl Dos for GmtOpticalModelInner {
    type Input = Vec<f64>;
    type Output = Vec<f64>;
    fn inputs(&mut self, data: Option<Vec<IO<Self::Input>>>) -> Result<&mut Self, DOSIOSError> {
        match data {
            Some(data) => data
                .into_iter()
                .try_for_each(|mut io| match io {
                    IO::OSSM1Lcl { data: Some(values) } => {
                        values.chunks(6).enumerate().for_each(|(sid0, v)| {
                            self.gmt
                                .m1_segment_state((sid0 + 1) as i32, &v[..3], &v[3..]);
                        });
                        Ok(())
                    }
                    IO::MCM2Lcl6D { data: Some(values) } => {
                        values.chunks(6).enumerate().for_each(|(sid0, v)| {
                            self.gmt
                                .m2_segment_state((sid0 + 1) as i32, &v[..3], &v[3..]);
                        });
                        Ok(())
                    }
                    IO::M1modes {
                        data: Some(ref mut values),
                    } => {
                        self.gmt.m1_modes(values);
                        Ok(())
                    }
                    IO::M1modes { data: None } => Ok(()),
                    IO::OSSM1Lcl { data: None } => Ok(()),
                    IO::MCM2Lcl6D { data: None } => Ok(()),
                    _ => Err(DOSIOSError::Inputs("GmtOpticalModel invalid inputs".into())),
                })
                .and(Ok(self)),
            None => Ok(self),
        }
    }
    fn outputs(&mut self) -> Option<Vec<IO<Self::Output>>> {
        self.outputs
            .clone()
            .iter()
            .map(|io| match io {
                IO::SrcWfeRms { .. } => Some(ios!(SrcWfeRms(self.src.wfe_rms_f64()))),
                IO::SrcSegmentWfeRms { .. } => {
                    Some(ios!(SrcSegmentWfeRms(self.src.segment_wfe_rms_f64())))
                }
                IO::SrcSegmentPiston { .. } => {
                    Some(ios!(SrcSegmentPiston(self.src.segment_piston_f64())))
                }
                IO::SrcSegmentGradients { .. } => {
                    Some(ios!(SrcSegmentGradients(self.src.segment_gradients_f64())))
                }
                IO::Pssn { .. } => match &mut self.pssn {
                    Some(pssn) => Some(ios!(Pssn(
                        pssn.peek()
                            .estimates
                            .iter()
                            .cloned()
                            .map(|x| x as f64)
                            .collect()
                    ))),
                    None => None,
                },
                _ => None,
            })
            .collect()
    }
}
