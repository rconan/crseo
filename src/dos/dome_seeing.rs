use crate::{cu::Single, Cu, Propagation, Source};
use cirrus;

pub struct DomeSeeing {
    pub region: String,
    pub bucket: String,
    pub folder: String,
    pub case: String,
    pub keys: Vec<String>,
    pub n_keys: usize,
    pub opd: Vec<Vec<f32>>,
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
            opd: vec![],
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
    pub async fn get_keys(&mut self) -> Result<&mut Self, Box<dyn std::error::Error>> {
        let keys = cirrus::list(
            &self.region,
            &self.bucket,
            &format! {"{}/{}/OPDData_OPD_Data_",self.folder,self.case},
            None,
        )
        .await?;
        self.n_keys = keys.len();
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
        self.time = b;
        Ok(self)
    }
    pub async fn load_opd(&mut self) -> Result<&mut Self, Box<dyn std::error::Error>> {
        let first = self.n_keys - self.duration - 1;
        let keys = &self.keys[first..];
        self.opd = cirrus::load::<Vec<f32>>(&self.region, &self.bucket, keys).await?;
        self.buffer = Cu::vector(self.opd[0].len());
        self.buffer.malloc();
        Ok(self)
    }
    pub fn reset(&mut self, rate: Option<usize>) {
        self.step = 0;
        if let Some(rate) = rate {
            self.n_step = rate * self.n_step / self.rate;
            self.rate = rate;
            self.sampling_time = (self.time[1] - self.time[0]) / self.rate as f64;
        }
    }
}
impl Propagation for DomeSeeing {
    /// Ray traces a `Source` through `Gmt`, ray tracing stops at the exit pupil
    fn propagate(&mut self, src: &mut Source) -> &mut Self {
        let idx = (self.step - 1) / self.rate;
        let k = (self.step - 1) % self.rate;
        let alpha = k as f64 / self.rate as f64;
        let mut opd = self.opd[idx]
            .iter()
            .zip(self.opd[idx + 1].iter())
            .map(|x| (1f64 - alpha) as f32 * x.0 + alpha as f32 * x.1)
            .collect::<Vec<f32>>();
        src.add_same(&mut self.buffer.to_dev(&mut opd));
        self
    }
    fn time_propagate(&mut self, _secs: f64, src: &mut Source) -> &mut Self {
        self.propagate(src)
    }
}

impl Iterator for DomeSeeing {
    type Item = usize;
    fn next(&mut self) -> Option<Self::Item> {
        self.current_time = self.step as f64 * self.sampling_time;
        self.step += 1;
        if self.step <= self.n_step {
            Some(self.step)
        } else {
            None
        }
    }
}
