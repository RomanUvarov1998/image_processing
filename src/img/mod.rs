pub mod pixel_pos;

use std::{ops::{Index, IndexMut}, path::PathBuf, result};
use fltk::{app::Sender, image, prelude::ImageExt};
use crate::{filter::{filter_option::ExtendValue, filter_trait::Filter}, my_app::Message, my_err::MyError};

use self::pixel_pos::PixelPos;

#[derive(Clone)]
pub struct Matrix2D {
    width: usize,
    height: usize,
    pixels: Vec<f64>
}

impl Matrix2D {
    pub fn empty_with_size(width: usize, height: usize) -> Self {
        let mut pixels = Vec::<f64>::new();
        pixels.resize(width * height, 0_f64);        
        Matrix2D { width, height, pixels }
    }

    pub fn load(path: PathBuf) -> result::Result<Self, MyError> {
        let im = fltk::image::SharedImage::load(path)?;
        let values = im.to_rgb_data();
        let mut values_grey: Vec<f64>;

        const RGB_2_GRAY_RED: f64 = 0.299;
        const RGB_2_GRAY_GREEN: f64 = 0.587;
        const RGB_2_GRAY_BLUE: f64 = 0.114;

        match im.depth() {
            fltk::enums::ColorDepth::L8 => values_grey = im.to_rgb_data().into_iter().map(|v| v as f64).collect(),
            fltk::enums::ColorDepth::La8 => {
                assert_eq!(values.len() % 2, 0);
                values_grey = Vec::<f64>::with_capacity(values.len() / 2);
                for i in 0..values.len() {
                    if i % 2 == 0 { values_grey.push(values[i] as f64); }
                }
            },
            fltk::enums::ColorDepth::Rgb8 => {
                assert_eq!(values.len() % 3, 0);
                values_grey = Vec::<f64>::with_capacity(values.len() / 3);
                for i in 0..values.len() {
                    if i % 3 == 0 { 
                        let grey: f64 = 
                            values[i] as f64 * RGB_2_GRAY_RED + 
                            values[i + 1] as f64 * RGB_2_GRAY_GREEN + 
                            values[i + 2] as f64 * RGB_2_GRAY_BLUE;
                        values_grey.push(grey); 
                    }
                }
            },
            fltk::enums::ColorDepth::Rgba8 => {
                assert_eq!(values.len() % 4, 0);
                values_grey = Vec::<f64>::with_capacity(values.len() / 3);
                for i in 0..values.len() {
                    if i % 4 == 0 { 
                        let grey: f64 = 
                        values[i] as f64 * RGB_2_GRAY_RED + 
                        values[i + 1] as f64 * RGB_2_GRAY_GREEN + 
                        values[i + 2] as f64 * RGB_2_GRAY_BLUE;
                        values_grey.push(grey); 
                    }
                }
            }
        }

        let im_grey = fltk::image::RgbImage::new(
            &values_grey.into_iter().map(|v| v as u8).collect::<Vec<u8>>(), 
            im.w(), im.h(), fltk::enums::ColorDepth::L8)?;
            
        let width = im.width() as usize;
        let height = im.height() as usize;
        let pixels = im_grey.to_rgb_data().into_iter().map(|v| v as f64).collect();

        Ok(Matrix2D {
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

    pub fn get_area_iter(&self, from: PixelPos, to_excluded: PixelPos) -> ImgIterator {
        ImgIterator::for_rect_area( from, to_excluded)
    }

    pub fn get_drawable_copy(&self) -> Result<image::RgbImage, MyError> { 
        let im_rgb = image::RgbImage::new(
            self.pixels.iter().map(|v| *v as u8).collect::<Vec<u8>>().as_slice(), 
            self.width as i32, self.height as i32,  fltk::enums::ColorDepth::L8)?;
        Ok(im_rgb)
    }

    pub fn try_save(&self, path: &str) -> Result<(), MyError> {
        let mut img_to_save = bmp::Image::new(self.w() as u32, self.h() as u32);

        for pos in self.get_iterator() {
            let pix = bmp::Pixel::new(self[pos] as u8, self[pos] as u8, self[pos] as u8);
            img_to_save.set_pixel(pos.col as u32, pos.row as u32, pix);
        }

        img_to_save.save(path)?;

        Ok(())
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
                img.set_rect(origin, corner, self[origin]);
                // top middle
                for pos in self.get_area_iter(corner_col, corner + horz_border_col) {
                    img[pos] = self[pos - pos.row_vec() - corner_col];
                }        
                // top right
                img.set_rect(corner_row + horz_border_row, corner_row + horz_border_row + corner, 
                    self[PixelPos::new(0, self.w() - 1)]);
                
                // ------------------------------------ middle ------------------------------------   
                // middle left  
                for pos in self.get_area_iter(corner_row, corner_row + vert_border) {
                    img[pos] = self[pos - pos.col_vec() - corner_row];
                }  
                // middle middle    
                for pos in self.get_area_iter(corner, corner + img_size) {
                    img[pos] = self[pos - corner];
                }    
                // middle right
                for pos in self.get_area_iter(corner + horz_border_col, corner + horz_border_col + vert_border) {
                    img[pos] = self[PixelPos::new(pos.row - corner.row, self.w() - 1)];
                } 
                
                // ------------------------------------ bottom ------------------------------------
                // bottom left
                img.set_rect(corner_row + vert_border_row, corner_row + vert_border_row + corner, 
                    self[PixelPos::new(self.h() - 1, 0)]);
                // bottom middle
                for pos in self.get_area_iter(corner + vert_border_row, corner + img_size + horz_border_row) {
                    img[pos] = self[PixelPos::new(self.h() - 1, pos.col - corner.col)];
                }        
                // bottom right
                img.set_rect(corner + img_size, corner + img_size + corner, 
                    self[PixelPos::new(self.h() - 1, self.w() - 1)]);
            },
            ExtendValue::Given(ext_value) => {
                // ------------------------------------ top ------------------------------------
                // top left
                img.set_rect(origin, corner, self[origin]);
                // top middle
                for pos in self.get_area_iter(corner_col, corner + horz_border_col) {
                    img[pos] = ext_value;
                }        
                // top right
                img.set_rect(corner_row + horz_border_row, corner_row + horz_border_row + corner, ext_value);
                
                // ------------------------------------ middle ------------------------------------   
                // middle left  
                for pos in self.get_area_iter(corner_row, corner_row + vert_border) {
                    img[pos] = ext_value;
                }  
                // middle middle      
                for pos in self.get_area_iter(corner, corner + img_size) {
                    img[pos] = self[pos - corner];
                } 
                // middle right
                for pos in self.get_area_iter(corner + horz_border_col, corner + horz_border_col + vert_border) {
                    img[pos] = ext_value;
                } 
                
                // ------------------------------------ bottom ------------------------------------
                // bottom left
                img.set_rect(corner_row + vert_border_row, corner_row + vert_border_row + corner, ext_value);
                // bottom middle
                for pos in self.get_area_iter(corner + vert_border_row, corner + img_size + horz_border_row) {
                    img[pos] = ext_value;
                }        
                // bottom right
                img.set_rect(corner + img_size, corner + img_size + corner, ext_value);
            }
        }        

        img
    }

    pub fn processed_copy<T: Filter>(&self, filter: &T, step_num: usize, sender: Sender<Message>) -> Self {
        let result_img = self.clone();
        filter.filter(result_img, step_num, sender)
    }

    fn set_rect(&mut self, tl: PixelPos, br: PixelPos, value: f64) -> () {
        for pos in self.get_area_iter(tl, br) {
            self[pos] = value;
        }
    }
}

impl Index<PixelPos> for Matrix2D {
    type Output = f64;

    fn index(&self, index: PixelPos) -> &Self::Output {
        if !self.fits(index) {
            panic!("pos is {:?} which is doesn't fit into {}, {}", index, self.max_col(), self.max_row());
        }
        &self.pixels[index.row * self.width + index.col]
    }
}

impl IndexMut<PixelPos> for Matrix2D {
    fn index_mut(&mut self, index: PixelPos) -> &mut Self::Output {
        if !self.fits(index) {
            panic!("pos is {:?} which is doesn't fit into {}, {}", index, self.max_col(), self.max_row());
        }
        &mut self.pixels[index.row * self.width + index.col]
    }
}

pub struct ImgIterator {
    top_left: PixelPos,
    bottom_right_excluded: PixelPos,
    cur_pos: PixelPos
}

impl ImgIterator {
    pub fn for_full_image(img: &Matrix2D) -> Self {
        ImgIterator {
            top_left: PixelPos::new(0, 0),
            bottom_right_excluded: PixelPos::new(img.h(), img.w()),
            cur_pos: PixelPos::new(0, 0)
        }
    }

    pub fn for_rect_area(top_left: PixelPos, bottom_right_excluded: PixelPos) -> Self {
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