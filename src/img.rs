use std::{path::PathBuf, result};

use fltk::{image, prelude::ImageExt};
use crate::{filter::{self, ExtendValue}, my_err::MyError, pixel_pos::PixelPos};

#[derive(Clone)]
pub struct Img {
    width: usize,
    height: usize,
    pixels: Vec<u8>
}

impl Img {
    pub fn empty_with_size(width: usize, height: usize) -> Self {
        let mut pixels = Vec::<u8>::new();
        pixels.resize(width * height, 0);        
        Img { width, height, pixels }
    }

    pub fn load(path: PathBuf) -> result::Result<Self, MyError> {
        let im = fltk::image::SharedImage::load(path)?;
        let values = im.to_rgb_data();
        let mut values_grey: Vec<u8>;

        const RGB_2_GRAY_RED_NUM: u32 = 299;
        const RGB_2_GRAY_GREEN_NUM: u32 = 587;
        const RGB_2_GRAY_BLUE_NUM: u32 = 114;
        const RGB_2_GRAY_DEN: u32 = 1000;

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
                            values[i] as u32 * RGB_2_GRAY_RED_NUM / RGB_2_GRAY_DEN + 
                            values[i + 1] as u32 * RGB_2_GRAY_GREEN_NUM / RGB_2_GRAY_DEN + 
                            values[i + 2] as u32 * RGB_2_GRAY_BLUE_NUM / RGB_2_GRAY_DEN;
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
                            values[i] as u32 * RGB_2_GRAY_RED_NUM / RGB_2_GRAY_DEN + 
                            values[i + 1] as u32 * RGB_2_GRAY_GREEN_NUM / RGB_2_GRAY_DEN + 
                            values[i + 2] as u32 * RGB_2_GRAY_BLUE_NUM / RGB_2_GRAY_DEN;
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
        ImgIterator::for_full_image(self)
    }

