mod builder;
pub use builder::PyramidBuilder;
mod pyramid;
pub use pyramid::Pyramid;

#[derive(Default, Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
struct Modulation {
    amplitude: f32,
    sampling: i32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        wavefrontsensor::{data_processing::DataRef, Slopes},
        FromBuilder, Gmt, Source,
    };

    #[test]
    fn calibrate() {
        let n_lenslet = 90;
        let sid = 2;
        let mut pym = Pyramid::builder()
            .n_lenslet(n_lenslet)
            .modulation(8., 101)
            .build()
            .unwrap();
        let mut slopes_mat = pym.calibrate(3);
        dbg!(slopes_mat.shape());
        slopes_mat.pseudo_inverse().unwrap();

        let mut gmt = Gmt::builder().m2("Karhunen-Loeve", 100).build().unwrap();
        // gmt.keep(&[sid as i32]);
        // let mut pym = Pyramid::builder().n_lenslet(n_lenslet).build().unwrap();
        let mut src = Source::builder()
            .pupil_sampling(pym.pupil_sampling())
            .build()
            .unwrap();
        src.rotate_rays(0.5 * std::f64::consts::FRAC_PI_6);
        gmt.m2_modes_ij(sid - 1, 2, 1e-7);
        src.through(&mut gmt).xpupil().through(&mut pym);

        let _: complot::Heatmap = (
            (pym.frame().as_slice(), dbg!(pym.camera_resolution())),
            Some(complot::Config::new().filename("frame.png")),
        )
            .into();

        // let slopes = Slopes::from((&quad_cell, &pym));
        let coefs = &slopes_mat * &pym;
        dbg!(coefs);
    }

    #[test]
    fn calibrate_segment() {
        let n_lenslet = 90;
        let sid = 2;
        let mut pym = Pyramid::builder()
            .n_lenslet(n_lenslet)
            .modulation(8., 101)
            .build()
            .unwrap();
        let mut slopes_mat = pym.calibrate_segment(sid, 15, None);
        dbg!(slopes_mat.shape());
        slopes_mat.pseudo_inverse().unwrap();

        let mut gmt = Gmt::builder().m2("Karhunen-Loeve", 100).build().unwrap();
        // gmt.keep(&[sid as i32]);
        // let mut pym = Pyramid::builder().n_lenslet(n_lenslet).build().unwrap();
        let mut src = Source::builder()
            .pupil_sampling(pym.pupil_sampling())
            .build()
            .unwrap();
        src.rotate_rays(0.5 * std::f64::consts::FRAC_PI_6);
        gmt.m2_modes_ij(sid - 1, 2, 1e-7);
        src.through(&mut gmt).xpupil().through(&mut pym);

        let _: complot::Heatmap = (
            (pym.frame().as_slice(), dbg!(pym.camera_resolution())),
            Some(complot::Config::new().filename("frame.png")),
        )
            .into();

        // let slopes = Slopes::from((&quad_cell, &pym));
        let coefs = &slopes_mat * &pym;
        dbg!(coefs);
    }

    #[test]
    fn propagation() {
        let n_lenslet = 90;
        let mut gmt = Gmt::builder().build().unwrap();
        let mut pym = Pyramid::builder().n_lenslet(n_lenslet).build().unwrap();
        let mut src = Source::builder()
            .pupil_sampling(pym.pupil_sampling())
            .build()
            .unwrap();
        src.through(&mut gmt).xpupil().through(&mut pym);

        let _: complot::Heatmap = (
            (pym.frame().as_slice(), dbg!(pym.camera_resolution())),
            None,
        )
            .into();

        let (sx, sy) = pym.data();

        let _: complot::Heatmap = (
            (sx.as_slice(), (n_lenslet, n_lenslet)),
            Some(complot::Config::new().filename("pym_sx.png")),
        )
            .into();

        let _: complot::Heatmap = (
            (sy.as_slice(), (n_lenslet, n_lenslet)),
            Some(complot::Config::new().filename("pym_sy.png")),
        )
            .into();
    }

    #[test]
    fn quad_cell() {
        let sid = 2_usize;
        let n_lenslet = 90;

        let mut gmt = Gmt::builder().build().unwrap();
        gmt.keep(&[sid as i32]);
        let mut src = Source::builder().pupil_sampling(n_lenslet).build().unwrap();
        src.rotate_rays(0.5 * std::f64::consts::FRAC_PI_6);
        src.through(&mut gmt).xpupil();

        let pupil = nalgebra::DMatrix::<f32>::from_iterator(
            n_lenslet,
            n_lenslet,
            src.amplitude().into_iter().rev(),
        );
        let _: complot::Heatmap = (
            (pupil.as_slice(), (n_lenslet, n_lenslet)),
            Some(complot::Config::new().filename("pupil.png")),
        )
            .into();

        let mut quad_cell = DataRef::new(pupil);

        let mut gmt = Gmt::builder().m2("Karhunen-Loeve", 100).build().unwrap();
        gmt.keep(&[sid as i32]);
        let mut pym = Pyramid::builder().n_lenslet(n_lenslet).build().unwrap();
        let mut src = Source::builder()
            .pupil_sampling(pym.pupil_sampling())
            .build()
            .unwrap();
        src.rotate_rays(0.5 * std::f64::consts::FRAC_PI_6);

        src.through(&mut gmt).xpupil().through(&mut pym);
        quad_cell.set_ref_with(Slopes::from((&quad_cell, &pym)));
        pym.reset();

        let kl_mode = 5;
        let kl_a0 = 1e-6;
        gmt.m2_modes_ij(sid - 1, kl_mode, kl_a0);
        src.through(&mut gmt).xpupil();
        let opd = src.phase().clone();
        let opd_minmax =
            opd.iter()
                .fold((f32::INFINITY, f32::NEG_INFINITY), |(min, max), value| {
                    (
                        if *value < min { *value } else { min },
                        if *value > max { *value } else { max },
                    )
                });
        let o2p = (2. * std::f64::consts::PI / src.wavelength()) as f32;
        let phase_minmax = (o2p * opd_minmax.0, o2p * opd_minmax.1);
        println!("𝜑 minmax: {:?}", phase_minmax);
        let kl_coef = 1e-2 * kl_a0 as f32 / phase_minmax.0.abs().max(phase_minmax.1);
        println!("KL coef.:{:e}", kl_coef);

        gmt.m2_modes_ij(sid - 1, kl_mode, kl_coef as f64);
        src.through(&mut gmt).xpupil().through(&mut pym);

        serde_pickle::to_writer(
            &mut std::fs::File::create(format!("KL{kl_mode}.pkl")).unwrap(),
            &pym.frame(),
            Default::default(),
        )
        .unwrap();

        let _: complot::Heatmap = (
            (
                src.phase().as_slice(),
                (pym.pupil_sampling(), pym.pupil_sampling()),
            ),
            Some(complot::Config::new().filename("kl.png")),
        )
            .into();

        let _: complot::Heatmap = (
            (pym.frame().as_slice(), dbg!(pym.camera_resolution())),
            None,
        )
            .into();

        let slopes = Slopes::from((&quad_cell, &pym));

        let _: complot::Heatmap = (
            (
                quad_cell.sx(&slopes).unwrap().as_slice(),
                (n_lenslet, n_lenslet),
            ),
            Some(complot::Config::new().filename("pym_sx.png")),
        )
            .into();

        let _: complot::Heatmap = (
            (
                quad_cell.sy(&slopes).unwrap().as_slice(),
                (n_lenslet, n_lenslet),
            ),
            Some(complot::Config::new().filename("pym_sy.png")),
        )
            .into();
    }

    #[test]
    fn karhunen_loeve() {
        let sid = 7_usize;
        let n_lenslet = 90;

        let mut gmt = Gmt::builder().build().unwrap();
        gmt.keep(&[sid as i32]);
        let mut src = Source::builder().pupil_sampling(n_lenslet).build().unwrap();
        src.rotate_rays(0.5 * std::f64::consts::FRAC_PI_6);
        src.through(&mut gmt).xpupil();

        let pupil = nalgebra::DMatrix::<f32>::from_iterator(
            n_lenslet,
            n_lenslet,
            src.amplitude().into_iter().rev(),
        );
        let _: complot::Heatmap = (
            (pupil.as_slice(), (n_lenslet, n_lenslet)),
            Some(complot::Config::new().filename("pupil.png")),
        )
            .into();

        let mut gmt = Gmt::builder().m2("Karhunen-Loeve", 100).build().unwrap();
        gmt.keep(&[sid as i32]);
        let mut pym = Pyramid::builder().n_lenslet(n_lenslet).build().unwrap();
        let mut src = Source::builder()
            .pupil_sampling(pym.pupil_sampling())
            .build()
            .unwrap();
        src.rotate_rays(0.5 * std::f64::consts::FRAC_PI_6);

        src.through(&mut gmt).xpupil().through(&mut pym);
        let (mut sx0, mut sy0) = pym.data();
        let a0 = pym.add_quads();
        sx0.iter_mut()
            .zip(&mut sy0)
            .zip(&a0)
            .zip(&pupil)
            .for_each(|(((sx, sy), a), p)| {
                if *p > 0f32 {
                    *sx /= a;
                    *sy /= a;
                } else {
                    *sx = 0f32;
                    *sy = 0f32;
                }
            });
        pym.reset();

        let kl_mode = 2;

        gmt.m2_modes_ij(sid - 1, kl_mode, 1e-6);
        src.through(&mut gmt).xpupil();
        let opd = src.phase().clone();
        let opd_minmax =
            opd.iter()
                .fold((f32::INFINITY, f32::NEG_INFINITY), |(min, max), value| {
                    (
                        if *value < min { *value } else { min },
                        if *value > max { *value } else { max },
                    )
                });
        let o2p = (2. * std::f64::consts::PI / src.wavelength()) as f32;
        let phase_minmax = (o2p * opd_minmax.0, o2p * opd_minmax.1);
        println!("𝜑 minmax: {:?}", phase_minmax);
        let kl_coef = 0.1e-6 / phase_minmax.0.abs().max(phase_minmax.1);
        println!("KL coef.:{:e}", kl_coef);

        gmt.m2_modes_ij(sid - 1, kl_mode, kl_coef as f64 / 2.);
        src.through(&mut gmt).xpupil().through(&mut pym);

        serde_pickle::to_writer(
            &mut std::fs::File::create(format!("KL{kl_mode}.pkl")).unwrap(),
            &pym.frame(),
            Default::default(),
        )
        .unwrap();

        let _: complot::Heatmap = (
            (
                src.phase().as_slice(),
                (pym.pupil_sampling(), pym.pupil_sampling()),
            ),
            Some(complot::Config::new().filename("kl.png")),
        )
            .into();

        let _: complot::Heatmap = (
            (pym.frame().as_slice(), dbg!(pym.camera_resolution())),
            None,
        )
            .into();

        let (mut sx, mut sy) = pym.data();

        let a = pym.add_quads();
        sx.iter_mut()
            .zip(&mut sy)
            .zip(&a)
            .zip(&pupil)
            .for_each(|(((sx, sy), a), p)| {
                if *p > 0f32 {
                    *sx /= a;
                    *sy /= a;
                } else {
                    *sx = 0f32;
                    *sy = 0f32;
                }
            });
        sx -= &sx0;
        sy -= &sy0;
        dbg!(sx.sum());
        dbg!(sy.sum());
        let _: complot::Heatmap = (
            (sx.as_slice(), (n_lenslet, n_lenslet)),
            Some(complot::Config::new().filename("pym_sx.png")),
        )
            .into();

        let _: complot::Heatmap = (
            (sy.as_slice(), (n_lenslet, n_lenslet)),
            Some(complot::Config::new().filename("pym_sy.png")),
        )
            .into();
    }

    #[test]
    fn add_quads() {
        let n_lenslet = 90;
        let mut gmt = Gmt::builder().build().unwrap();
        let mut pym = Pyramid::builder().n_lenslet(n_lenslet).build().unwrap();
        let mut src = Source::builder()
            .pupil_sampling(pym.pupil_sampling())
            .build()
            .unwrap();
        src.rotate_rays(0.5 * std::f64::consts::FRAC_PI_6);

        src.through(&mut gmt).xpupil().through(&mut pym);

        let a = pym.add_quads();

        let _: complot::Heatmap = (
            (a.as_slice(), (n_lenslet, n_lenslet)),
            Some(complot::Config::new().filename("pym_a15.png")),
        )
            .into();
    }

    #[test]
    fn add_quads_with_kl() {
        let n_lenslet = 90;

        let mut gmt = Gmt::builder().build().unwrap();
        gmt.keep(&[1]);
        let mut src = Source::builder().pupil_sampling(n_lenslet).build().unwrap();
        src.rotate_rays(0.5 * std::f64::consts::FRAC_PI_6);
        src.through(&mut gmt).xpupil();

        let pupil = nalgebra::DMatrix::<f32>::from_iterator(
            n_lenslet,
            n_lenslet,
            src.amplitude().into_iter().rev(),
        );
        let _: complot::Heatmap = (
            (pupil.as_slice(), (n_lenslet, n_lenslet)),
            Some(complot::Config::new().filename("pupil.png")),
        )
            .into();

        let mut gmt = Gmt::builder().m2("Karhunen-Loeve", 100).build().unwrap();
        gmt.keep(&[1]);
        let mut pym = Pyramid::builder().n_lenslet(n_lenslet).build().unwrap();
        let mut src = Source::builder()
            .pupil_sampling(pym.pupil_sampling())
            .build()
            .unwrap();
        src.rotate_rays(0.5 * std::f64::consts::FRAC_PI_6);

        src.through(&mut gmt).xpupil().through(&mut pym);

        let mut a0 = pym.add_quads();
        a0.iter_mut().zip(&pupil).for_each(|(a, p)| *a *= p);
        let _: complot::Heatmap = (
            (a0.as_slice(), (n_lenslet, n_lenslet)),
            Some(complot::Config::new().filename("pym_a0.png")),
        )
            .into();
        pym.reset();

        let kl_mode = 63;
        let sid = 1;

        let kl_a0 = -1e-6;
        gmt.m2_modes_ij(sid - 1, kl_mode, kl_a0);
        src.through(&mut gmt).xpupil();
        let opd = src.phase().clone();
        let opd_minmax =
            opd.iter()
                .fold((f32::INFINITY, f32::NEG_INFINITY), |(min, max), value| {
                    (
                        if *value < min { *value } else { min },
                        if *value > max { *value } else { max },
                    )
                });
        let o2p = (2. * std::f64::consts::PI / src.wavelength()) as f32;
        let phase_minmax = (o2p * opd_minmax.0, o2p * opd_minmax.1);
        println!("𝜑 minmax: {:?}", phase_minmax);
        let kl_coef = 1e-2 * kl_a0 as f32 / phase_minmax.0.abs().max(phase_minmax.1);
        println!("KL coef.:{:e}", kl_coef);

        gmt.m2_modes_ij(sid - 1, kl_mode, kl_coef as f64);
        src.through(&mut gmt).xpupil().through(&mut pym);

        let _: complot::Heatmap = (
            (
                src.phase().as_slice(),
                (pym.pupil_sampling(), pym.pupil_sampling()),
            ),
            Some(complot::Config::new().filename("kl.png")),
        )
            .into();

        let mut a = pym.add_quads() - &a0;
        a.iter_mut().zip(&pupil).for_each(|(a, p)| *a *= p);
        let _: complot::Heatmap = (
            (a.as_slice(), (n_lenslet, n_lenslet)),
            Some(complot::Config::new().filename("pym_a.png")),
        )
            .into();
    }
}
