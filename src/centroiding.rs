use crate::FromBuilder;

use ffi::{centroiding, dev2host, host2dev_char, mask};

mod builder;
use crate::imaging::Frame;
pub use builder::CentroidingBuilder;

/// Wrapper for CEO centroiding
pub struct Centroiding {
    _c_: centroiding,
    _c_mask_: mask,
    /// The total number of lenslets
    pub n_lenslet_total: u32,
    /// The number of centroids
    pub n_centroids: u32,
    /// The centroid units, default: 1 (pixel)
    pub units: f32,
    flux: Vec<f32>,
    /// The valid lenslet mask
    pub valid_lenslets: Vec<i8>,
    /// The number of valid lenslet
    pub n_valid_lenslet: u32,
    /// The centroids
    pub centroids: Vec<f32>,
}

impl FromBuilder for Centroiding {
    type ComponentBuilder = CentroidingBuilder;
}

impl Centroiding {
    /// Creates a new `Centroiding`
    pub fn new() -> Centroiding {
        Centroiding {
            _c_: Default::default(),
            _c_mask_: Default::default(),
            n_lenslet_total: 0u32,
            n_centroids: 0u32,
            units: 1f32,
            flux: vec![],
            valid_lenslets: vec![],
            n_valid_lenslet: 0u32,
            centroids: vec![],
        }
    }
    /*     /// Sets the `Centroiding` parameters:
    ///
    /// * `n_lenslet` - the size of the square lenslet array
    /// * `data_units` - the centroids units
    pub fn build(&mut self, n_lenslet: u32, data_units: Option<f64>) -> &mut Self {
        self.n_lenslet_total = n_lenslet * n_lenslet;
        self.n_centroids = 2 * self.n_lenslet_total;
        self.n_valid_lenslet = self.n_lenslet_total;
        unsafe {
            self._c_.setup(n_lenslet as i32, 1);
            self._c_mask_.setup(self.n_lenslet_total as i32);
        }
        self.flux = vec![0.0; self.n_lenslet_total as usize];
        self.centroids = vec![0.0; self.n_centroids as usize];
        self.units = data_units.or(Some(1f64)).unwrap() as f32;
        self
    } */
    /// Computes the `centroids` from the `sensor` image; optionally, a `Centroiding` `reference` may be provided that offsets the `centroids` and sets the `valid_lenslets`
    pub fn process(&mut self, frame: &Frame, reference: Option<&Centroiding>) -> &mut Self {
        match reference {
            None => unsafe {
                self._c_.get_data2(
                    frame.dev.as_ptr() as *mut _,
                    frame.n_px_camera as i32,
                    0.0,
                    0.0,
                    self.units,
                );
            },
            Some(r) => {
                assert_eq!(self.n_lenslet_total, r.n_lenslet_total);
                unsafe {
                    self._c_.get_data3(
                        frame.dev.as_ptr() as *mut _,
                        frame.n_px_camera as i32,
                        r._c_.d__cx,
                        r._c_.d__cy,
                        self.units,
                        r._c_mask_.m,
                    );
                }
            } /*
              let n = r.n_lenslet_total as usize;
              let cx = &self.centroids[..n];
              let cy = &self.centroids[n..];
              let mut  vcx: Vec<f32> = r
                  .valid_lenslets
                  .iter()
                  .zip(cx.iter())
                  .filter(|x| x.0.is_positive())
                  .map(|x| *x.1)
                  .collect();
              let mut vcy: Vec<f32> = r
                  .valid_lenslets
                  .iter()
                  .zip(cy.iter())
                  .filter(|x| x.0.is_positive())
                  .map(|x| *x.1)
                  .collect();
              vcx.append(&mut vcy);
              */
        };
        self
    }
    /// grabs the `centroids` from the GPU
    pub fn grab(&mut self) -> &mut Self {
        unsafe {
            dev2host(
                self.centroids.as_mut_ptr(),
                self._c_.d__c,
                self.n_centroids as i32,
            );
        }
        self
    }
    /// returns the valid `centroids` i.e. the `centroids` that corresponds to a non-zero entry in the `valid_lenslets` mask; if `some_valid_lenslet` is given, then it supersedes any preset `valid_lenset`
    pub fn valids(&self, some_valid_lenslets: Option<&Vec<i8>>) -> Vec<f32> {
        let valid_lenslets = some_valid_lenslets.unwrap_or_else(|| &self.valid_lenslets);
        assert_eq!(self.n_lenslet_total, valid_lenslets.len() as u32);
        let n = valid_lenslets.iter().fold(0u32, |a, x| a + (*x as u32)) as usize;
        let mut valid_centroids: Vec<f32> = vec![0f32; 2 * n];
        let mut l = 0;
        for (k, v) in valid_lenslets.iter().enumerate() {
            if *v > 0 {
                valid_centroids[l] = self.centroids[k];
                valid_centroids[l + n] = self.centroids[k + valid_lenslets.len()];
                l += 1;
            }
        }
        return valid_centroids;
    }
    /// returns the flux of each lenslet
    pub fn lenslet_flux(&mut self) -> &Vec<f32> {
        unsafe {
            dev2host(
                self.flux.as_mut_ptr(),
                self._c_.d__mass,
                self.n_lenslet_total as i32,
            );
        }
        &self.flux
    }
    /// returns the sum of the flux of all the lenslets
    pub fn integrated_flux(&mut self) -> f32 {
        self.lenslet_flux().iter().sum()
    }
    /// Computes the valid lenslets and return the number of valid lenslets; the valid lenslets are computed based on the maximum flux threshold or a given valid lenslets mask
    pub fn valid_lenslets(
        &mut self,
        some_flux_threshold: Option<f64>,
        some_valid_lenslets: Option<Vec<i8>>,
    ) -> u32 {
        if some_flux_threshold.is_some() {
            let lenslet_flux = self.lenslet_flux();
            let lenslet_flux_max = lenslet_flux.iter().cloned().fold(0.0, f32::max);
            let threshold_flux = lenslet_flux_max * some_flux_threshold.unwrap() as f32;
            self.valid_lenslets = lenslet_flux
                .iter()
                .map(|x| if x >= &threshold_flux { 1i8 } else { 0i8 })
                .collect();
        }
        if some_valid_lenslets.is_some() {
            self.valid_lenslets = some_valid_lenslets.unwrap();
        }
        unsafe {
            host2dev_char(
                self._c_.lenslet_mask,
                self.valid_lenslets.as_mut_ptr() as *mut _,
                self.n_lenslet_total as i32,
            );
            self._c_mask_.reset();
            self._c_mask_
                .add1(self._c_.lenslet_mask, self.n_lenslet_total as i32);
        }
        self.n_valid_lenslet = self
            .valid_lenslets
            .iter()
            .fold(0u32, |a, x| a + (*x as u32));
        self.n_valid_lenslet
    }
    pub fn __ceo__(&mut self) -> (&centroiding, &mask) {
        (&self._c_, &self._c_mask_)
    }
    pub fn __mut_ceo__(&mut self) -> (&mut centroiding, &mut mask) {
        (&mut self._c_, &mut self._c_mask_)
    }
}

