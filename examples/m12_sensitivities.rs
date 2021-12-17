//use bincode;
use crseo::{ceo, Builder, Propagation, SOURCE};
use glass::BendingModes;
use serde_pickle as pkl;
use skyangle::Conversion;
use std::{fs::File, io::BufReader};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
pub enum Segment {
    CS,
    OA,
}
impl Segment {
    pub fn bending_modes(&self) -> Result<(Vec<f64>, Vec<f64>, BendingModes)> {
        let bm_file = match *self {
            Segment::CS => File::open(
                "/home/rconan/Documents/GMT/Notes/M1/Thermal/CFD/glass/data/bending_modes_CS.pkl",
            )?,
            Segment::OA => File::open(
                "/home/rconan/Documents/GMT/Notes/M1/Thermal/CFD/glass/data/bending_modes_OA.pkl",
            )?,
        };
        let rdr = BufReader::with_capacity(100_000, bm_file);
        let mut bending: BendingModes = pkl::from_reader(rdr)?;
        let (x, y): (Vec<_>, Vec<_>) = bending.nodes.chunks(2).map(|xy| (xy[0], xy[1])).unzip();
        if let Segment::OA = self {
            bending.nodes.chunks_mut(2).for_each(|xy| {
                let v = geotrans::oss_to_any_m1(1, [xy[0], xy[1], 0.]);
                xy[0] = v[0];
                xy[1] = v[1];
            });
        }
        Ok((x, y, bending))
    }
    pub fn heatmaps<'a>(
        &self,
        x: &[f64],
        y: &[f64],
        data: impl Iterator<Item = (&'a [f64], String)>,
    ) {
        data.for_each(|(p, rbm)| {
            complot::tri::heatmap(
                x,
                y,
                p,
                -5f64..5f64,
                Some(complot::Config::new().filename(match self {
                    Segment::CS => format!("S7-{}.png", rbm),
                    Segment::OA => format!("S1-{}.png", rbm),
                })),
            )
            .unwrap();
        });
    }
    pub fn dump(&self, phase: Vec<f64>) -> Result<()> {
        let mut file = match self {
            Segment::CS => File::create("RBMS_CS.pkl")?,
            Segment::OA => File::create("RBMS_OA.pkl")?,
        };
        pkl::to_writer(&mut file, &phase, true)?;
        Ok(())
    }
    pub fn calibrate_m1_rbms(&self, bending: &BendingModes) -> Result<Vec<f64>> {
        let stroke_fn = |dof| if dof < 3 { 1e-6 } else { 1f64.from_arcsec() };

        let mut phase = vec![];
        let n = bending.nodes.len() / 2;
        let sid = match self {
            Segment::CS => 6,
            Segment::OA => 0,
        };
        for dof in 0..6 {
            let mut m1_rbm = vec![vec![0.; 6]; 7];
            let stroke = stroke_fn(dof);

            let push_phase = {
                let (rays_x, rays_y): (Vec<_>, Vec<_>) =
                    bending.nodes.chunks(2).map(|xy| (xy[0], xy[1])).unzip();
                let mut gmt = ceo!(GMT);
                let mut src = SOURCE::new().rays_coordinates(rays_x, rays_y).build()?;
                m1_rbm[sid][dof] = stroke;

                gmt.update(Some(&m1_rbm), None, None, None);

                //src.through(&mut gmt); //.xpupil();
                gmt.propagate(&mut src);
                src.rays().opd()
            };
            println!("sum: {}", push_phase.iter().sum::<f64>());

            {
                let (rays_x, rays_y): (Vec<_>, Vec<_>) =
                    bending.nodes.chunks(2).map(|xy| (xy[0], xy[1])).unzip();
                let mut gmt = ceo!(GMT);
                let mut src = SOURCE::new().rays_coordinates(rays_x, rays_y).build()?;
                m1_rbm[sid][dof] = -stroke;

                gmt.update(Some(&m1_rbm), None, None, None);

                //            src.through(&mut gmt); //.xpupil();
                gmt.propagate(&mut src);
                println!("sum: {}", src.rays().opd().iter().sum::<f64>());
                phase.extend(
                    src.rays()
                        .opd()
                        .into_iter()
                        .zip(push_phase.into_iter())
                        .map(|(l, r)| 0.5f64 * (r as f64 - l as f64) / stroke),
                );
            }
        }
        Ok(phase)
    }
    pub fn calibrate_m2_rbms(&self, bending: &BendingModes) -> Result<Vec<f64>> {
        let stroke_fn = |dof| if dof < 3 { 1e-6 } else { 1f64.from_arcsec() };

        let mut phase = vec![];
        let n = bending.nodes.len() / 2;
        let sid = match self {
            Segment::CS => 6,
            Segment::OA => 0,
        };
        for dof in 0..6 {
            let mut m2_rbm = vec![vec![0.; 6]; 7];
            let stroke = stroke_fn(dof);

            let push_phase = {
                let (rays_x, rays_y): (Vec<_>, Vec<_>) =
                    bending.nodes.chunks(2).map(|xy| (xy[0], xy[1])).unzip();
                let mut gmt = ceo!(GMT);
                let mut src = SOURCE::new().rays_coordinates(rays_x, rays_y).build()?;
                m2_rbm[sid][dof] = stroke;

                gmt.update(None, Some(&m2_rbm), None, None);

                //src.through(&mut gmt); //.xpupil();
                gmt.propagate(&mut src);
                src.rays().opd()
            };
            //        println!("sum: {}", push_phase.iter().sum::<f64>());

            {
                let (rays_x, rays_y): (Vec<_>, Vec<_>) =
                    bending.nodes.chunks(2).map(|xy| (xy[0], xy[1])).unzip();
                let mut gmt = ceo!(GMT);
                let mut src = SOURCE::new().rays_coordinates(rays_x, rays_y).build()?;
                m2_rbm[sid][dof] = -stroke;

                gmt.update(None, Some(&m2_rbm), None, None);

                //            src.through(&mut gmt); //.xpupil();
                gmt.propagate(&mut src);
                //  println!("sum: {}", src.rays().opd().iter().sum::<f64>());
                phase.extend(
                    src.rays()
                        .opd()
                        .into_iter()
                        .zip(push_phase.into_iter())
                        .map(|(l, r)| 0.5f64 * (r as f64 - l as f64) / stroke),
                );
            }
        }
        Ok(phase)
    }
}

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let segment = Segment::OA;

    let (x, y, bending) = segment.bending_modes()?;
    let mut phase = segment.calibrate_m1_rbms(&bending)?;
    phase.append(&mut segment.calibrate_m2_rbms(&bending)?);

    let segment_rbms = vec!["Tx", "Ty", "Tz", "Rx", "Ry", "Rz"];
    let data = phase
        .chunks(bending.nodes.len() / 2)
        .zip((1..=2).into_iter().flat_map(|k| {
            segment_rbms
                .iter()
                .map(|r| format!("M{}_{}", k, r))
                .collect::<Vec<String>>()
        }));
    segment.heatmaps(&x, &y, data);

    segment.dump(phase)?;

    Ok(())
}