    pub fn get_area_iter(&self, from: PixelPos, to_excluded: PixelPos) -> ImgIterator 
    {
        ImgIterator::for_rect_area(self, from, to_excluded)
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

    pub fn copy_with_extended_borders(&self, with: ExtendValue, by_rows: usize, by_cols: usize) -> Self {
        let mut img = Self::empty_with_size(self.w() + by_cols * 2, self.h() + by_rows * 2);

        let origin = PixelPos::new(0, 0);

        let corner = PixelPos::new(by_rows, by_cols);
        let corner_row = PixelPos::new(corner.row, 0);
        let corner_col = PixelPos::new(0, corner.col);

        let horz_border_row = PixelPos::new(by_rows, 0);
        let horz_border_col = PixelPos::new(0, self.w());

        let vert_border = PixelPos::new(self.h(), by_cols);
        let vert_border_row = PixelPos::new(self.h(), 0);

        let img_size = PixelPos::new(self.h(), self.w());

        match with {
            ExtendValue::Closest => {
                // ------------------------------------ top ------------------------------------
                // top left
                img.set_rect(origin, corner, self.pixel_at(origin));
                // top middle
                for pos in self.get_area_iter(corner_col, corner + horz_border_col) {
                    img.set_pixel(pos, self.pixel_at(pos - pos.row_vec() - corner_col));
                }        
                // top right
                img.set_rect(corner_row + horz_border_row, corner_row + horz_border_row + corner, 
                    self.pixel_at(PixelPos::new(0, self.w() - 1)));
                
                // ------------------------------------ middle ------------------------------------   
                // middle left  
                for pos in self.get_area_iter(corner_row, corner_row + vert_border) {
                    img.set_pixel(pos, self.pixel_at(pos - pos.col_vec() - corner_row));
                }  
                // middle middle    
                for pos in self.get_area_iter(corner, corner + img_size) {
                    img.set_pixel(pos, self.pixel_at(pos - corner));
                }    
                // middle right
                for pos in self.get_area_iter(corner + horz_border_col, corner + horz_border_col + vert_border) {
                    img.set_pixel(pos, self.pixel_at(PixelPos::new(pos.row - corner.row, self.w() - 1)));
                } 
                
                // ------------------------------------ bottom ------------------------------------
                // bottom left
                img.set_rect(corner_row + vert_border_row, corner_row + vert_border_row + corner, 
                    self.pixel_at(PixelPos::new(self.h() - 1, 0)));
                // bottom middle
                for pos in self.get_area_iter(corner + vert_border_row, corner + img_size + horz_border_row) {
                    img.set_pixel(pos, self.pixel_at(PixelPos::new(self.h() - 1, pos.col - corner.col)));
                }        
                // bottom right
                img.set_rect(corner + img_size, corner + img_size + corner, 
                    self.pixel_at(PixelPos::new(self.h() - 1, self.w() - 1)));
            },
            ExtendValue::Given(ext_value) => {
                // ------------------------------------ top ------------------------------------
                // top left
                img.set_rect(origin, corner, self.pixel_at(origin));
                // top middle
                for pos in self.get_area_iter(corner_col, corner + horz_border_col) {
                    img.set_pixel(pos, ext_value);
                }        
                // top right
                img.set_rect(corner_row + horz_border_row, corner_row + horz_border_row + corner, ext_value);
                
                // ------------------------------------ middle ------------------------------------   
                // middle left  
                for pos in self.get_area_iter(corner_row, corner_row + vert_border) {
                    img.set_pixel(pos, ext_value);
                }  
                // middle middle      
                for pos in self.get_area_iter(corner, corner + img_size) {
                    img.set_pixel(pos, self.pixel_at(pos - corner));
                } 
                // middle right
                for pos in self.get_area_iter(corner + horz_border_col, corner + horz_border_col + vert_border) {
                    img.set_pixel(pos, ext_value);
                } 
                
                // ------------------------------------ bottom ------------------------------------
                // bottom left
                img.set_rect(corner_row + vert_border_row, corner_row + vert_border_row + corner, ext_value);
                // bottom middle
                for pos in self.get_area_iter(corner + vert_border_row, corner + img_size + horz_border_row) {
                    img.set_pixel(pos, ext_value);
                }        
                // bottom right
                img.set_rect(corner + img_size, corner + img_size + corner, ext_value);
            }
        }        

        img
    }

    pub fn clip(&self, top_left: PixelPos, bottom_right_ex: PixelPos, mut to: Img) -> Self {
        let new_width: usize = bottom_right_ex.col - top_left.col;
        let new_height: usize = bottom_right_ex.col - top_left.col;

        to.pixels.resize(new_width * new_height, 0);

        for pos in self.get_area_iter(top_left, bottom_right_ex) {
            to.set_pixel(pos - top_left, self.pixel_at(pos));
        }
        
        to
    }

    pub fn processed_copy<T: filter::Filter>(&self, filter: &mut T) -> Self {
        let result_img = self.clone();
        filter.filter(result_img)
    }

    fn set_rect(&mut self, tl: PixelPos, br: PixelPos, value: u8) -> () {
        for pos in self.get_area_iter(tl, br) {
            self.set_pixel(pos, value);
        }
    }
}

pub struct ImgIterator {
    top_left: PixelPos,
    bottom_right_excluded: PixelPos,
    cur_pos: PixelPos
}

impl ImgIterator {
    pub fn for_full_image(img: &Img) -> Self {
        ImgIterator {
            top_left: PixelPos::new(0, 0),
            bottom_right_excluded: PixelPos::new(img.h(), img.w()),
            cur_pos: PixelPos::new(0, 0)
        }
    }

    pub fn for_rect_area(img: &Img, top_left: PixelPos, bottom_right_excluded: PixelPos) -> Self {
        assert!(top_left.row < bottom_right_excluded.row);
        assert!(top_left.col < bottom_right_excluded.col);

        ImgIterator {
            top_left,
            bottom_right_excluded,
            cur_pos: top_left
        }
    }

    pub fn fits(&self, pos: PixelPos) -> bool {
        let mut val = 
        self.top_left.col <= pos.col && pos.col < self.bottom_right_excluded.col 
        && self.top_left.row <= pos.row && pos.row < self.bottom_right_excluded.row ;
        if val == false {
            val = true;
        }
        val
    }
}

impl Iterator for ImgIterator {
    type Item = PixelPos;

    fn next(&mut self) -> Option<PixelPos> {
        let curr = self.cur_pos;

        assert!(self.fits(self.cur_pos));

        if self.cur_pos.col < self.bottom_right_excluded.col - 1 {
            self.cur_pos.col += 1;
            return Some(curr);
        } else if self.cur_pos.row < self.bottom_right_excluded.row - 1 {
            self.cur_pos.col = self.top_left.col;
            self.cur_pos.row += 1;
            return Some(curr);
        } else {
            self.cur_pos = self.top_left;
            return None;
        }        
    }
}