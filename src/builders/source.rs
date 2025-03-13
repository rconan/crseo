use std::ffi::CString;

use ffi::vector;
use serde::{Deserialize, Serialize};
use skyangle::Conversion;

use crate::{
    source::{PupilSampling, PHOTOMETRY},
    Builder, Source,
};

/// `Source` builder
///
/// Default properties:
///  - size             : 1
///  - pupil size       : 25.5m
///  - pupil sampling   : 512px
///  - photometric band : V (550nm)
///  - zenith           : 0degree
///  - azimuth          : 0degree
///  - magnitude        : 0
///
/// # Examples
///
/// - on-axis source with default parameters
///
/// ```
/// use crseo::{Builder, FromBuilder, Source};
/// let mut src = Source::builder().build();
/// ```
///
/// - 3 sources evenly spread on a ring with a 8 arcminute radius
///
/// ```
/// use crseo::{Builder, FromBuilder, Source};
/// use skyangle::Conversion;
/// let mut src = Source::builder().size(3).on_ring(8f32.from_arcmin()).build();
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceBuilder {
    pub size: usize,
    pub pupil_size: f64,
    pub pupil_sampling: PupilSampling,
    pub band: String,
    pub zenith: Vec<f32>,
    pub azimuth: Vec<f32>,
    pub magnitude: Vec<f32>,
    pub rays_coordinates: Option<(Vec<f64>, Vec<f64>)>,
    pub fwhm: Option<f64>,
    pub rays_azimuth: Option<f64>,
}
impl Default for SourceBuilder {
    fn default() -> Self {
        SourceBuilder {
            size: 1,
            pupil_size: 25.5,
            pupil_sampling: PupilSampling::SquareGrid {
                size: Some(25.5),
                resolution: 512,
            },
            band: "V".into(),
            zenith: vec![0f32],
            azimuth: vec![0f32],
            magnitude: vec![0f32],
            rays_coordinates: None,
            fwhm: None,
            rays_azimuth: None,
        }
    }
}
impl SourceBuilder {
    /// Set the number of source
    ///
    /// Reset the zenith, azimuth and magnitude to their default values
    pub fn size(self, size: usize) -> Self {
        Self {
            size,
            zenith: vec![0f32; size],
            azimuth: vec![0f32; size],
            magnitude: vec![0f32; size],
            ..self
        }
    }
    /// Set the sampling of the pupil in pixels
    pub fn pupil_sampling(self, resolution: usize) -> Self {
        Self {
            pupil_sampling: PupilSampling::SquareGrid {
                size: self.pupil_sampling.size().or(Some(25.5)),
                resolution,
            },
            ..self
        }
    }
    /// Set the pupil size in meters
    pub fn pupil_size(self, pupil_size: f64) -> Self {
        Self { pupil_size, ..self }
    }
    /// Set the photometric band
    pub fn band(self, band: &str) -> Self {
        assert!(
            PHOTOMETRY
                .iter()
                .find(|photometry| band == **photometry)
                .is_some(),
            "found photometric band {band}, expected V, R, I, R+I, J, H or K"
        );
        Self {
            band: band.to_owned(),
            ..self
        }
    }
    pub fn rays_azimuth(mut self, rays_azimuth: f64) -> Self {
        self.rays_azimuth = Some(rays_azimuth);
        self
    }
    /// Set the source zenith and azimuth angles
    pub fn zenith_azimuth(self, zenith: Vec<f32>, azimuth: Vec<f32>) -> Self {
        assert_eq!(
            self.size,
            zenith.len(),
            "zenith vector must be of length {}",
            self.size
        );
        assert_eq!(
            self.size,
            azimuth.len(),
            "azimuth vector must be of length {}",
            self.size
        );
        Self {
            zenith,
            azimuth,
            ..self
        }
    }
    /// Set n sources at zenith angle evenly spread of a ring
    pub fn on_ring(self, zenith: f32) -> Self {
        Self {
            zenith: vec![zenith; self.size],
            azimuth: (0..self.size)
                .map(|x| 2. * std::f32::consts::PI * x as f32 / self.size as f32)
                .collect::<Vec<f32>>(),
            ..self
        }
    }
    /// Set the source magnitude
    pub fn magnitude(self, magnitude: Vec<f32>) -> Self {
        assert_eq!(
            self.size,
            magnitude.len(),
            "azimuth vector must be of length {}",
            self.size
        );
        Self { magnitude, ..self }
    }
    ///  Builds a star field made of 21 sources located at the vertices of a Delaunay mesh sampling a 10 arcminute field of view
    pub fn field_delaunay21(self) -> Self {
        #[derive(Deserialize)]
        struct Field {
            pub zenith_arcmin: Vec<f32>,
            pub azimuth_degree: Vec<f32>,
        }
        let Field {
            zenith_arcmin,
            azimuth_degree,
        } = serde_pickle::from_slice(include_bytes!("../fielddelaunay21.pkl"), Default::default())
            .expect("fielddelaunay21.pkl loading failed!");
        let n_src = zenith_arcmin.len();
        Self {
            size: n_src,
            zenith: zenith_arcmin
                .iter()
                .map(|x| x.from_arcmin())
                .collect::<Vec<f32>>(),
            azimuth: azimuth_degree
                .iter()
                .map(|x| x.to_radians())
                .collect::<Vec<f32>>(),
            magnitude: vec![0f32; n_src],
            ..self
        }
    }
    /// Set the \[x,y\] coordinates of the bundle of rays in the entrance pupil
    pub fn rays_coordinates(self, rays_x: Vec<f64>, rays_y: Vec<f64>) -> Self {
        assert_eq!(
            rays_x.len(),
            rays_y.len(),
            "x and y rays coordinates vector must have the same lenght"
        );
        Self {
            pupil_sampling: PupilSampling::UserSet(rays_x.len()),
            rays_coordinates: Some((rays_x, rays_y)),
            ..self
        }
    }
    /// Sets the `Source` full width at half maximum in un-binned detector pixel
    pub fn fwhm(self, value: f64) -> Self {
        Self {
            fwhm: Some(value),
            ..self
        }
    }
}
impl Builder for SourceBuilder {
    type Component = Source;
    /// Build the `Source`
    fn build(self) -> crate::Result<Self::Component> {
        let mut src = Source {
            _c_: Default::default(),
            size: self.size as i32,
            pupil_size: self.pupil_size,
            pupil_sampling: self.pupil_sampling.side() as i32,
            _wfe_rms: vec![0.0; self.size],
            _phase: vec![0.0; self.pupil_sampling.total() * self.size],
            zenith: self.zenith.clone(),
            azimuth: self.azimuth.clone(),
            magnitude: self.magnitude,
        };

        let origin = vector {
            x: 0.0,
            y: 0.0,
            z: 25.0,
        };
        let src_band = CString::new(self.band.into_bytes()).unwrap();
        if let Some((mut rays_x, mut rays_y)) = self.rays_coordinates {
            let mut zenith: Vec<_> = self.zenith.iter().map(|&x| x as f64).collect();
            let mut azimuth: Vec<_> = self.azimuth.iter().map(|&x| x as f64).collect();
            unsafe {
                src._c_.setup9(
                    src_band.into_raw(),
                    src.magnitude.as_mut_ptr(),
                    zenith.as_mut_ptr(),
                    azimuth.as_mut_ptr(),
                    f32::INFINITY,
                    self.size as i32,
                    rays_x.len() as i32,
                    rays_x.as_mut_ptr(),
                    rays_y.as_mut_ptr(),
                    origin,
                );
            }
        } else {
            unsafe {
                src._c_.setup7(
                    src_band.into_raw(),
                    src.magnitude.as_mut_ptr(),
                    src.zenith.as_mut_ptr(),
                    src.azimuth.as_mut_ptr(),
                    f32::INFINITY,
                    self.size as i32,
                    self.pupil_size,
                    self.pupil_sampling.side() as i32,
                    origin,
                );
            }
        }
        if let Some(fwhm) = self.fwhm {
            src._c_.fwhm = fwhm as f32;
        }
        if let Some(angle) = self.rays_azimuth {
            src.rotate_rays(angle)
        }
        Ok(src)
    }
}

impl From<&Source> for SourceBuilder {
    fn from(src: &Source) -> Self {
        Self {
            size: src.size as usize,
            pupil_size: src.pupil_size,
            pupil_sampling: PupilSampling::SquareGrid {
                size: Some(src.pupil_size),
                resolution: src.pupil_sampling as usize,
            },
            band: src.get_photometric_band(),
            zenith: src.zenith.clone(),
            azimuth: src.azimuth.clone(),
            magnitude: src.magnitude.clone(),
            rays_coordinates: None,
            fwhm: Some(src._c_.fwhm as f64),
            rays_azimuth: None,
        }
    }
}
