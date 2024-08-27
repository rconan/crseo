use crate::calib::Calib;
use crate::{CalibrationMode, PushPull, M2_N_MODE};
use crseo::{
    gmt::{GmtBuilder, GmtMirror, GmtMirrorBuilder, GmtMx, MirrorGetSet},
    Source,
};
use crseo::{Builder, FromBuilder, Gmt};
use std::time::Instant;

impl<M, const SID: u8> Calib<M, SID>
where
    Gmt: GmtMirror<M>,
    GmtBuilder: GmtMirrorBuilder<M>,
    M: Default + GmtMx,
{
    pub fn calibrate_segment_modes(&mut self, stroke: f64) -> crate::Result<&mut Self> {
        self.calibrate(
            crate::CalibrationMode::Modes {
                n_mode: M2_N_MODE,
                stroke,
            },
            |gmt, sid, cmd| {
                <Gmt as GmtMirror<M>>::as_mut(gmt).set_segment_modes(sid, cmd);
            },
        )
        /*        println!("Calibrating segment modes ...");
        let mut gmt = Gmt::builder().n_mode::<M>(self.n_mode).build()?;
        gmt.keep(&[SID as i32]);
        let mut src = self.src_builder.clone().build()?;

        src.through(&mut gmt).xpupil();
        // let phase0 = src.phase().clone();
        let amplitude0 = src.amplitude();
        let mut mask: Vec<_> = amplitude0.iter().map(|x| *x > 0.).collect();

        let mut a = vec![0f64; self.n_mode];
        let mut calib = vec![];
        let now = Instant::now();
        for i in 0..self.n_mode {
            a[i] = stroke;
            <Gmt as GmtMirror<M>>::as_mut(&mut gmt).set_segment_modes(SID, &a);
            src.through(&mut gmt).xpupil();
            mask.iter_mut()
                .zip(src.amplitude().into_iter())
                .for_each(|(m, a)| *m &= a > 0.);
            let push = src.phase().clone();

            a[i] *= -1.;
            <Gmt as GmtMirror<M>>::as_mut(&mut gmt).set_segment_modes(SID, &a);
            src.through(&mut gmt).xpupil();
            mask.iter_mut()
                .zip(src.amplitude().into_iter())
                .for_each(|(m, a)| *m &= a > 0.);

            let pushpull: Vec<_> = push
                .iter()
                .zip(src.phase().iter())
                .zip(&mask)
                .map(|((x, y), &m)| if m { 0.5 * (x - y) as f64 / stroke } else { 0. })
                .collect();
            calib.push(pushpull);

            a[i] = 0.;
        }
        calib.iter_mut().for_each(|x| {
            let mut iter = mask.iter();
            x.retain(|_| *iter.next().unwrap())
        });
        println!("Elapsed: {:?}", now.elapsed());
        self.mask = mask;
        self.c = calib.into_iter().flatten().collect();
        Ok(self)*/
    }

    pub fn calibrate_rigid_body_motions(
        &mut self,
        stroke: [Option<f64>; 6],
    ) -> crate::Result<&mut Self> {
        self.calibrate(crate::CalibrationMode::RBM(stroke), |gmt, sid, cmd| {
            <Gmt as GmtMirror<M>>::as_mut(gmt).set_rigid_body_motions(sid, cmd);
        })
        /*        println!("Calibrating rigid body motions ...");
        let mut gmt = Gmt::builder().build()?;
        gmt.keep(&[SID as i32]);
        let mut src = self.src_builder.clone().build()?;

        src.through(&mut gmt).xpupil();
        // let phase0 = src.phase().clone();
        let amplitude0 = src.amplitude();
        let mut mask: Vec<_> = amplitude0.iter().map(|x| *x > 0.).collect();

        let mut tr_xyz = [0f64; 6];
        let mut calib = vec![];
        let now = Instant::now();
        for i in 0..6 {
            let Some(s) = stroke[i] else {
                continue;
            };
            tr_xyz[i] = s;
            <Gmt as GmtMirror<M>>::as_mut(&mut gmt).set_rigid_body_motions(SID, &tr_xyz);
            src.through(&mut gmt).xpupil();
            mask.iter_mut()
                .zip(src.amplitude().into_iter())
                .for_each(|(m, a)| *m &= a > 0.);
            let push = src.phase().clone();

            tr_xyz[i] *= -1.;
            <Gmt as GmtMirror<M>>::as_mut(&mut gmt).set_rigid_body_motions(SID, &tr_xyz);
            src.through(&mut gmt).xpupil();
            mask.iter_mut()
                .zip(src.amplitude().into_iter())
                .for_each(|(m, a)| *m &= a > 0.);

            let pushpull: Vec<_> = push
                .iter()
                .zip(src.phase().iter())
                .zip(&mask)
                .map(|((x, y), &m)| if m { 0.5 * (x - y) as f64 / s } else { 0. })
                .collect();
            calib.push(pushpull);

            tr_xyz[i] = 0.;
        }
        calib.iter_mut().for_each(|x| {
            let mut iter = mask.iter();
            x.retain(|_| *iter.next().unwrap())
        });
        println!("Elapsed: {:?}", now.elapsed());
        self.mask = mask;
        self.c = calib.into_iter().flatten().collect();
        Ok(self)*/
    }

    pub fn calibrate<F>(
        &mut self,
        calib_mode: CalibrationMode,
        cmd_fn: F,
    ) -> crate::Result<&mut Self>
    where
        F: Fn(&mut Gmt, u8, &[f64]),
    {
        match calib_mode {
            CalibrationMode::RBM(stroke) => {
                println!("Calibrating rigid body motions ...");
                let mut gmt = Gmt::builder().build()?;
                gmt.keep(&[SID as i32]);
                let mut src = self.src_builder.clone().build()?;

                src.through(&mut gmt).xpupil();
                // let phase0 = src.phase().clone();
                let amplitude0 = src.amplitude();
                self.mask = amplitude0.iter().map(|x| *x > 0.).collect();

                let mut tr_xyz = [0f64; 6];
                let mut calib = vec![];
                let now = Instant::now();
                for i in 0..6 {
                    let Some(s) = stroke[i] else {
                        continue;
                    };
                    calib.push(self.push_pull(i, s, &mut gmt, &mut src, &mut tr_xyz, &cmd_fn));
                }
                calib.iter_mut().for_each(|x| {
                    let mut iter = self.mask.iter();
                    x.retain(|_| *iter.next().unwrap())
                });
                println!("Elapsed: {:?}", now.elapsed());
                // self.mask = mask;
                self.c = calib.into_iter().flatten().collect();
            }
            CalibrationMode::Modes { n_mode, stroke } => {
                println!("Calibrating segment modes ...");
                let mut gmt = Gmt::builder().n_mode::<M>(n_mode).build()?;
                gmt.keep(&[SID as i32]);
                let mut src = self.src_builder.clone().build()?;

                src.through(&mut gmt).xpupil();
                // let phase0 = src.phase().clone();
                let amplitude0 = src.amplitude();
                self.mask = amplitude0.iter().map(|x| *x > 0.).collect();

                let mut a = vec![0f64; n_mode];
                let mut calib = vec![];
                let now = Instant::now();
                for i in 0..self.n_mode {
                    calib.push(self.push_pull(i, stroke, &mut gmt, &mut src, &mut a, &cmd_fn));
                }
                calib.iter_mut().for_each(|x| {
                    let mut iter = self.mask.iter();
                    x.retain(|_| *iter.next().unwrap())
                });
                println!("Elapsed: {:?}", now.elapsed());
                self.c = calib.into_iter().flatten().collect();
            }
        }
        Ok(self)
    }

    pub fn match_areas<T: GmtMx>(&mut self, other: &mut Calib<T, SID>) {
        assert_eq!(self.mask.len(), other.mask.len());
        let area_a = self.area();
        let area_b = other.area();
        let mask: Vec<_> = self
            .mask
            .iter()
            .zip(other.mask.iter())
            .map(|(&a, &b)| a && b)
            .collect();

        let c_to_area: Vec<_> = self
            .c
            .chunks(area_a)
            .flat_map(|c| {
                self.mask
                    .iter()
                    .zip(&mask)
                    .filter(|&(&ma, _)| ma)
                    .zip(c)
                    .filter(|&((_, &mb), _)| mb)
                    .map(|(_, c)| *c)
            })
            .collect();
        self.c = c_to_area;
        let c_to_area: Vec<_> = other
            .c
            .chunks(area_b)
            .flat_map(|c| {
                other
                    .mask
                    .iter()
                    .zip(&mask)
                    .filter(|&(&ma, _)| ma)
                    .zip(c)
                    .filter(|&((_, &mb), _)| mb)
                    .map(|(_, c)| *c)
            })
            .collect();
        other.c = c_to_area;

        self.mask = mask.clone();
        other.mask = mask;
    }
}

impl<M: Default + GmtMx, const SID: u8> PushPull for Calib<M, SID> {
    fn push_pull<F>(
        &mut self,
        i: usize,
        s: f64,
        gmt: &mut Gmt,
        src: &mut Source,
        cmd: &mut [f64],
        cmd_fn: &F,
    ) -> Vec<f64>
    where
        F: Fn(&mut Gmt, u8, &[f64]),
    {
        cmd[i] = s;
        cmd_fn(gmt, SID, cmd);
        src.through(gmt).xpupil();
        self.mask
            .iter_mut()
            .zip(src.amplitude().into_iter())
            .for_each(|(m, a)| *m &= a > 0.);
        let push = src.phase().clone();

        cmd[i] *= -1.;
        cmd_fn(gmt, SID, cmd);
        src.through(gmt).xpupil();
        self.mask
            .iter_mut()
            .zip(src.amplitude().into_iter())
            .for_each(|(m, a)| *m &= a > 0.);

        cmd[i] = 0.0;

        push.iter()
            .zip(src.phase().iter())
            .zip(&self.mask)
            .map(|((x, y), &m)| if m { 0.5 * (x - y) as f64 / s } else { 0. })
            .collect()
    }
}
