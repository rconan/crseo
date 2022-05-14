use super::{cu, Builder, Result};
use ffi::{bundle, conic, intersect, reflect, refract, transform_to_R, transform_to_S};

pub struct CONIC {
    curvature_radius: f64,
    conic_cst: f64,
    origin: [f64; 3],
    euler_angles: [f64; 3],
    conic_origin: [f64; 3],
}
impl Default for CONIC {
    fn default() -> Self {
        Self {
            curvature_radius: 1f64,
            conic_cst: -1f64,
            origin: [0f64; 3],
            euler_angles: [0f64; 3],
            conic_origin: [0f64; 3],
        }
    }
}
impl CONIC {
    pub fn curvature_radius(self, curvature_radius: f64) -> Self {
        Self {
            curvature_radius,
            ..self
        }
    }
    pub fn conic_cst(self, conic_cst: f64) -> Self {
        Self { conic_cst, ..self }
    }
    pub fn origin(self, origin: [f64; 3]) -> Self {
        Self { origin, ..self }
    }
    pub fn conic_origin(self, conic_origin: [f64; 3]) -> Self {
        Self {
            origin: conic_origin,
            ..self
        }
    }
    pub fn euler_angles(self, euler_angles: [f64; 3]) -> Self {
        Self {
            euler_angles,
            ..self
        }
    }
}
impl Builder for CONIC {
    type Component = Conic;
    fn build(self) -> Result<Self::Component> {
        let mut optics = Conic::default();
        unsafe {
            optics._c_.setup2(
                self.curvature_radius.recip(),
                self.conic_cst,
                self.origin.into(),
                self.euler_angles.into(),
                self.conic_origin.into(),
            );
        }
        Ok(optics)
    }
}
#[derive(Default)]
pub struct Conic {
    _c_: conic,
}
impl Drop for Conic {
    fn drop(&mut self) {
        unsafe {
            self._c_.cleanup();
        }
    }
}

#[derive(Default)]
pub struct RAYS {
    pub zenith: f64,
    pub azimuth: f64,
    pub xy: Vec<f64>,
    pub origin: [f64; 3],
}
impl RAYS {
    pub fn zenith(self, zenith: f64) -> Self {
        Self { zenith, ..self }
    }
    pub fn azimuth(self, azimuth: f64) -> Self {
        Self { azimuth, ..self }
    }
    pub fn xy(self, xy: Vec<f64>) -> Self {
        Self { xy, ..self }
    }
    pub fn origin(self, origin: [f64; 3]) -> Self {
        Self { origin, ..self }
    }
}
impl Builder for RAYS {
    type Component = Rays;
    fn build(self) -> Result<Self::Component> {
        let mut rays = Rays::default();
        let (mut x, mut y): (Vec<_>, Vec<_>) = self.xy.chunks(2).map(|xy| (xy[0], xy[1])).unzip();
        let n_ray = x.len();
        unsafe {
            rays._c_.setup_free1(
                self.zenith,
                self.azimuth,
                n_ray as i32,
                x.as_mut_ptr(),
                y.as_mut_ptr(),
                self.origin.into(),
            );
        }
        Ok(rays)
    }
}
#[derive(Default)]
pub struct Rays {
    _c_: bundle,
}

