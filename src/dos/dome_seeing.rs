use crate::{cu::Single, Cu, Propagation, Source};
use cirrus;
use std::{cell::RefCell, rc::Rc};

/// Dome seeing model
pub struct DomeSeeing {
    pub region: String,
    pub bucket: String,
    pub folder: String,
    pub case: String,
    pub keys: Vec<String>,
    pub n_keys: usize,
    pub opd: Vec<Vec<f32>>,
    pub current_opd: Rc<RefCell<Vec<f32>>>,
    step: usize,
    pub n_step: usize,
    buffer: Cu<Single>,
    duration: usize,
    rate: usize,
    time: Vec<f64>,
    pub sampling_time: f64,
    pub current_time: f64,
}
impl DomeSeeing {
    /// Creates a new dome seeing model
    pub fn new(
        region: &str,
        bucket: &str,
        folder: &str,
        case: &str,
        duration: usize,
        rate: Option<usize>,
    ) -> Self {
        DomeSeeing {
            region: region.to_owned(),
            bucket: bucket.to_owned(),
            folder: folder.to_owned(),
            case: case.to_owned(),
            keys: vec![],
            n_keys: 0,
            opd: Vec::new(),
            current_opd: Rc::new(RefCell::new(Vec::new())),
            step: 0,
            n_step: duration * rate.unwrap_or(1),
            buffer: Cu::new(),
            duration,
            rate: rate.unwrap_or(1),
            time: vec![],
            sampling_time: 0f64,
            current_time: 0f64,
        }
    }
    /// Retrieves the OPD keys in AWS S3
    pub async fn get_keys(&mut self) -> Result<&mut Self, Box<dyn std::error::Error>> {
        let keys = cirrus::list(
            &self.region,
            &self.bucket,
            &format! {"{}/{}/OPDData_OPD_Data_",self.folder,self.case},
            None,
        )
        .await?;
        self.n_keys = keys.len();
        log::info!("Retrieved {} keys", self.n_keys);
        let mut sorter = keys
            .into_iter()
            .map(|x| {
                let time = x.split('/').last().unwrap()[17..29].parse::<f64>().unwrap();
                (x, time)
            })
            .collect::<Vec<(String, f64)>>();
        sorter.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        let (a, b): (Vec<_>, Vec<_>) = sorter.iter().cloned().unzip();
        self.keys = a;
        self.sampling_time = (b[1] - b[0]) / self.rate as f64;
        log::info!("Sampling time {:.3}s", self.sampling_time);
        self.time = b;
        Ok(self)
    }
    /// Loads the OPD from AWS S3
    pub async fn load_opd(&mut self) -> Result<&mut Self, Box<dyn std::error::Error>> {
        let first = self.n_keys as i32 - self.duration as i32 - 1;
        let keys = &self.keys[if first < 0 { 0 } else { first as usize }..];
        self.opd = cirrus::load::<Vec<f32>>(&self.region, &self.bucket, keys).await?;
        self.current_opd = Rc::new(RefCell::new(vec![0f32; self.opd[0].len()]));
        self.buffer = Cu::vector(self.opd[0].len());
        self.buffer.malloc();
        Ok(self)
    }
    /// Reset the dome seeing time series and optionally change the sampling rate
    pub fn reset(&mut self, rate: Option<usize>) {
        self.step = 0;
        if let Some(rate) = rate {
            self.n_step = rate * self.n_step / self.rate;
            self.rate = rate;
            self.sampling_time = (self.time[1] - self.time[0]) / self.rate as f64;
        }
    }
    /// Reset the dome seeing time
    pub fn at_time(&mut self, this_time: f64) {
        self.current_time = this_time;
        self.step = (self.current_time / self.sampling_time) as usize;
    }
}
impl Propagation for DomeSeeing {
    /// Ray traces a `Source` through `Gmt`, ray tracing stops at the exit pupil
    fn propagate(&mut self, src: &mut Source) -> &mut Self {
        src.add_same(&mut self.buffer.to_dev(&mut self.current_opd.borrow_mut()));
        self
    }
    fn time_propagate(&mut self, _secs: f64, src: &mut Source) -> &mut Self {
        self.propagate(src)
    }
}

impl Iterator for DomeSeeing {
    type Item = Rc<RefCell<Vec<f32>>>;
    /// Computes the next dome seeing OPD
    ///
    /// The dome seeing OPD is upsampled with a 1st order hold
    /// If the duration is longer than the available data then the same OPD is replayed in reverse order
    fn next(&mut self) -> Option<Self::Item> {
        self.current_time = self.step as f64 * self.sampling_time;
        if self.step < self.n_step {
            let nm1 = self.n_keys as i64 - 1;
            let k = self.step as i64 / self.rate as i64;
            let a = (nm1 - (k % (2 * nm1) - nm1).abs()) as usize;
            let b = (nm1 - ((k + 1) % (2 * nm1) - nm1).abs()) as usize;
            match (self.opd.get(a), self.opd.get(b)) {
                (Some(opd_prev), Some(opd_next)) => {
                    let alpha = (self.step % self.rate) as f64 / self.rate as f64;
                    self.current_opd
                        .borrow_mut()
                        .iter_mut()
                        .zip(opd_prev.iter().zip(opd_next.iter()))
                        .for_each(|(c, x)| *c = (1f64 - alpha) as f32 * x.0 + alpha as f32 * x.1);
                    self.step += 1;
                    Some(self.current_opd.clone())
                }
                _ => {
                    println!("Dome seeing failed @ (k: {} , idx: {:?})", k, (a, b));
                    None
                }
            }
        } else {
            None
        }
    }
}
