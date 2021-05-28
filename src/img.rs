use std::{path::PathBuf, result};

use fltk::{image, prelude::ImageExt};
use crate::{filter::{self, MAX_WINDOW_BUFFER_SIZE, MAX_WINDOW_SIZE}, my_err::MyError, pixel_pos::PixelPos};

#[derive(Clone)]
pub struct Img {
    width: usize,
    height: usize,
    pixels: Vec<u8>
}

impl Img {
    pub fn load(path: PathBuf) -> result::Result<Self, MyError> {
        let im = fltk::image::SharedImage::load(path)?;
        let values = im.to_rgb_data();
        let mut values_grey: Vec<u8>;
        match im.depth() {
            fltk::enums::ColorDepth::L8 => values_grey = im.to_rgb_data(),
            fltk::enums::ColorDepth::La8 => {
                assert_eq!(values.len() % 2, 0);
                values_grey = Vec::<u8>::with_capacity(values.len() / 2);
                for i in 0..values.len() {
                    if i % 2 == 0 { values_grey.push(values[i]); }
                }
            },
            fltk::enums::ColorDepth::Rgb8 => {
                assert_eq!(values.len() % 3, 0);
                values_grey = Vec::<u8>::with_capacity(values.len() / 3);
                for i in 0..values.len() {
                    if i % 3 == 0 { 
                        let grey: u32 = 
                            values[i] as u32 * 299 / 1000 + 
                            values[i + 1] as u32 * 587 / 1000 + 
                            values[i + 2] as u32 * 114 / 1000;
                        values_grey.push(grey as u8); 
                    }
                }
            },
            fltk::enums::ColorDepth::Rgba8 => {
                assert_eq!(values.len() % 4, 0);
                values_grey = Vec::<u8>::with_capacity(values.len() / 3);
                for i in 0..values.len() {
                    if i % 4 == 0 { 
                        let grey: u32 = 
                            values[i] as u32 * 299 / 1000 + 
                            values[i + 1] as u32 * 587 / 1000 + 
                            values[i + 2] as u32 * 114 / 1000;
                        values_grey.push(grey as u8); 
                    }
                }
            }
        }

        let im_grey = fltk::image::RgbImage::new(&values_grey, 
            im.w(), im.h(), fltk::enums::ColorDepth::L8)?;
            
        let width = im.width() as usize;
        let height = im.height() as usize;
        let pixels = im_grey.to_rgb_data();

        Ok(Img {
            pixels,
            width,
            height
        })
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
        self.pixels[pos.row * self.width + pos.col] 
    }

    pub fn set_pixel(&mut self, pos: PixelPos, value: u8) {
        if !self.fits(pos) {
            panic!("pos is {:?} which is doesn't fit into {}, {}", pos, self.max_col(), self.max_row());
        }
        self.pixels[pos.row * self.width + pos.col] = value;
    }

    pub fn get_drawable_copy(&self) -> Result<image::RgbImage, MyError> { 
        let im_rgb = image::RgbImage::new(self.pixels.as_slice(), 
            self.width as i32, self.height as i32,  fltk::enums::ColorDepth::L8)?;
        Ok(im_rgb)
    }

    pub fn apply_filter<T: filter::Filter>(&self, filter: &mut T) -> Self {
        let mut result_img = self.clone();

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

            let filter_result = filter.filter(&mut pixel_buf[0..pixel_buf_actual_size]);
            result_img.set_pixel(pos_im, filter_result as u8);
        }

        result_img
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