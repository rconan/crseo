use crate::{
    shackhartmann::WavefrontSensor, shackhartmann::WavefrontSensorBuilder, Atmosphere, Builder,
    Geometric, Gmt, Propagation, ShackHartmann, Source, ATMOSPHERE, GMT, SOURCE,
};
use dosio::{io::IO, DOSIOSError, Dos, DosIoData};

/// GMT Optical Sensor Model
pub struct GmtOpticalSensorModel<U, T>
where
    U: WavefrontSensor + Propagation,
    T: WavefrontSensorBuilder + Builder<Component = U>,
{
    gmt: GMT,
    src: SOURCE,
    atm: Option<ATMOSPHERE>,
    sensor: T,
    flux_threshold: f64,
}
impl<U, T> GmtOpticalSensorModel<U, T>
where
    U: WavefrontSensor + Propagation,
    T: WavefrontSensorBuilder + Builder<Component = U>,
{
    /// Creates a new GMT optical model
    ///
    /// Creates a default model based on the default parameters for [GMT] and the given sensor model
    pub fn new(sensor: T, flux_threshold: f64) -> Self {
        Self {
            gmt: Default::default(),
            src: sensor.guide_stars(),
            atm: None,
            sensor,
            flux_threshold,
        }
    }
    /// Sets the [atmosphere](ATMOSPHERE) template    
    pub fn atmosphere(self, atm: ATMOSPHERE) -> Self {
        Self {
            atm: Some(atm),
            ..self
        }
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
        })
    }
}

/// GMT Optical Sensor Model CEO Interface
///
/// The [GmtOpticalSensorModelInner] structure is the interface between CEO and DOS.
/// The propagation through the optical system happened each time the [Self::next()] method of the [Iterator] trait is invoked.
/// The states of the GMT M1 and M2 segments are set with the `OSSM1Lcl` and `MCM2Lcl6D` variant of the `IO` type of the `dosio` module that are passed to the [Self::inputs()] method of the `Dos` trait.
/// Sensor data are collected with the [Self::outputs()] method of the `Dos` trait wrapped into the `dosio::io::IO::SensorData` .
pub struct GmtOpticalSensorModelInner<T: Propagation> {
    pub gmt: Gmt,
    pub src: Source,
    pub sensor: T,
    pub atm: Option<Atmosphere>,
}
impl<T: Propagation> Iterator for GmtOpticalSensorModelInner<T> {
    type Item = ();
    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.atm {
            Some(atm) => self
                .src
                .through(&mut self.gmt)
                .xpupil()
                .through(atm)
                .through(&mut self.sensor),
            None => self
                .src
                .through(&mut self.gmt)
                .xpupil()
                .through(&mut self.sensor),
        };
        Some(())
    }
}
impl Dos for GmtOpticalSensorModelInner<ShackHartmann<Geometric>> {
    fn inputs(&mut self, data: DosIoData) -> Result<&mut Self, DOSIOSError> {
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
    fn outputs(&mut self) -> DosIoData {
        self.sensor.process();
        let data: Vec<f32> = self.sensor.get_data().into();
        self.sensor.reset();
        Some(vec![IO::SensorData {
            data: Some(data.into_iter().map(|x| x as f64).collect::<Vec<f64>>()),
        }])
    }
}