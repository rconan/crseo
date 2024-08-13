use crseo::{ceo, FromBuilder};
use crseo::{pssn::TelescopeError, Builder, Fwhm, PSSn, Source};
use serde::Deserialize;
use serde_pickle as pickle;
use skyangle::{Conversion, SkyAngle};
use std::collections::BTreeMap;
use std::fs::File;

#[allow(dead_code)]
enum Field {
    Radial,
    Delaunay,
}

#[derive(Deserialize)]
struct DField {
    pub zenith_arcmin: Vec<f32>,
    pub azimuth_degree: Vec<f32>,
    pub se: Vec<Vec<usize>>,
}

fn telescope_aberration_free(filename: &str, e_fwhm: &[f64]) {
    println!("{}", filename);
    let file = File::open("/home/rconan/projects/gicsdom/ceo/fielddelaunay21.pkl").unwrap();
    let DField {
        zenith_arcmin,
        azimuth_degree,
        se,
    } = pickle::from_reader(&file).unwrap();
    //println!("Vertices: {:#?}", se);
    let (x, y): (Vec<f64>, Vec<f64>) = zenith_arcmin
        .iter()
        .zip(azimuth_degree.iter())
        .map(|(z, a)| {
            (
                (z.from_arcmin() * a.to_radians().cos()) as f64,
                (z.from_arcmin() * a.to_radians().sin()) as f64,
            )
        })
        .unzip();
    let areas: Vec<_> = se
        .iter()
        .map(|v| {
            let (a, b, c) = (v[0] - 1, v[1] - 1, v[2] - 1);
            0.5 * ((x[a] - x[c]) * (y[b] - y[a]) - (x[a] - x[b]) * (y[c] - y[a])).abs()
        })
        .collect();
    let field_area = areas.iter().sum::<f64>();
    /*
        println!(
            "areas: {:?} ==> {}/{}",
            areas,
            field_area,
            std::f64::consts::PI * (5f64.from_arcmin()).powf(2f64)
        );
    */
    let file = File::open(format!("{}.pkl", filename)).unwrap();
    let fwhm: BTreeMap<String, Vec<f64>> = pickle::from_reader(file).unwrap();
    println!("Case #:{}", fwhm.len());

    let mut field_free_fwhm = BTreeMap::<String, Vec<f64>>::new();
    for (key, value) in fwhm.iter() {
        let n = value.len() as f64;
        let fwhm_mean = value.iter().sum::<f64>() / n;
        let fwhm_del_mean = se
            .iter()
            .zip(areas.iter())
            .map(|(v, ai)| ai * v.iter().map(|i| value[i - 1]).sum::<f64>() / 3f64)
            .sum::<f64>()
            / field_area;
        let field_free: Vec<_> = value
            .iter()
            .zip(e_fwhm)
            .map(|(v, e)| (v * v * 1e6 - e * e).sqrt())
            .collect();
        let field_free_mean = field_free.iter().sum::<f64>() / n;
        let field_free_del_mean = se
            .iter()
            .zip(areas.iter())
            .map(|(v, ai)| ai * v.iter().map(|i| field_free[i - 1]).sum::<f64>() / 3f64)
            .sum::<f64>()
            / field_area;
        println!(
            " {:24}: [{:3.0}/{:3.0}mas]/[{:3.0}/{:3.0}mas]",
            key,
            fwhm_mean * 1e3,
            field_free_mean,
            fwhm_del_mean * 1e3,
            field_free_del_mean
        );
        field_free_fwhm.insert(key.to_string(), field_free);
    }

    let mut file = File::create(format!("{}-tel.pkl", filename)).unwrap();
    // pickle::to_writer(&mut file, &field_free_fwhm, true).unwrap();
}
fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut gmt = ceo!(GmtBuilder);
    match Field::Delaunay {
        Field::Radial => {
            for z_arcmin in 0..11 {
                let mut src = Source::builder()
                    .zenith_azimuth(
                        vec![SkyAngle::Arcminute(z_arcmin as f32).to_radians()],
                        vec![0.],
                    )
                    .build()?;
                let wfe_rms = src.through(&mut gmt).xpupil().wfe_rms_10e(-9);
                let mut pssn: PSSn<TelescopeError> = PSSn::new();
                pssn.build(&mut src);
                src.through(&mut pssn);
                let e_pssn = pssn.peek().estimates[0];
                let otf = pssn.telescope_otf();
                let mut fwhm = Fwhm::new();
                fwhm.build(&mut src);
                let e_fwhm = fwhm.from_complex_otf(&otf)[0].to_mas();
                println!(
                    "z: {:02}arcmin ; WFE RMS: {:6.0}nm ; PSSn: {:.4} ; FWHM: {:4.0}mas",
                    z_arcmin, wfe_rms[0], e_pssn, e_fwhm
                )
            }
        }
        Field::Delaunay => {
            let mut src = Source::builder().field_delaunay21().build()?;
            let wfe_rms = src.through(&mut gmt).xpupil().wfe_rms_10e(-9);
            let mut pssn: PSSn<TelescopeError> = PSSn::new();
            pssn.build(&mut src);
            src.through(&mut pssn);
            let e_pssn = pssn.peek().estimates.clone();
            let mut fwhm = Fwhm::new();
            fwhm.build(&mut src);
            let otf = pssn.telescope_otf();
            let e_fwhm: Vec<_> = fwhm
                .from_complex_otf(&otf)
                .iter()
                .map(|x| x.to_mas())
                .collect();
            println!(
                "{:^10}  {:^10}  {:^10}  {:^10}",
                "zenith[']", "WFE[nm]", "PSSn", "FWHM[mas]"
            );
            for k in 0..21 {
                println!(
                    "{:>10.3}  {:>10.0}  {:>10.4}  {:>10.0}",
                    src.zenith[k].to_arcmin(),
                    wfe_rms[k],
                    e_pssn[k],
                    e_fwhm[k]
                );
            }
            telescope_aberration_free(
                "/home/rconan/Dropbox/Documents/GMT/Notes/GLAO/Analysis/glao_open_loop.fwhm",
                &e_fwhm,
            );
            telescope_aberration_free(
                "/home/rconan/Dropbox/Documents/GMT/Notes/GLAO/Analysis/glao_closed_loop.fwhm",
                &e_fwhm,
            );
        }
    }
    Ok(())
}
