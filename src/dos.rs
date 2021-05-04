use super::{
    shackhartmann::WavefrontSensor, shackhartmann::WavefrontSensorBuilder, Builder, Geometric, Gmt,
    Propagation, ShackHartmann, Source, GMT, SOURCE,
};
use dosio::{io::IO, DOSIOSError, DOS};

pub struct GmtOpticalSensorModel<
    U: WavefrontSensor + Propagation,
    T: WavefrontSensorBuilder + Builder<Component = U>,
> {
    gmt: GMT,
    src: SOURCE,
    sensor: T,
    flux_threshold: f64,
}
impl<U: WavefrontSensor + Propagation, T: WavefrontSensorBuilder + Builder<Component = U>>
    GmtOpticalSensorModel<U, T>
{
    pub fn new(sensor: T, flux_threshold: f64) -> Self {
        Self {
            gmt: Default::default(),
            src: sensor.guide_stars(),
            sensor,
            flux_threshold,
        }
    }
    pub fn build(self) -> GmtOpticalSensorModelInner<U> {
        let mut gmt = self.gmt.build();
        let mut src = self.src.build();
        let mut sensor = self.sensor.build();
        src.through(&mut gmt).xpupil();
        sensor.calibrate(&mut src, self.flux_threshold);
        GmtOpticalSensorModelInner { gmt, src, sensor }
    }
}

pub struct GmtOpticalSensorModelInner<T: Propagation> {
    pub gmt: Gmt,
    pub src: Source,
    pub sensor: T,
}
impl GmtOpticalSensorModelInner<ShackHartmann<Geometric>> {
    pub fn new<U: Builder<Component = ShackHartmann<Geometric>> + WavefrontSensorBuilder>(
        sensor: U,
    ) -> Self {
        Self {
            gmt: GMT::new().build(),
            src: sensor.guide_stars().build(),
            sensor: sensor.build(),
        }
    }
}
impl<T: Propagation> Iterator for GmtOpticalSensorModelInner<T> {
    type Item = ();
    fn next(&mut self) -> Option<Self::Item> {
        self.src
            .through(&mut self.gmt)
            .xpupil()
            .through(&mut self.sensor);
        Some(())
    }
}
impl DOS for GmtOpticalSensorModelInner<ShackHartmann<Geometric>> {
    fn inputs(&mut self, data: Vec<IO<Vec<f64>>>) -> Result<&mut Self, DOSIOSError> {
        data.into_iter()
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
                _ => Err(DOSIOSError::Inputs("GmtOpticalModel invalid inputs".into())),
            })
            .and(Ok(self))
    }
    fn outputs(&mut self) -> Option<Vec<IO<Vec<f64>>>> {
        self.sensor.process();
        let data: Vec<f32> = self.sensor.get_data().into();
        self.sensor.reset();
        Some(vec![IO::SensorData {
            data: Some(data.into_iter().map(|x| x as f64).collect::<Vec<f64>>()),
        }])
    }
}
