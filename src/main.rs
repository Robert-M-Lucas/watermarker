#![allow(dead_code)]
use std::cmp::max;
use std::time::Instant;
use image::io::Reader;
use image::{GenericImage, GenericImageView};
use imageproc::drawing::Canvas;
use noise::{Abs, NoiseFn, Perlin};
use rand::{Rng, thread_rng};
use watermark::Watermark;
use crate::config::Config;

mod watermark;
mod config;

fn main() {
    let config = Config::get_config_or_default("config.json");

    print!("Caching watermark...");
    let start = Instant::now();
    let watermark = Watermark::load("watermark.png");
    println!(" {:?}", start.elapsed());

    print!("Loading image...");
    let start = Instant::now();
    let mut img = Reader::open("input.jpg").unwrap().decode().unwrap();
    let (width, height) = GenericImageView::dimensions(&img);
    let mut rgb_raw;
    let rgba = img.as_mut_rgba8();
    let has_alpha = if rgba.is_some() {
        rgb_raw = rgba.unwrap().as_mut();
        true
    }
    else {
        rgb_raw = img.as_mut_rgb8().unwrap().as_mut();
        false
    };
    println!(" {:?}", start.elapsed());

    print!("Adjusting image...");
    let start = Instant::now();
    let mut rand = thread_rng();
    let noise = [
        Abs::new(Perlin::new(rand.gen())),
        Abs::new(Perlin::new(rand.gen())),
        Abs::new(Perlin::new(rand.gen())),
        Abs::new(Perlin::new(rand.gen())),
    ];


    let xcount = 1+(width / config.watermark_interval);
    let ycount = 1+(height / config.watermark_interval);

    let noise_scale_factor =
        (max(xcount, ycount) * 4) as f64 /
            (
                max(width, height)
            ) as f64;

    let mini_noise_scale_factor =
        max(watermark.width(), watermark.height()) as f64 /
            (
                max(width, height) * 5
            ) as f64;

    for wx in 0..xcount {
        for wy in 0..ycount {
            let iter = watermark.get_iter(
                (config.offset + wx * config.watermark_interval,
                 config.offset + wy * config.watermark_interval),
                config.scale
            );
            for position in iter {
                if position.0 >= width || position.1 >= height {
                    continue;
                }
                let raw_pixel_pos = if has_alpha {
                    (((position.1 * width) + position.0) * 4) as usize
                }
                else {
                    (((position.1 * width) + position.0) * 3) as usize
                };

                let mut colour = [
                    rgb_raw[raw_pixel_pos],
                    rgb_raw[raw_pixel_pos + 1],
                    rgb_raw[raw_pixel_pos + 2]
                ];

                let strength = noise[3].get(
                    [
                        position.0 as f64 * noise_scale_factor,
                        position.1 as f64 * noise_scale_factor
                    ]
                ).sqrt();
                let mut i = 0;
                for c in &mut colour {
                     let strength = strength * noise[i].get(
                        [
                            position.0 as f64 * mini_noise_scale_factor,
                            position.1 as f64 * mini_noise_scale_factor
                        ]
                    ).sqrt();

                    let dist = 255 - *c;
                    let min = 0;
                    let max = dist - (dist / 5);
                    let effect = if min != max {
                        rand.gen_range(min..max)
                    }
                    else {
                        max
                    };
                    let effect = (effect as f64 * strength) as u8;
                    *c += effect;
                    i += 1;
                }

                rgb_raw[raw_pixel_pos] = colour[0];
                rgb_raw[raw_pixel_pos + 1] = colour[1];
                rgb_raw[raw_pixel_pos + 2] = colour[2];
                if has_alpha {
                    rgb_raw[raw_pixel_pos + 3] = 255;
                }
            }
        }
    }
    println!(" {:?}", start.elapsed());

    print!("Saving image...");
    let start = Instant::now();
    img.save("output.jpg").unwrap();
    println!(" {:?}", start.elapsed());
}
