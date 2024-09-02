use std::ops::DerefMut;

use skyangle::Conversion;

use crate::{gmt::GmtBuilder, source::SourceBuilder, Builder};

use super::SegmentPistonSensor;

/// Segment piston sensor builder
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SegmentPistonSensorBuilder {
    gmt_builder: GmtBuilder,
    src_builder: SourceBuilder,
    lenslet_size: f64,
    dispersion: f64,
    field_of_view: f64,
    nyquist_factor: f64,
    bin_image: usize,
    malloc_dft: bool,
    middle_mask_width: Option<f64>,
}

impl Default for SegmentPistonSensorBuilder {
    fn default() -> Self {
        Self {
            gmt_builder: GmtBuilder::default(),
            src_builder: SourceBuilder::default(),
            lenslet_size: 1.5,
            dispersion: 5.0.from_arcsec() * 1e6,
            field_of_view: 3.0.from_arcsec(),
            nyquist_factor: 1.0,
            bin_image: 2,
            malloc_dft: true,
            middle_mask_width: None,
        }
    }
}

impl SegmentPistonSensorBuilder {
    pub fn gmt(mut self, gmt: GmtBuilder) -> Self {
        self.gmt_builder = gmt;
        self
    }
    pub fn src(mut self, src: SourceBuilder) -> Self {
        self.src_builder = src;
        self
    }
    pub fn lenslet_size(mut self, lenslet_size: f64) -> Self {
        self.lenslet_size = lenslet_size;
        self
    }
    pub fn dispersion(mut self, dispersion: f64) -> Self {
        self.dispersion = dispersion;
        self
    }
    pub fn field_of_view(mut self, field_of_view: f64) -> Self {
        self.field_of_view = field_of_view;
        self
    }
    pub fn nyquist_factor(mut self, nyquist_factor: f64) -> Self {
        self.nyquist_factor = nyquist_factor;
        self
    }
    pub fn bin_image(mut self, bin_image: usize) -> Self {
        self.bin_image = bin_image;
        self
    }
    pub fn malloc_dft(mut self, malloc_dft: bool) -> Self {
        self.malloc_dft = malloc_dft;
        self
    }
    pub fn middle_mask_width(mut self, middle_mask_width: f64) -> Self {
        self.middle_mask_width = Some(middle_mask_width);
        self
    }
}

impl Builder for SegmentPistonSensorBuilder {
    type Component = SegmentPistonSensor;

    fn build(self) -> crate::Result<Self::Component> {
        let Self {
            gmt_builder,
            src_builder,
            lenslet_size,
            dispersion,
            field_of_view,
            nyquist_factor,
            bin_image,
            malloc_dft,
            middle_mask_width,
        } = self;
        let mut gmt = gmt_builder.build()?;
        let mut src = src_builder.build()?;
        let mut sps = SegmentPistonSensor {
            _c_: Default::default(),
            malloc_dft,
            middle_mask_width,
        };
        if malloc_dft {
            unsafe {
                sps._c_.setup3(
                    gmt.m1.deref_mut(),
                    src.as_raw_mut_ptr(),
                    lenslet_size as f32,
                    dispersion as f32,
                    field_of_view as f32,
                    nyquist_factor as f32,
                    bin_image as i32,
                )
            }
        } else {
            unsafe {
                sps._c_.setup_alt(
                    gmt.m1.deref_mut(),
                    src.as_raw_mut_ptr(),
                    dispersion as f32,
                    field_of_view as f32,
                    nyquist_factor as f32,
                    bin_image as i32,
                )
            }
        }

        Ok(sps)
    }
}
