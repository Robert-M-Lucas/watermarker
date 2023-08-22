use std::cmp::max;
use std::path::Path;
use image::io::Reader;
use image::{GenericImage, GenericImageView, Rgb, Rgba};
use imageproc::definitions::Image;
use imageproc::drawing::Canvas;
use noise::{Abs, NoiseFn, Perlin};
use rand::{Rng, thread_rng};


struct Watermark {
    width: u32,
    height: u32,
    // TODO: Be more efficient
    data: Vec<bool>
}

impl Watermark {
    pub fn load<P>(path: P) -> Watermark
        where
            P: AsRef<Path>,
    {
        let img = Reader::open(path).unwrap().decode().unwrap();
        let (width, height) = GenericImageView::dimensions(&img);
        let raw_rgb = img.as_rgb8().unwrap().as_raw();

        let data_size = (width * height);
        let mut data = Vec::with_capacity(data_size as usize);

        for i in 0..(data_size as usize) {
            data.push(
                raw_rgb[i*3] == 0 && raw_rgb[i*3+1] == 0 && raw_rgb[i*3+2] == 0
            );
        }

        Watermark {
            width,
            height,
            data
        }
    }

    pub fn get_iter(&self, position: (u32, u32), scale: u32) -> WatermarkIterator {
        WatermarkIterator::new(position, scale, self)
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn data(&self) -> &[bool] {
        &self.data
    }
}

struct WatermarkIterator<'a> {
    watermark: &'a Watermark,
    position: (u32, u32),
    scale: u32,
    scale_index: u32,
    current_pixel_index: u32
}

impl<'a> WatermarkIterator<'a> {
    pub fn new(position: (u32, u32), scale: u32, watermark: &'a Watermark) -> WatermarkIterator {
        WatermarkIterator {
            position,
            watermark,
            scale,
            scale_index: 0,
            current_pixel_index: 0
        }
    }
}

impl Iterator for WatermarkIterator<'_> {
    type Item = (u32, u32);

    fn next(&mut self) -> Option<Self::Item> {
        if self.scale_index == 0 && self.current_pixel_index as usize >= self.watermark.data().len() {
            return None;
        }

        if self.scale_index == 0 {
            while !self.watermark.data()[self.current_pixel_index as usize] {
                self.current_pixel_index += 1;
                if self.current_pixel_index as usize >= self.watermark.data().len() {
                    return None;
                }
            }
        }

        let watermark_pos = (
            ((self.current_pixel_index % self.watermark.width()) * self.scale) + (self.scale_index % self.scale),
            ((self.current_pixel_index / self.watermark.width()) * self.scale) + (self.scale_index / self.scale)
        );

        let pos = (self.position.0 + watermark_pos.0, self.position.1 + watermark_pos.1);

        self.scale_index += 1;
        if self.scale_index >= self.scale * self.scale {
            self.scale_index = 0;
            self.current_pixel_index += 1;
        }
        Some(pos)
    }
}

fn main() {
    const OFFSET: u32 = 0;
    const WATERMARK_INTERVAL: u32 = 600;
    const SCALE: u32 = 3;

    println!("Caching watermark");
    let watermark = Watermark::load("watermark.png");

    println!("Loading image");
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

    println!("Adjusting image");
    let mut rand = thread_rng();
    let noise = [
        Abs::new(Perlin::new(rand.gen())),
        Abs::new(Perlin::new(rand.gen())),
        Abs::new(Perlin::new(rand.gen())),
        Abs::new(Perlin::new(rand.gen())),
    ];


    let xcount = 1+(width / WATERMARK_INTERVAL);
    let ycount = 1+(height / WATERMARK_INTERVAL);

    let noise_scale_factor =
        max(xcount, ycount) as f64 /
            (
                max(width, height)
            ) as f64;

    let mini_noise_scale_factor =
        max(watermark.width(), watermark.height) as f64 /
            (
                max(width, height) * 5
            ) as f64;

    for wx in 0..xcount {
        for wy in 0..ycount {
            let iter = watermark.get_iter(
                (OFFSET + wx * WATERMARK_INTERVAL,
                 OFFSET + wy * WATERMARK_INTERVAL),
                SCALE
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
                ).powf(0.75);
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
                    let max = (dist / 4) * 3;
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

    println!("Saving image");
    img.save("output.jpg").unwrap();
}
