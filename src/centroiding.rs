use crate::{builders::centroiding::CentroidingBuilder, Builder, FromBuilder};

use ffi::{centroiding, dev2host, host2dev_char, mask};

use crate::imaging::Frame;

/// Wrapper for CEO centroiding
///
/// The x and y centroids are stored as `[[cx,cy]_1, ... , [cx,cy]_n]` for `n` guide stars
pub struct Centroiding {
    pub(crate) _c_: centroiding,
    pub(crate) _c_mask_: mask,
    /// The total number of lenslets
    pub n_lenslet_total: usize,
    /// The number of centroids
    pub n_centroids: usize,
    /// The centroid units, default: 1 (pixel)
    pub units: f32,
    pub(crate) flux: Vec<f32>,
    /// The valid lenslet mask
    pub valid_lenslets: Vec<i8>,
    /// The number of valid lenslet per guide star
    pub n_valid_lenslet: Vec<usize>,
    /// The centroids
    pub centroids: Vec<f32>,
    pub(crate) xy_mean: Option<Vec<(f32, f32)>>,
}

impl FromBuilder for Centroiding {
    type ComponentBuilder = CentroidingBuilder;
}

impl Centroiding {
    // /// Creates a new `Centroiding`
    // pub fn new() -> Centroiding {
    //     Centroiding {
    //         _c_: Default::default(),
    //         _c_mask_: Default::default(),
    //         n_lenslet_total: 0,
    //         n_centroids: 0,
    //         units: 1f32,
    //         flux: vec![],
    //         valid_lenslets: vec![],
    //         n_valid_lenslet: 0,
    //         centroids: vec![],
    //     }
    // }
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
            }
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
    /// Removes the mean from the X and Y centroids
    pub fn remove_mean(&mut self, some_valid_lenslets: Option<&Vec<i8>>) -> &mut Self {
        let valid_lenslets = some_valid_lenslets.unwrap_or_else(|| &self.valid_lenslets);
        let xy_mean: Vec<_> = valid_lenslets
            .chunks(self.n_lenslet_total)
            .zip(self.centroids.chunks_mut(self.n_lenslet_total * 2))
            .map(|(v, c)| {
                let vc = c
                    .iter()
                    .zip(v.iter().cycle())
                    .filter_map(|(c, v)| if v.is_positive() { Some(*c) } else { None })
                    .collect::<Vec<_>>();
                let n = vc.len() / 2;
                let (x, y) = vc.split_at(n);
                let x_mean = x.iter().sum::<f32>() / n as f32;
                let y_mean = y.iter().sum::<f32>() / n as f32;
                let (cx, cy) = c.split_at_mut(self.n_lenslet_total);
                cx.iter_mut()
                    .zip(cy.iter_mut())
                    .zip(v.iter())
                    .for_each(|((cx, cy), v)| {
                        if v.is_positive() {
                            *cx -= x_mean;
                            *cy -= y_mean;
                        }
                    });
                (x_mean, y_mean)
            })
            .collect();
        self.xy_mean = Some(xy_mean);
        self
    }
    /// Returns the valid `centroids` i.e. the `centroids` that corresponds to a non-zero entry in the `valid_lenslets` mask
    ///
    ///  if `some_valid_lenslet` is given, then it supersedes any preset `valid_lenset`
    pub fn valids(&self, some_valid_lenslets: Option<&Vec<i8>>) -> Vec<Vec<f32>> {
        let mut valid_lenslets = some_valid_lenslets
            .unwrap_or_else(|| &self.valid_lenslets)
            .chunks(self.n_lenslet_total);

        self.centroids
            .chunks(self.n_lenslet_total * 2)
            .map(|c| {
                c.iter()
                    .zip(valid_lenslets.next().unwrap().iter().cycle())
                    .filter_map(|(c, v)| if v.is_positive() { Some(*c) } else { None })
                    .collect::<Vec<_>>()
            })
            .collect()
    }
    /// returns the flux of each lenslet
    pub fn lenslet_flux(&self) -> &Vec<f32> {
        unsafe {
            dev2host(
                self.flux.as_ptr() as *mut _,
                self._c_.d__mass,
                self.flux.len() as i32,
            );
        }
        &self.flux
    }
    pub fn lenslet_array_flux(&mut self) -> Vec<f32> {
        self.lenslet_flux()
            .chunks(self.n_lenslet_total)
            .map(|x| x.iter().sum())
            .collect::<Vec<_>>()
    }
    /// Returns the sum of the flux of all the lenslets
    pub fn integrated_flux(&mut self) -> f32 {
        self.lenslet_flux().iter().sum()
    }
    /// Computes the valid lenslets and return the number of valid lenslets
    ///
    /// the valid lenslets are computed based on the maximum flux threshold or a given valid lenslets mask
    pub fn valid_lenslets(
        &mut self,
        some_flux_threshold: Option<f64>,
        some_valid_lenslets: Option<Vec<i8>>,
    ) -> &mut Self {
        if let Some(some_flux_threshold) = some_flux_threshold {
            let n = self.n_lenslet_total;
            let lenslet_flux = self.lenslet_flux();
            // dbg!(lenslet_flux.len());
            // let lenslet_flux_max = lenslet_flux.iter().cloned().fold(0.0, f32::max);
            // let threshold_flux = dbg!(lenslet_flux_max) * some_flux_threshold.unwrap() as f32;
            // dbg!(lenslet_flux
            //     .iter()
            //     .map(|x| if x >= &threshold_flux { 1i8 } else { 0i8 })
            //     .filter(|x| x.is_positive())
            //     .count());
            // self.valid_lenslets = lenslet_flux
            //     .iter()
            //     .map(|x| if x >= &threshold_flux { 1i8 } else { 0i8 })
            //     .collect();
            self.valid_lenslets = lenslet_flux
                .chunks(n)
                .flat_map(|flux| {
                    let threshold_flux = flux
                        .iter()
                        .max_by(|a, b| a.partial_cmp(b).unwrap())
                        .unwrap()
                        * some_flux_threshold as f32;
                    flux.iter()
                        .map(|x| if x >= &threshold_flux { 1i8 } else { 0i8 })
                        .collect::<Vec<_>>()
                })
                .collect();
        }
        if some_valid_lenslets.is_some() {
            self.valid_lenslets = some_valid_lenslets.unwrap();
        }
        unsafe {
            host2dev_char(
                self._c_mask_.m,
                self.valid_lenslets.as_mut_ptr(),
                self.valid_lenslets.len() as i32,
            );
            // self._c_mask_.reset();
            // self._c_mask_
            //     .add1(self._c_.lenslet_mask, self.n_lenslet_total as i32);
            self._c_mask_.set_filter();
        }

        // dbg!((self.valid_lenslets.len(), self.n_lenslet_total));

        self.n_valid_lenslet = self
            .valid_lenslets
            .chunks(self.n_lenslet_total)
            .map(|x| x.iter().filter(|x| x.is_positive()).count())
            .collect();
        self
    }
    pub fn n_valid_lenslet_total(&self) -> usize {
        self.n_valid_lenslet.iter().sum()
    }
    /// Returns the `Centroiding` components (ref, mask
    pub fn __ceo__(&mut self) -> (&centroiding, &mask) {
        (&self._c_, &self._c_mask_)
    }
    pub fn __mut_ceo__(&mut self) -> (&mut centroiding, &mut mask) {
        (&mut self._c_, &mut self._c_mask_)
    }
}

