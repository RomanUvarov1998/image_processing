use fltk::{
    prelude::*,
    image
};

#[derive(Clone)]
pub struct Img {
    image: image::BmpImage,
    pixels: Vec<u8>
}

const CHANNELS_COUNT: usize = 4;

impl Img {
    pub fn new(image: image::BmpImage) -> Self {
        Img {
            pixels: image.to_rgb_data(),
            image,
        }
    }

    pub fn w(&self) -> usize { self.image.w() as usize }
    pub fn h(&self) -> usize { self.image.h() as usize }
    pub fn c(&self) -> usize { self.image.count() as usize }

    pub fn pixel_at(&self, col: usize, row: usize, plane: usize) -> u8 {
        assert!(col < self.w());
        assert!(row < self.h());
        assert!(plane < self.c());

        let col_offset: usize = col * CHANNELS_COUNT;
        let row_offset: usize = row * self.w() * CHANNELS_COUNT;
        let plane_offset: usize = plane;
        let total_offset: usize = col_offset + row_offset + plane_offset;
        self.pixels[total_offset]
    }

    pub fn give_image(self) -> image::BmpImage {
        self.image
    }

    pub fn process(&self) {
        for row in 0..self.h() {
            for col in 0..self.w() {
                let pix = self.pixel_at(col, row, 0);
            }
        }
    }
}