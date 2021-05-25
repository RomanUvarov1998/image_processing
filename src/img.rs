use ImgLib::{GenericImage, GenericImageView, Pixel};
use fltk::{
    image
};
use ::image as ImgLib;
use crate::{filter, my_err::MyError, pixel_pos::PixelPos};

#[derive(Clone)]
pub struct Img {
    image: ImgLib::DynamicImage,
    width: usize,
    height: usize,
    pixels: Vec<u8>
}

pub const CHANNELS_COUNT: usize = 4;
pub const MAX_WINDOW_SIZE: usize = 11;
pub const MAX_WINDOW_BUFFER_SIZE: usize = MAX_WINDOW_SIZE * MAX_WINDOW_SIZE;

impl Img {
    pub fn new(image: ImgLib::DynamicImage) -> Self {
        let im = image.to_rgba8();
        let width = im.width() as usize;
        let height = im.height() as usize;
        let pixels = im.to_vec();

        Img {
            image,
            pixels,
            width,
            height
        }
    }

    pub fn w(&self) -> usize { self.width }
    pub fn h(&self) -> usize { self.height }

    pub fn max_col(&self) -> usize { self.width - 1 }
    pub fn max_row(&self) -> usize { self.height - 1 }

    pub fn fits(&self, pos: PixelPos) -> bool {
        pos.col <= self.max_col() && pos.row <= self.max_row()
    }

    pub fn get_iterator(&self) -> ImgIterator {
        ImgIterator {
            width: self.w(),
            height: self.h(),
            cur_pos: PixelPos::default()
        }
    }

    pub fn pixel_at(&self, pos: PixelPos) -> u8 {
        if !self.fits(pos) {
            panic!("pos is {:?} which is doesn't fit into {}, {}", pos, self.max_col(), self.max_row());
        }

        let col_offset: usize = pos.col * crate::img::CHANNELS_COUNT;
        let row_offset: usize = pos.row * self.w() * crate::img::CHANNELS_COUNT;
        let total_offset: usize = col_offset + row_offset;        
        
        self.pixels[total_offset]
    }

    pub fn give_image(self) -> Result<image::BmpImage, MyError> { 
        let mut bytes = Vec::<u8>::new();
        self.image.write_to(&mut bytes, ImgLib::ImageOutputFormat::Bmp)?;        
        let img_bmp = image::BmpImage::from_data(bytes.as_slice())?;
        Ok(img_bmp)
    }

    pub fn apply_filter<T: filter::Filter>(&mut self, filter: &mut T) -> Self {
        let mut result = self.clone();

        let pixel_buf_actual_size = filter.window_size() * filter.window_size();
        assert!(pixel_buf_actual_size < MAX_WINDOW_BUFFER_SIZE, 
            "filter size must be <= {}", MAX_WINDOW_SIZE);
        let mut pixel_buf = [0_f64; MAX_WINDOW_BUFFER_SIZE];

        let filter_ext = PixelPos::new(
            filter.window_size() / 2, 
            filter.window_size() / 2);

        for pos_im in self.get_iterator() {
            for pos_w in filter.get_iterator() {
                let mut pix_pos: PixelPos = pos_im + pos_w;

                let buf_ind: usize = pos_w.row * filter.window_size() + pos_w.col;
                
                if pix_pos.negative_if_substract(filter_ext) {
                    pixel_buf[buf_ind] = 0_f64; 
                    continue;
                }

                pix_pos -= filter_ext;
                
                if self.fits(pix_pos) {
                    pixel_buf[buf_ind] = self.pixel_at(pix_pos) as f64;
                } else {
                    pixel_buf[buf_ind] = 0_f64; 
                }
            }

            let filter_result = filter.filter(&pixel_buf[0..pixel_buf_actual_size]);
            let res = filter_result as u8;

            let alpha = result.image.get_pixel(pos_im.col as u32, pos_im.row as u32).channels()[3];
            result.image.put_pixel(pos_im.col as u32, pos_im.row as u32, 
                ImgLib::Rgba::<u8>::from_channels(res, res, res, alpha));
        }

        result
    }
}

pub struct ImgIterator {
    width: usize,
    height: usize,
    cur_pos: PixelPos
}

impl ImgIterator {
    pub fn fits(&self, pos: PixelPos) -> bool {
        pos.col <= self.max_col() && pos.row <= self.max_row()
    }
    pub fn max_col(&self) -> usize { self.width as usize - 1 }
    pub fn max_row(&self) -> usize { self.height as usize - 1 }
}

impl Iterator for ImgIterator {
    type Item = PixelPos;

    fn next(&mut self) -> Option<PixelPos> {
        let curr = self.cur_pos;

        assert!(self.fits(self.cur_pos));

        if self.cur_pos.col < self.max_col() {
            self.cur_pos.col += 1;
            return Some(curr);
        } else if self.cur_pos.row < self.max_row() {
            self.cur_pos.col = 0;
            self.cur_pos.row += 1;
            return Some(curr);
        } else {
            self.cur_pos = PixelPos::default();
            return None;
        }        
    }
}