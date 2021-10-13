use crate::{
    dos::{CfdDataBase, DomeSeeing},
    shackhartmann::WavefrontSensor,
    shackhartmann::WavefrontSensorBuilder,
    Atmosphere, Builder, Diffractive, Geometric, Gmt, Propagation, ShackHartmann, Source,
    ATMOSPHERE, GMT, SOURCE,
};
use dosio::{io::IO, DOSIOSError, Dos};

/// GMT Optical Sensor Model
pub struct GmtOpticalSensorModel<U, T>
where
    U: WavefrontSensor + Propagation,
    T: WavefrontSensorBuilder + Builder<Component = U>,
{
    gmt: GMT,
    src: SOURCE,
    atm: Option<ATMOSPHERE>,
    dome_seeing: Option<DomeSeeing>,
    sensor: T,
    flux_threshold: f64,
}
impl<U, T> GmtOpticalSensorModel<U, T>
where
    U: WavefrontSensor + Propagation,
    T: WavefrontSensorBuilder + Builder<Component = U> + Clone,
{
    /// Creates a new  wavefront sensor based GMT optical model
    ///
    /// Creates a new model based on the default parameters for [GMT] and the wavefront sensor model
    pub fn new(src_template: Option<SOURCE>) -> Self {
        Self {
            gmt: Default::default(),
            src: <T as Builder>::new().guide_stars(src_template),
            atm: None,
            sensor: Builder::new(),
            flux_threshold: 0.8,
            dome_seeing: None,
        }
    }
    /// Sets the GMT model
    pub fn gmt(self, gmt: GMT) -> Self {
        Self { gmt, ..self }
    }
    /// Sets the wavefront sensor
    pub fn sensor(self, sensor: T) -> Self {
        let src = sensor.clone().guide_stars(Some(self.src));
        Self {
            sensor,
            src,
            ..self
        }
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
    /// Builds a new GMT optical sensor model
    pub fn build(self) -> crate::Result<GmtOpticalSensorModelInner<U>> {
        let mut gmt = self.gmt.build()?;
        let mut src = self.src.build()?;
        let mut sensor = self.sensor.build()?;
        src.through(&mut gmt).xpupil();
        sensor.calibrate(&mut src, self.flux_threshold);
        Ok(GmtOpticalSensorModelInner {
            gmt,
            src,
            sensor,
            atm: match self.atm {
                Some(atm) => Some(atm.build()?),
                None => None,
            },
            dome_seeing: self.dome_seeing,
        })
    }
}

/// GMT Optical Sensor Model CEO Interface
///
/// The [GmtOpticalSensorModelInner] structure is the interface between CEO and DOS.
/// The propagation through the optical system happened each time the [Self::next()] method of the [Iterator] trait is invoked.
/// The states of the GMT M1 and M2 segments are set with the `OSSM1Lcl` and `MCM2Lcl6D` variant of the `IO` type of the `dosio` module that are passed to the [Self::inputs()] method of the `Dos` trait.
/// Sensor data are collected with the [Self::outputs()] method of the `Dos` trait wrapped into the `dosio::io::IO::SensorData` .
pub struct GmtOpticalSensorModelInner<S>
where
    S: Propagation,
{
    pub gmt: Gmt,
    pub src: Source,
    pub sensor: S,
    pub atm: Option<Atmosphere>,
    pub dome_seeing: Option<DomeSeeing>,
}
impl<T: Propagation> Iterator for GmtOpticalSensorModelInner<T> {
    type Item = ();
    fn next(&mut self) -> Option<Self::Item> {
        match (&mut self.atm, &mut self.dome_seeing) {
            (Some(atm), None) => self
                .src
                .through(&mut self.gmt)
                .xpupil()
                .through(atm)
                .through(&mut self.sensor),
            (None, Some(ds)) => {
                ds.next();
                self.src
                    .through(&mut self.gmt)
                    .xpupil()
                    .through(ds)
                    .through(&mut self.sensor)
            }
            (Some(atm), Some(ds)) => {
                ds.next();
                self.src
                    .through(&mut self.gmt)
                    .xpupil()
                    .through(ds)
                    .through(atm)
                    .through(&mut self.sensor)
            }
            (None, None) => self
                .src
                .through(&mut self.gmt)
                .xpupil()
                .through(&mut self.sensor),
        };
        Some(())
    }
}
impl Dos for GmtOpticalSensorModelInner<ShackHartmann<Geometric>> {
    type Input = Vec<f64>;
    type Output = Vec<f64>;
    fn inputs(&mut self, data: Option<Vec<IO<Self::Input>>>) -> Result<&mut Self, DOSIOSError> {
        match data {
            Some(data) => data
                .into_iter()
                .try_for_each(|io| match io {
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
                    IO::OSSM1Lcl { data: None } => Ok(()),
                    IO::MCM2Lcl6D { data: None } => Ok(()),
                    _ => Err(DOSIOSError::Inputs("GmtOpticalModel invalid inputs".into())),
                })
                .and(Ok(self)),
            None => Ok(self),
        }
    }
    fn outputs(&mut self) -> Option<Vec<IO<Self::Output>>> {
        self.sensor.process();
        let data: Vec<f32> = self.sensor.get_data().into();
        self.sensor.reset();
        Some(vec![IO::SensorData {
            data: Some(data.into_iter().map(|x| x as f64).collect::<Vec<f64>>()),
        }])
    }
}
impl Dos for GmtOpticalSensorModelInner<ShackHartmann<Diffractive>> {
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
        self.sensor.readout();
        self.sensor.process();
        let data: Vec<f32> = self.sensor.get_data().into();
        self.sensor.reset();
        Some(vec![IO::SensorData {
            data: Some(data.into_iter().map(|x| x as f64).collect::<Vec<f64>>()),
        }])
    }
}