impl Default for Centroiding {
    fn default() -> Self {
        Self::new()
    }
}
impl Drop for Centroiding {
    /// Frees CEO memory before dropping `Centroiding`
    fn drop(&mut self) {
        unsafe {
            self._c_.cleanup();
            self._c_mask_.cleanup();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use super::*;
    use crate::{
        ceo, imaging::LensletArray, wavefrontsensor::shackhartmann::sensor, Builder, Conversion,
        Gmt, Imaging, Source,
    };

    #[test]
    fn shackhartmann() {
        env::set_var("GMT_MODES_PATH", "/home/ubuntu/CEO/gmtMirrors/");

        let pupil_size = 25.5f64;
        let n_side_lenslet = 6;
        let n_px_lenslet = 16;
        let pupil_sampling = n_side_lenslet * n_px_lenslet + 1;
        let mut gmt = Gmt::builder().build().unwrap();
        let n_gs = 2;
        let mut src = Source::builder()
            .pupil_sampling(pupil_sampling)
            .size(n_gs)
            .build()
            .unwrap();

        let sensor = Imaging::builder().lenslet_array(
            LensletArray::default()
                .n_side_lenslet(n_side_lenslet)
                .n_px_lenslet(n_px_lenslet),
        );

        let mut cog0 = CentroidingBuilder::from(&sensor).build().unwrap();
        let mut cog = CentroidingBuilder::from(&sensor).build().unwrap();
        let mut sensor = sensor.build().unwrap();

        src.through(&mut gmt).xpupil().through(&mut sensor);
        cog0.process(&sensor.frame(), None)
            .valid_lenslets(Some(0.85), None);

        src.through(&mut gmt).xpupil().through(sensor.reset());
        cog.process(&sensor.frame(), Some(&cog0)).grab();

        let m2_rbm = vec![vec![0f64, 0f64, 0f64, 1e-6, 0f64, 0f64]; 7];
        gmt.update(None, Some(&m2_rbm), None, None);
        src.through(&mut gmt).xpupil().through(sensor.reset());
        cog.process(&sensor.frame(), Some(&cog0)).grab();

        {
            println!("centroids");
            let (cx, cy) = cog.centroids.split_at(cog.centroids.len() / 2);
            cx.iter()
                .zip(cy)
                .for_each(|(x, y)| println!("{:+.3} {:+.3}", x, y));
        }

        let v = cog.valids(Some(&cog0.valid_lenslets));
        {
            println!("valid centroids");
            let (cx, cy) = v.split_at(v.len() / 2);
            cx.iter()
                .zip(cy)
                .for_each(|(x, y)| println!("{:+.3} {:+.3}", x, y));
        }

        // let mut cog = Centroiding::new();
        // cog.build(n_side_lenslet as u32, Some(p))
        //     .valid_lenslets(None, Some(cog0.valid_lenslets.clone()));
        // src.through(&mut gmt).xpupil().lenslet_gradients(
        //     n_side_lenslet,
        //     lenslet_size as f64,
        //     &mut cog,
        // );
        // let s0 = cog.grab().valids(None);

        // let m2_rbm = vec![vec![0f64, 0f64, 0f64, 1e-6, 1e-6, 0f64]; 7];
        // gmt.update(None, Some(&m2_rbm), None, None);
        // sensor.reset();
        // src.through(&mut gmt).xpupil().through(&mut sensor);
        // let c = cog.process(&sensor, Some(&cog0)).grab().valids(None);
        // src.lenslet_gradients(n_side_lenslet, lenslet_size as f64, &mut cog);
        // let s = cog
        //     .grab()
        //     .valids(None)
        //     .iter()
        //     .zip(s0.iter())
        //     .map(|x| x.0 - x.1)
        //     .collect::<Vec<f32>>();

        // let e = ((c
        //     .iter()
        //     .zip(s.iter())
        //     .map(|x| (x.0 - x.1).powi(2))
        //     .sum::<f32>()
        //     / nv as f32)
        //     .sqrt() as f64)
        //     .to_mas();
        // println!("Centroid error: {}mas", e);
        // assert!(e < 5f64);
    }
}

/* #[cfg(test)]
mod tests {
    use super::*;
    use crate::{imaging, Builder};
    use std::error::Error;

    #[test]
    pub fn centroids() -> Result<(), Box<dyn Error>> {
        let n_lenslet = 1;
        let n_px = 4;
        let mut cog = Centroiding::builder().n_lenslet(n_lenslet).build()?;
        dbg!(&cog.valid_lenslets);
        dbg!(&cog.centroids);

        // let x: Vec<_> = vec![1f32; (n_lenslet * n_px ).pow(2)];
        let px: Vec<_> = vec![
            (0..n_lenslet)
                .flat_map(|l| {
                    (0..n_px)
                        .map(|i| {
                            let h = (n_px - 1) as f32 * 0.5 + 1f32;
                            if i as f32 == h {
                                0f32
                            } else {
                                if (i as f32) < h {
                                    -1f32
                                } else {
                                    1f32
                                }
                            }
                        })
                        .collect::<Vec<f32>>()
                })
                .collect::<Vec<_>>();
            n_lenslet * n_px
        ]
        .into_iter()
        .flatten()
        .collect();

        dbg!(&px);

        let frame = imaging::Frame::new(px, n_lenslet * n_px, n_px-1);
        let c = &cog.process(&frame, None).grab().centroids;
        dbg!(&c);

        Ok(())
    }
}
 */
