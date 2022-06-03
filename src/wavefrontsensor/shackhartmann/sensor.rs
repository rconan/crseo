use super::{Diffractive, Geometric, Model, WavefrontSensor};
use crate::{cu::Single, imaging::NoiseDataSheet, Cu, Mask, Propagation, Source};

/// shackhartmann wrapper
pub struct ShackHartmann<S: Model> {
    pub _c_: S,
    /// The size of the square lenslet array
    pub n_side_lenslet: i32,
    /// The number of pixel per lenslet in the telescope pupil
    pub n_px_lenslet: i32,
    /// The lenslet array pitch [m]
    pub d: f64,
    /// The number of WFS
    pub n_sensor: i32,
    /// The total number of centroids
    pub n_centroids: i32,
    /// The centroids
    pub centroids: Cu<Single>,
    /// The optional detector noise specifications
    pub detector_noise_model: Option<NoiseDataSheet>,
}
/*impl<S: Model> WavefrontSensor for ShackHartmann<S> {
    fn calibrate(&mut self, src: &mut Source, threshold: f64) {
        self._c_.calibrate(src, threshold);
    }
}*/
/*
impl<T: ?Sized + WavefrontSensor + DerefMut> WavefrontSensor for &mut T {
    fn calibrate(&mut self, src: &mut Source, threshold: f64) {
        WavefrontSensor::calibrate(*self, src, threshold);
    }
    fn reset(&mut self) {
        WavefrontSensor::reset(*self);
    }
    fn process(&mut self) {
        WavefrontSensor::process(*self);
    }

    fn data(&mut self) -> Vec<f64> {
        WavefrontSensor::data(*self)
    }
}
*/
impl<M: Model> WavefrontSensor for ShackHartmann<M> {
    fn calibrate(&mut self, src: &mut Source, threshold: f64) {
        <M as Model>::calibrate(&mut self._c_, src, threshold);
    }
    fn reset(&mut self) {
        <M as Model>::reset(&mut self._c_);
    }
    fn process(&mut self) {
        <M as Model>::process(&mut self._c_);
    }

    fn data(&mut self) -> Vec<f64> {
        <M as Model>::data(&mut self._c_).into()
    }
    fn readout(&mut self) {
        if let Some(noise_model) = self.detector_noise_model {
            <M as Model>::readout(
                &mut self._c_,
                noise_model.exposure_time as f32,
                noise_model.rms_read_out_noise as f32,
                noise_model.n_background_photon as f32,
                noise_model.noise_factor as f32,
            );
        }
    }
    fn frame(&self) -> Option<Vec<f32>> {
        <M as Model>::frame(&self._c_)
    }
    fn n_frame(&self) -> usize {
        <M as Model>::n_frame(&self._c_)
    }
    fn valid_lenslet_from(&mut self, wfs: &mut dyn WavefrontSensor) {
        <M as Model>::valid_lenslet_from(&mut self._c_, wfs.valid_lenslet())
    }
    fn valid_lenslet(&mut self) -> &mut crate::mask {
        <M as Model>::valid_lenslet(&mut self._c_)
    }
}
impl<M: Model> ShackHartmann<M> {
    pub fn n_valid_lenslet(&mut self) -> usize {
        <M as Model>::n_valid_lenslet(&mut self._c_)
    }
    pub fn lenslet_mask(&mut self) -> Cu<Single> {
        <M as Model>::lenslet_mask(&mut self._c_)
    }
    pub fn lenslet_flux(&mut self) -> Cu<Single> {
        <M as Model>::lenslet_flux(&mut self._c_)
    }
    pub fn set_valid_lenslet(&mut self, lenslet_mask: &[i32]) {
        <M as Model>::set_valid_lenslet(&mut self._c_, lenslet_mask);
    }
    pub fn filter(&mut self, lenslet_mask: &mut Mask) -> Cu<Single> {
        <M as Model>::filter(&mut self._c_, lenslet_mask)
    }
    pub fn set_reference_slopes(&mut self, src: &mut Source) {
        <M as Model>::set_reference_slopes(&mut self._c_, src)
    }
    pub fn as_mut_ptr(&mut self) -> &mut M {
        &mut self._c_
    }
    pub fn guide_star_args(&self) -> (i32, f64, i32) {
        (
            self.n_sensor,
            self.d * self.n_side_lenslet as f64,
            self.n_px_lenslet * self.n_side_lenslet + 1,
        )
    }
    pub fn new_guide_stars(&self) -> Source {
        Source::new(
            self.n_sensor,
            self.d * self.n_side_lenslet as f64,
            self.n_px_lenslet * self.n_side_lenslet + 1,
        )
    }
}
impl ShackHartmann<Geometric> {
    pub fn fold_into(&mut self, data: &mut Cu<Single>, lenslet_mask: &mut Mask) {
        unsafe {
            self._c_
                .folded_slopes(data.as_ptr(), lenslet_mask.as_mut_prt());
        }
    }
}
impl<S: Model> Drop for ShackHartmann<S> {
    fn drop(&mut self) {
        self._c_.drop();
    }
}
impl<M: Model> Propagation for ShackHartmann<M> {
    fn propagate(&mut self, src: &mut Source) {
        <M as Model>::propagate(&mut self._c_, src);
    }
    fn time_propagate(&mut self, _secs: f64, src: &mut Source) {
        self.propagate(src)
    }
}
impl ShackHartmann<Diffractive> {
    pub fn detector_resolution(&self) -> (usize, usize) {
        let res = (self._c_.camera.N_PX_CAMERA * self._c_.camera.N_SIDE_LENSLET) as usize;
        (res, res)
    }
}

