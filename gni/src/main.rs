/*!
# GMT PSF from 1 to 7 segments

Compute each image with
```shell
cargo r -r
```
and then build the gif with
```shell
convert -delay 200 -loop 0 image_*.png gmt-partial.gif
convert gmt-partial.gif -coalesce -duplicate 1,-2-1 -set loop 0 gmt-partial.gif

90214680000.0
```
*/

use crseo::{Builder, FromBuilder, Gmt, Imaging, Source, imaging::Detector};
use image::{Rgb, RgbImage, imageops};
// use imageproc::drawing::draw_cross_mut;

// # of telescopes
const N: usize = 6;

fn main() -> anyhow::Result<()> {
    // optical model
    let mut gmt = Gmt::builder().build()?;
    let mut src = Source::builder().band("R").build()?;
    let n_imgr = 101;
    let mut imgr = Imaging::builder()
        .detector(
            Detector::default()
                .n_px_imagelet(n_imgr)
                .n_px_framelet(n_imgr)
                .osf(4),
        )
        .build()?;

    // selection of segment pairs and piston values
    let wl = src.wavelength();
    let (sid, piston) = match N {
        6 => (
            vec![1, 2, 3, 4, 5, 6],
            // [0f64, wl / 4., -wl / 4., wl / 2., -wl / 4., wl / 4., 0f64],
            // [0f64, 0., wl / 2., wl / 2., wl / 2., 0., 0f64],
            // [0f64, wl / 2., 0., wl / 2., wl / 2., 0., 0f64],
            [0f64, wl / 2., 0., wl / 2., 0., wl / 2., 0f64],
        ),
        4 => (
            vec![2, 3, 5, 6],
            // [0f64, wl / 4., wl / 2., 0., -wl / 4., 0., 0f64],
            [0f64, 0., wl / 2., 0., 0., wl / 2., 0f64],
        ),
        2 => (vec![1, 4], [0., 0f64, 0f64, wl / 2., 0f64, 0f64, 0f64]),
        n => panic!("found {n} telescopes, expected 2, 4 or 6"),
    };
    gmt.keep(&sid);

    // Phase GMT reference frame
    src.through(&mut gmt).xpupil().through(&mut imgr);
    let frame0: Vec<f32> = imgr.frame().into();
    imgr.reset();

    // GMT with nulling piston
    src.add_piston(&piston);
    src.through(&mut imgr);

    // estimating piston from phase [wl]
    let piston: Vec<f64> = src.segment_piston().into_iter().map(|x| x / wl).collect();
    println!("Piston: {:+.3?}", &piston);

    // GMT pupil
    let pupil = src.amplitude();
    let n = src.pupil_sampling as u32;
    let mut bw_pupil = RgbImage::new(n, n);
    bw_pupil
        .enumerate_pixels_mut()
        .zip(&pupil)
        .for_each(|((_, _, px), p)| {
            if *p > 0f32 {
                *px = Rgb([255; 3]);
            }
        });
    imageops::flip_vertical_in_place(&mut bw_pupil);
    let bw_pupil_resized = imageops::resize(&bw_pupil, 100, 100, imageops::FilterType::Triangle);

    // phase in wavelength units
    let phase: Vec<_> = src.phase().iter().map(|x| *x / wl as f32).collect();
    let a = phase
        .iter()
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap()
        .clone();
    let b = phase
        .iter()
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap()
        .clone();
    let mut phase_inlay = RgbImage::new(n, n);
    phase_inlay
        .enumerate_pixels_mut()
        .zip(&phase)
        .for_each(|((_, _, px), p)| {
            // let color = colorous::SPECTRAL.eval_continuous(((*p - b) / (a - b)) as f64);
            let i = (4. * (*p - b) / (a - b)) as usize;
            *px = Rgb(if i == 0 {
                [0; 3]
            } else {
                colorous::SET3[i].into_array()
            })
        });
    imageops::flip_vertical_in_place(&mut phase_inlay);
    let phase_inlay_resized =
        imageops::resize(&phase_inlay, 100, 100, imageops::FilterType::Triangle);

    // detector image plane frame
    let frame: Vec<f32> = imgr.frame().into();
    dbg!(frame[50 + 101 * 50] / frame0[50 + 101 * 50]);
    // let max_image = frame
    //     .iter()
    //     .max_by(|a, b| a.partial_cmp(b).unwrap())
    //     .unwrap()
    //     .clone();
    let mut image = RgbImage::new(n_imgr as u32, n_imgr as u32);
    image
        .enumerate_pixels_mut()
        .zip(&frame)
        .for_each(|((_, _, px), f)| {
            let color = colorous::SPECTRAL.eval_continuous((*f / frame0[50 + 101 * 50]) as f64);
            *px = Rgb([color.r, color.g, color.b])
        });
    imageops::flip_vertical_in_place(&mut image);
    let mut image_resized = imageops::resize(&image, 401, 401, imageops::FilterType::CatmullRom);

    imageops::overlay(&mut image_resized, &bw_pupil_resized, 0, 0);
    imageops::overlay(&mut image_resized, &phase_inlay_resized, 301, 0);
    // draw_cross_mut(&mut image_resized, Rgb([0, 0, 0]), 200, 200);
    image_resized.save(format!("image.png"))?;

    Ok(())
}