impl Default for Centroiding {
    fn default() -> Self {
        Self::builder().build().unwrap()
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

    use skyangle::Conversion;

    use super::*;
    use crate::{imaging::LensletArray, Builder, Gmt, Imaging, Source};

    #[test]
    fn shackhartmann() {
        env::set_var("GMT_MODES_PATH", "/home/ubuntu/CEO/gmtMirrors/");

        let n_side_lenslet = 6;
        let n_px_lenslet = 16;
        let pupil_sampling = n_side_lenslet * n_px_lenslet + 1;
        let mut gmt = Gmt::builder().build().unwrap();
        let n_gs = 2;
        let mut src = Source::builder()
            .pupil_sampling(pupil_sampling)
            .size(n_gs)
            .zenith_azimuth(vec![0f32; n_gs], vec![0f32; n_gs])
            .build()
            .unwrap();

        let sensor = Imaging::builder()
            .lenslet_array(
                LensletArray::default()
                    .n_side_lenslet(n_side_lenslet)
                    .n_px_lenslet(n_px_lenslet),
            )
            .n_sensor(n_gs);

        let mut cog0 = CentroidingBuilder::from(&sensor).build().unwrap();
        let mut cog = CentroidingBuilder::from(&sensor).build().unwrap();
        let mut sensor = sensor.build().unwrap();

        src.through(&mut gmt).xpupil().through(&mut sensor);
        cog0.process(&sensor.frame(), None)
            .valid_lenslets(Some(0.85), None);

        src.through(&mut gmt).xpupil().through(sensor.reset());
        cog.process(&sensor.frame(), Some(&cog0)).grab();

        let m2_rbm = vec![vec![0f64, 0f64, 0f64, 1e-6, 0f64, 0f64]; 7];
        let mut m2_rbm = vec![vec![0f64; 6]; 7];
        m2_rbm[0][3] = 1f64.from_arcsec();
        gmt.update(None, Some(&m2_rbm), None, None);
        src.through(&mut gmt).xpupil().through(sensor.reset());
        dbg!(src.wfe_rms());
        cog.process(&sensor.frame(), Some(&cog0)).grab();

        {
            println!("centroids");
            for (i, c) in cog.centroids.chunks(n_side_lenslet.pow(2) * 2).enumerate() {
                println!("GS #{}", i + 1);
                let (cx, cy) = c.split_at(n_side_lenslet * n_side_lenslet);
                cx.iter()
                    .zip(cy)
                    .for_each(|(x, y)| println!("{:+.3} {:+.3}", x, y));
            }
        }

        let v = cog.valids(Some(&cog0.valid_lenslets));
        {
            println!("valid centroids");
            for (i, v) in v.iter().enumerate() {
                println!("GS #{}", i + 1);
                let (cx, cy) = v.split_at(v.len() / 2);
                cx.iter()
                    .zip(cy)
                    .for_each(|(x, y)| println!("{:+.3} {:+.3}", x, y));
            }
        }

        let v0 = cog
            .remove_mean(Some(&cog0.valid_lenslets))
            .valids(Some(&cog0.valid_lenslets));
        dbg!(&cog.xy_mean);
        {
            println!("zero mean valid centroids");
            for (i, v) in v0.iter().enumerate() {
                println!("GS #{}", i + 1);
                let (cx, cy) = v.split_at(v.len() / 2);
                cx.iter()
                    .zip(cy)
                    .for_each(|(x, y)| println!("{:+.3} {:+.3}", x, y));
            }
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