impl Rays {
    pub fn intersect(&mut self, optics: &mut Conic) -> &mut Self {
        unsafe {
            intersect(&mut self._c_, &mut optics._c_);
        }
        self
    }
    pub fn reflect(&mut self) -> &mut Self {
        unsafe {
            reflect(&mut self._c_);
        }
        self
    }
    pub fn refract(&mut self, mu: f64) -> &mut Self {
        unsafe {
            refract(&mut self._c_, mu);
        }
        self
    }
    pub fn into_optics(&mut self, optics: &mut Conic) -> &mut Self {
        unsafe {
            transform_to_S(&mut self._c_, &mut optics._c_);
        }
        self
    }
    pub fn from_optics(&mut self, optics: &mut Conic) -> &mut Self {
        unsafe {
            transform_to_R(&mut self._c_, &mut optics._c_);
        }
        self
    }
    pub fn n_ray(&self) -> usize {
        self._c_.N_RAY as usize
    }
    pub fn coordinates(&mut self) -> Vec<f64> {
        let mut data = cu::Cu::<cu::Double>::vector(3 * self.n_ray());
        data.malloc();
        unsafe {
            self._c_.get_coordinates(data.as_mut_ptr());
        }
        data.into()
    }
    pub fn directions(&mut self) -> Vec<f64> {
        let mut data = cu::Cu::<cu::Double>::vector(3 * self.n_ray());
        data.malloc();
        unsafe {
            self._c_.get_directions(data.as_mut_ptr());
        }
        data.into()
    }
    pub fn optical_path_difference(&mut self) -> Vec<f64> {
        let mut data = cu::Cu::<cu::Double>::vector(self.n_ray());
        data.malloc();
        unsafe {
            self._c_.get_optical_path_difference(data.as_mut_ptr());
        }
        data.into()
    }
    pub fn fmt<S: Into<String>>(&mut self, msg: Option<S>) -> String {
        let mut data_fmt: Vec<_> = self
            .coordinates()
            .chunks(3)
            .zip(self.directions().chunks(3))
            .enumerate()
            .map(|(k, (coords, dirs))| {
                format!(
                    "#{:03} coords: {:+12.6?}\n#{:03} dirs:   {:+12.6?}",
                    k + 1,
                    coords,
                    k + 1,
                    dirs
                )
            })
            .collect();
        if let Some(msg) = msg {
            data_fmt.insert(0, msg.into());
        }
        data_fmt.join("\n")
    }
    pub fn println<S: Into<String>>(&mut self, msg: Option<S>) -> &mut Self {
        println!("{}", self.fmt(msg));
        self
    }
}
impl Drop for Rays {
    fn drop(&mut self) {
        unsafe { self._c_.cleanup() }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn single_ray() {
        let mut m1 = CONIC::builder()
            .curvature_radius(36.)
            .conic_cst(1. - 0.9982857)
            .build()
            .unwrap();
        let mut m2 = CONIC::builder()
            .curvature_radius(-4.1639009)
            .conic_cst(1. - 0.71692784)
            .conic_origin([0., 0., 20.26247614])
            .build()
            .unwrap();
        let mut rays = RAYS::builder()
            .xy(vec![0., 0.])
            .origin([0., 0., 25.])
            .build()
            .unwrap();
        rays.println(Some(format!("rays [{}]", rays.n_ray())))
            .into_optics(&mut m1)
            .intersect(&mut m1)
            .println(Some("M1 intersection"))
            .reflect()
            .from_optics(&mut m1)
            .println(Some("M1 reflection"))
            .into_optics(&mut m2)
            .intersect(&mut m2)
            .println(Some("M2 intersection"))
            .reflect()
            .from_optics(&mut m2)
            .println(Some("M2 reflection"));
    }
    #[test]
    fn multiple_ray() {
        let mut m1 = CONIC::builder()
            .curvature_radius(36.)
            .conic_cst(1. - 0.9982857)
            .build()
            .unwrap();
        let mut m2 = CONIC::builder()
            .curvature_radius(-4.1639009)
            .conic_cst(1. - 0.71692784)
            .conic_origin([0., 0., 20.26247614])
            .build()
            .unwrap();
        let mut rays = RAYS::builder()
            .xy([0., 0.].repeat(1_000_000))
            .origin([0., 0., 25.])
            .build()
            .unwrap();
        println!("ray #: {}", rays.n_ray());
        let now = Instant::now();
        rays.into_optics(&mut m1)
            .intersect(&mut m1)
            .reflect()
            .from_optics(&mut m1)
            .into_optics(&mut m2)
            .intersect(&mut m2)
            .reflect()
            .from_optics(&mut m2);
        println!("Elapsed time: {}mus", now.elapsed().as_micros());
    }
}
