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
```
*/

use crseo::{Builder, FromBuilder, Gmt, Imaging, Source, imaging::Detector};
use image::{Rgb, RgbImage, imageops};

fn main() -> anyhow::Result<()> {
    let sid = vec![7, 1, 2, 3, 4, 5, 6];

    for k in 1..8 {
        println!("segment 1 to {k}");
        let mut gmt = Gmt::builder().build()?;
        gmt.keep(&sid[..k]);
        let mut src = Source::builder().band("R").build()?;
        let n_imgr = 128;
        let mut imgr = Imaging::builder()
            .detector(
                Detector::default()
                    .n_px_imagelet(n_imgr)
                    .n_px_framelet(n_imgr)
                    .osf(4),
            )
            .build()?;

        src.through(&mut gmt).xpupil().through(&mut imgr);

        let pupil = src.amplitude();
        let frame: Vec<f32> = imgr.frame().into();

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
        let bw_pupil_resized =
            imageops::resize(&bw_pupil, 100, 100, imageops::FilterType::Triangle);

        let max_image = frame
            .iter()
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap()
            .clone();
        let mut image = RgbImage::new(n_imgr as u32, n_imgr as u32);
        image
            .enumerate_pixels_mut()
            .zip(&frame)
            .for_each(|((_, _, px), f)| {
                let color = colorous::SPECTRAL.eval_continuous((*f / max_image) as f64);
                *px = Rgb([color.r, color.g, color.b])
            });
        imageops::flip_vertical_in_place(&mut image);
        let mut image_resized =
            imageops::resize(&image, 401, 401, imageops::FilterType::CatmullRom);

        imageops::overlay(&mut image_resized, &bw_pupil_resized, 0, 0);
        image_resized.save(format!("image_{k}.png"))?;
    }
    Ok(())
}