impl From<ShackHartmann<Geometric>> for Source {
    fn from(item: ShackHartmann<Geometric>) -> Self {
        item.new_guide_stars()
    }
}
impl From<ShackHartmann<Diffractive>> for Source {
    fn from(item: ShackHartmann<Diffractive>) -> Self {
        item.new_guide_stars()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Builder, FromBuilder, ShackHartmann, ShackHartmannBuilder, Source, SourceBuilder};

    #[test]
    fn shack_hartmann_geometric_new() {
        use crate::Gmt;
        let mut wfs = ShackHartmann::<Geometric>::builder()
            .n_sensor(1)
            .lenslet_array(48, 16, 25.5 / 48f64)
            .build()
            .unwrap();
        let mut src = Source::builder()
            .pupil_sampling(48 * 16 + 1)
            .build()
            .unwrap();
        let mut gmt = Gmt::builder().build().unwrap();
        src.through(&mut gmt).xpupil().through(&mut wfs);
        println!("WFE RMS: {:.3}nm", src.wfe_rms_10e(-9)[0]);
    }

    #[test]
    fn shack_hartmann_geometric_new_with_macro() {
        let mut wfs = crate::ceo!(
            ShackHartmannBuilder: Geometric,
            n_sensor = [1],
            lenslet_array = [48, 16, 25.5 / 48f64]
        );
        let mut src = crate::ceo!(SourceBuilder, pupil_sampling = [48 * 16 + 1]);
        let mut gmt = crate::ceo!(GmtBuilder);
        src.through(&mut gmt).xpupil().through(&mut wfs);
        println!("WFE RMS: {:.3}nm", src.wfe_rms_10e(-9)[0]);
    }

    /*
    #[test]
    fn shack_hartmann_diffractive_new() {
        use crate::Builder;
        use element::*;
        let mut wfs = CEO::<SHACKHARTMANN<Diffractive>>::new()
            .n_sensor(1)
            .lenslet_array(48, 16, 25.5 / 48f64)
            .detector(8, Some(24), None)
            .build();
        let mut src = CEO::<SOURCE>::new().pupil_sampling(48 * 16 + 1).build();
        let mut gmt = CEO::<GMT>::new().build();
        src.through(&mut gmt).xpupil().through(&mut wfs);
        println!("WFE RMS: {:.3}nm", src.wfe_rms_10e(-9)[0]);
    }

    #[test]
    fn shack_hartmann_diffractive_new_with_macro() {
        use element::*;
        let mut wfs = crate::ceo!(
            SHACKHARTMANN<Diffractive>,
            n_sensor = [1],
            lenslet_array = [48, 16, 25.5 / 48f64],
            detector = [8, Some(24), None]
        );
        let mut src = crate::ceo!(SOURCE, pupil_sampling = [48 * 16 + 1]);
        let mut gmt = crate::ceo!(GMT);
        src.through(&mut gmt).xpupil().through(&mut wfs);
        println!("WFE RMS: {:.3}nm", src.wfe_rms_10e(-9)[0]);
    }
    */
}
