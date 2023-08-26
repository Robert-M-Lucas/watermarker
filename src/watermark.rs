use image::io::Reader;
use image::GenericImageView;
use std::path::Path;

pub struct Watermark {
    width: u32,
    height: u32,
    // TODO: Be more efficient
    data: Vec<bool>,
}

impl Watermark {
    pub fn load<P>(path: P) -> Watermark
    where
        P: AsRef<Path>,
    {
        let img = Reader::open(path).unwrap().decode().unwrap();
        let (width, height) = GenericImageView::dimensions(&img);
        let raw_rgb = img.as_rgb8().unwrap().as_raw();

        let data_size = width * height;
        let mut data = Vec::with_capacity(data_size as usize);

        for i in 0..(data_size as usize) {
            data.push(raw_rgb[i * 3] == 0 && raw_rgb[i * 3 + 1] == 0 && raw_rgb[i * 3 + 2] == 0);
        }

        Watermark {
            width,
            height,
            data,
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

pub struct WatermarkIterator<'a> {
    watermark: &'a Watermark,
    position: (u32, u32),
    scale: u32,
    scale_index: u32,
    current_pixel_index: u32,
}

impl<'a> WatermarkIterator<'a> {
    pub fn new(position: (u32, u32), scale: u32, watermark: &'a Watermark) -> WatermarkIterator {
        WatermarkIterator {
            position,
            watermark,
            scale,
            scale_index: 0,
            current_pixel_index: 0,
        }
    }
}

impl Iterator for WatermarkIterator<'_> {
    type Item = (u32, u32);

    fn next(&mut self) -> Option<Self::Item> {
        if self.scale_index == 0 && self.current_pixel_index as usize >= self.watermark.data().len()
        {
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
            ((self.current_pixel_index % self.watermark.width()) * self.scale)
                + (self.scale_index % self.scale),
            ((self.current_pixel_index / self.watermark.width()) * self.scale)
                + (self.scale_index / self.scale),
        );

        let pos = (
            self.position.0 + watermark_pos.0,
            self.position.1 + watermark_pos.1,
        );

        self.scale_index += 1;
        if self.scale_index >= self.scale * self.scale {
            self.scale_index = 0;
            self.current_pixel_index += 1;
        }
        Some(pos)
    }
}
