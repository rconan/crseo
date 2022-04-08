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
impl WavefrontSensor for ShackHartmann<Diffractive>
where
    Diffractive: Model,
{
    fn calibrate(&mut self, src: &mut Source, threshold: f64) {
        <Diffractive as Model>::calibrate(&mut self._c_, src, threshold);
    }
    fn reset(&mut self) -> &mut Self {
        unsafe {
            self._c_.camera.reset();
        }
        self
    }
    fn process(&mut self) -> &mut Self {
        unsafe {
            self._c_.process();
        }
        self
    }

    fn data(&mut self) -> Vec<f32> {
        self.get_data().from_dev()
    }
}
impl WavefrontSensor for ShackHartmann<Geometric>
where
    Geometric: Model,
{
    fn calibrate(&mut self, src: &mut Source, threshold: f64) {
        <Geometric as Model>::calibrate(&mut self._c_, src, threshold);
    }
    fn reset(&mut self) -> &mut Self {
        unsafe {
            self._c_.reset();
        }
        self
    }
    fn process(&mut self) -> &mut Self {
        unsafe {
            self._c_.process();
        }
        self
    }

    fn data(&mut self) -> Vec<f32> {
        self.get_data().from_dev()
    }
}
impl ShackHartmann<Geometric> {
    /// Initializes the `ShackHartmann` WFS
    pub fn build(&mut self) -> &mut Self {
        self.n_centroids = self.n_side_lenslet * self.n_side_lenslet * 2 * self.n_sensor;
        unsafe {
            self._c_
                .setup(self.n_side_lenslet, self.d as f32, self.n_sensor);
            self.centroids.from_ptr(self._c_.data_proc.d__c);
        }
        self
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
    pub fn get_data(&mut self) -> Cu<Single> {
        let m = self._c_.valid_lenslet.nnz as usize * 2usize;
        let mut data: Cu<Single> = Cu::vector(m);
        data.malloc();
        unsafe {
            self._c_.get_valid_slopes(data.as_ptr());
        }
        data
    }
    pub fn filter(&mut self, lenslet_mask: &mut Mask) -> Cu<Single> {
        let m = lenslet_mask.nnz() as usize * 2usize;
        let mut data: Cu<Single> = Cu::vector(m);
        data.malloc();
        unsafe {
            self._c_
                .masked_slopes(data.as_ptr(), lenslet_mask.as_mut_prt());
        }
        data
    }
    pub fn fold_into(&mut self, data: &mut Cu<Single>, lenslet_mask: &mut Mask) {
        unsafe {
            self._c_
                .folded_slopes(data.as_ptr(), lenslet_mask.as_mut_prt());
        }
    }
    pub fn n_valid_lenslet(&mut self) -> usize {
        self._c_.valid_lenslet.nnz as usize
    }
    pub fn lenset_mask(&mut self) -> Cu<Single> {
        let mut mask: Cu<Single> =
            Cu::vector((self.n_side_lenslet * self.n_side_lenslet * self.n_sensor) as usize);
        mask.from_ptr(self._c_.valid_lenslet.f);
        mask
    }
    pub fn valid_lenslet_from(&mut self, wfs: &mut ShackHartmann<Diffractive>) {
        unsafe {
            self._c_.valid_lenslet.reset();
            self._c_.valid_lenslet.add(&mut wfs._c_.valid_lenslet);
            self._c_.valid_actuator.reset();
            self._c_.valid_actuator.add(&mut wfs._c_.valid_actuator);
            self._c_.valid_lenslet.set_filter();
            self._c_.valid_lenslet.set_index();
            self._c_.valid_actuator.set_filter();
        }
    }
    pub fn set_reference_slopes(&mut self, src: &mut Source) {
        unsafe { self._c_.set_reference_slopes(src.as_raw_mut_ptr()) }
    }
    pub fn lenlet_flux(&mut self) -> Cu<Single> {
        let mut flux: Cu<Single> =
            Cu::vector((self.n_side_lenslet * self.n_side_lenslet * self.n_sensor) as usize);
        flux.from_ptr(self._c_.data_proc.d__mass);
        flux
    }
    pub fn as_raw_mut_ptr(&mut self) -> &mut Geometric {
        &mut self._c_
    }
}
impl<S: Model> Drop for ShackHartmann<S> {
    fn drop(&mut self) {
        self._c_.drop();
    }
}
impl Propagation for ShackHartmann<Geometric> {
    fn propagate(&mut self, src: &mut Source) -> &mut Self {
        unsafe {
            self._c_.propagate(src.as_raw_mut_ptr());
        }
        self
    }
    fn time_propagate(&mut self, _secs: f64, src: &mut Source) -> &mut Self {
        self.propagate(src)
    }
}
impl ShackHartmann<Diffractive> {
    pub fn build(
        &mut self,
        n_px_framelet: i32,
        n_px_imagelet: Option<i32>,
        osf: Option<i32>,
    ) -> &mut Self {
        let n_px = match n_px_imagelet {
            Some(n_px_imagelet) => n_px_imagelet,
            None => n_px_framelet,
        };
        let b = n_px / n_px_framelet;
        let o = osf.unwrap_or(2);
        self.n_centroids = self.n_side_lenslet * self.n_side_lenslet * 2 * self.n_sensor;
        unsafe {
            self._c_.setup(
                self.n_side_lenslet,
                self.n_px_lenslet,
                self.d as f32,
                o,
                n_px,
                b,
                self.n_sensor,
            );
            self.centroids.from_ptr(self._c_.data_proc.d__c);
        }
        self
    }
    pub fn new_guide_stars(&self) -> Source {
        Source::new(
            self.n_sensor,
            self.d * self.n_side_lenslet as f64,
            self.n_px_lenslet * self.n_side_lenslet + 1,
        )
    }
    /// Returns the centroids corresponding to the valid lenslets
    ///
    /// The first half of the valid lenslet centroids contains all the valid centroids
    /// of all the guide stars along the X–axis direction and the second half contains
    /// all the valid slopes of  all the guide stars along the Y–axis direction
    pub fn get_data(&mut self) -> Cu<Single> {
        let m = self._c_.valid_lenslet.nnz as usize * 2usize;
        let mut data: Cu<Single> = Cu::vector(m);
        data.malloc();
        unsafe {
            self._c_.get_valid_slopes(data.as_ptr());
        }
        data
    }
    pub fn n_valid_lenslet(&mut self) -> usize {
        self._c_.valid_lenslet.nnz as usize
    }
    pub fn readout(&mut self) -> &mut Self {
        if let Some(noise_model) = self.detector_noise_model {
            unsafe {
                self._c_.camera.readout1(
                    noise_model.exposure_time as f32,
                    noise_model.rms_read_out_noise as f32,
                    noise_model.n_background_photon as f32,
                    noise_model.noise_factor as f32,
                );
            }
        }
        self
    }
    pub fn detector_resolution(&self) -> (usize, usize) {
        let res = (self._c_.camera.N_PX_CAMERA * self._c_.camera.N_SIDE_LENSLET) as usize;
        (res, res)
    }
    pub fn frame(&mut self) -> Vec<f32> {
        let n =
            self._c_.camera.N_PX_CAMERA * self._c_.camera.N_PX_CAMERA * self._c_.camera.N_LENSLET;
        let m = self._c_.camera.N_SOURCE;
        let mut data: Cu<Single> = Cu::array(n as usize, m as usize);
        data.from_ptr(self._c_.camera.d__frame);
        data.into()
    }
}
impl Propagation for ShackHartmann<Diffractive> {
    fn propagate(&mut self, src: &mut Source) -> &mut Self {
        unsafe {
            self._c_.propagate(src.as_raw_mut_ptr());
        }
        self
    }
    fn time_propagate(&mut self, _secs: f64, src: &mut Source) -> &mut Self {
        self.propagate(src)
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
    use crate::{Builder, SHACKHARTMANN, SOURCE};

    #[test]
    fn shack_hartmann_geometric_new() {
        use crate::GMT;
        let mut wfs = SHACKHARTMANN::<Geometric>::new()
            .n_sensor(1)
            .lenslet_array(48, 16, 25.5 / 48f64)
            .build()
            .unwrap();
        let mut src = SOURCE::new().pupil_sampling(48 * 16 + 1).build().unwrap();
        let mut gmt = GMT::new().build().unwrap();
        src.through(&mut gmt).xpupil().through(&mut wfs);
        println!("WFE RMS: {:.3}nm", src.wfe_rms_10e(-9)[0]);
    }

    #[test]
    fn shack_hartmann_geometric_new_with_macro() {
        let mut wfs = crate::ceo!(
            SHACKHARTMANN: Geometric,
            n_sensor = [1],
            lenslet_array = [48, 16, 25.5 / 48f64]
        );
        let mut src = crate::ceo!(SOURCE, pupil_sampling = [48 * 16 + 1]);
        let mut gmt = crate::ceo!(GMT);
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
