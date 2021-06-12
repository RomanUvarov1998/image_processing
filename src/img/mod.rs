use std::{ops::{Index, IndexMut}, path::PathBuf, result, time};
use fltk::{enums::ColorDepth, image, prelude::ImageExt};
use crate::{filter::{filter_option::ExtendValue, filter_trait::Filter}, my_err::MyError};
use self::pixel_pos::PixelPos;

pub mod pixel_pos;

#[derive(Clone)]
pub struct Matrix2D {
    width: usize,
    height: usize,
    pixels: Vec<f64>
}

#[allow(unused)]
impl Matrix2D {
    pub fn empty_with_size(width: usize, height: usize) -> Self {
        let mut pixels = Vec::<f64>::new();
        pixels.resize(width * height, 0_f64);        
        Matrix2D { width, height, pixels }
    }

    pub fn load_as_grayed(path: PathBuf) -> result::Result<Self, MyError> {
        let im = fltk::image::SharedImage::load(path)?;
        let values = im.to_rgb_data();
        let mut values_grey: Vec<f64>;

        const RGB_2_GRAY_RED: f64 = 0.299;
        const RGB_2_GRAY_GREEN: f64 = 0.587;
        const RGB_2_GRAY_BLUE: f64 = 0.114;

        match im.depth() {
            ColorDepth::L8 => values_grey = im.to_rgb_data().into_iter().map(|v| v as f64).collect(),
            ColorDepth::La8 => {
                assert_eq!(values.len() % 2, 0);
                values_grey = Vec::<f64>::with_capacity(values.len() / 2);
                for i in 0..values.len() {
                    if i % 2 == 0 { values_grey.push(values[i] as f64); }
                }
            },
            ColorDepth::Rgb8 => {
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
            ColorDepth::Rgba8 => {
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
            im.w(), im.h(), ColorDepth::L8)?;
            
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

    pub fn get_description(&self) -> String {
        format!("изображение {}x{}", self.h(), self.w())
    }

    pub fn fits(&self, pos: PixelPos) -> bool {
        pos.col <= self.max_col() && pos.row <= self.max_row()
    }

    pub fn get_iterator(&self) -> ImgIterator {
        ImgIterator::for_full_image(self)
    }

    pub fn get_area_iter(&self, from: PixelPos, to_excluded: PixelPos) -> ImgIterator {
        ImgIterator::for_rect_area( from, to_excluded)
    }

    pub fn get_progress_iter<Cbk: Fn(usize)>(&self, progress_cbk: Cbk) -> ProgressIterator<Cbk> {
        ProgressIterator::<Cbk>::for_full_image(self, progress_cbk)
    }

    pub fn get_progress_iter_area<Cbk: Fn(usize)>(&self, from: PixelPos, to_excluded: PixelPos, progress_cbk: Cbk) -> ProgressIterator<Cbk> {
        ProgressIterator::<Cbk>::for_rect_area(from,to_excluded, progress_cbk)
    }

    pub fn get_drawable_copy(&self) -> Result<image::RgbImage, MyError> { 
        let im_rgb = image::RgbImage::new(
            self.pixels.iter().map(|v| *v as u8).collect::<Vec<u8>>().as_slice(), 
            self.width as i32, self.height as i32,  ColorDepth::L8)?;
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

    pub fn processed_copy<T: Filter, Cbk: Fn(usize)>(&self, filter: &T, progress_cbk: Cbk) -> Self {
        let result_img = self.clone();
        filter.filter(result_img, progress_cbk)
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


#[derive(Clone)]
pub struct Matrix3D {
    width: usize,
    height: usize,
    layers: Vec<Matrix2D>,
    color_depth: ColorDepth
}

#[allow(unused)]
impl Matrix3D {
    pub fn empty_with_size(width: usize, height: usize, color_depth: ColorDepth) -> Self {
        let mut layers = Vec::<Matrix2D>::new();

        let depth: usize = color_depth as u8 as usize;

        for _ in 0..depth {
            layers.push(Matrix2D::empty_with_size(width, height));
        }
        Matrix3D { width, height, layers, color_depth }
    }

    pub fn load_as_rgb(path: PathBuf) -> result::Result<Self, MyError> {
        let im = fltk::image::SharedImage::load(path)?;

        if im.w() < 0 { return Err(MyError::new("Ширина загруженного изображения < 0".to_string())); }
        if im.h() < 0 { return Err(MyError::new("Высота загруженного изображения < 0".to_string())); }
        let width = im.w() as usize;
        let height = im.h() as usize;
        let color_depth = im.depth();
        let all_pixels: Vec<f64> = im.to_rgb_data().into_iter().map(|v| v as f64).collect();

        let layers_count = color_depth as u8 as usize;
        assert_eq!(all_pixels.len() % layers_count, 0);
        let mut layers = Vec::<Matrix2D>::new();
        for _ in 0..layers_count {
            let layer = Matrix2D::empty_with_size(width, height);
            layers.push(layer);
        }

        for pixel_num in 0..all_pixels.len() {
            let layer_num = pixel_num % layers_count;
            let layer_pixel_num = pixel_num / layers_count;
            layers[layer_num].pixels[layer_pixel_num] = all_pixels[pixel_num];
        }

        Ok(Matrix3D {  width, height, layers, color_depth } )
    }

    pub fn w(&self) -> usize { self.width }
    pub fn h(&self) -> usize { self.height }
    pub fn d(&self) -> usize { self.color_depth as u8 as usize }

    pub fn max_col(&self) -> usize { self.width - 1 }
    pub fn max_row(&self) -> usize { self.height - 1 }
    pub fn max_layer(&self) -> usize { self.height - 1 }

    pub fn get_description(&self) -> String {
        format!("изображение {}x{}x{}", self.h(), self.w(), self.d())
    }

    pub fn layers<'own>(&'own self) -> &'own Vec<Matrix2D> { &self.layers }
    pub fn layers_mut<'own>(&'own mut self) -> &'own mut Vec<Matrix2D> { &mut self.layers }

    pub fn get_iterator(&self) -> ImgIterator {
        ImgIterator::for_full_image(&self.layers[0])
    }

    pub fn get_drawable_copy(&self) -> Result<image::RgbImage, MyError> { 
        let mut all_pixels = Vec::<u8>::with_capacity(self.w() * self.h() * self.d());

        let layer_length = self.w() * self.h(); 
        for pix_num in 0..layer_length {
            for layer in self.layers.iter() {
                all_pixels.push(layer.pixels[pix_num] as u8);
            }
        }

        let im_rgb = image::RgbImage::new(
            all_pixels.as_slice(), 
            self.width as i32, self.height as i32,  self.color_depth)?;

        Ok(im_rgb)
    }

    pub fn try_save(&self, path: &str) -> Result<(), MyError> {
        let mut img_to_save = bmp::Image::new(self.w() as u32, self.h() as u32);

        for pos in self.get_iterator() {
            let pixel = match self.color_depth {
                ColorDepth::L8 => {
                    let pix_val = self.layers[0][pos] as u8;
                    bmp::Pixel::new(pix_val, pix_val, pix_val)
                },
                ColorDepth::La8 => {
                    let pix_val = (self.layers[0][pos] * self.layers[1][pos]) as u8;
                    bmp::Pixel::new(pix_val, pix_val, pix_val)
                },
                ColorDepth::Rgb8 => {
                    let r = self.layers[0][pos] as u8;
                    let g = self.layers[1][pos] as u8;
                    let b = self.layers[2][pos] as u8;
                    bmp::Pixel::new(r, g, b)
                },
                ColorDepth::Rgba8 => {                    
                    let a: f64 = self.layers[3][pos];
                    let r = (self.layers[0][pos] * a) as u8;
                    let g = (self.layers[1][pos] * a) as u8;
                    let b = (self.layers[2][pos] * a) as u8;
                    bmp::Pixel::new(r, g, b)
                },
            };
            img_to_save.set_pixel(pos.col as u32, pos.row as u32, pixel);
        }

        img_to_save.save(path)?;

        Ok(())
    }

    pub fn copy_with_extended_borders(&self, with: ExtendValue, by_rows: usize, by_cols: usize) -> Self {
        let mut layers_ext = Vec::<Matrix2D>::new();

        for layer in self.layers().iter() {
            layers_ext.push(layer.copy_with_extended_borders(with, by_rows, by_cols));
        }

        Matrix3D { width: self.w(), height: self.h(), layers: layers_ext, color_depth: self.color_depth }
    }

    pub fn processed_copy<T: Filter, Cbk: Fn(usize) + Clone>(&self, filter: &T, progress_cbk: Cbk) -> Self {
        let mut result_img = self.clone();

        for layer_num in 0..result_img.layers.len() {
            match self.color_depth {
                ColorDepth::La8 => if layer_num == 1 { continue; },
                ColorDepth::Rgba8 => if layer_num == 1 && layer_num == 3 { continue; },
                ColorDepth::L8 | ColorDepth::Rgb8 => {}
            }

            let progress_start = 100 * layer_num / self.d();
            let progress_step = 100 / self.d();
            let progress_cbk_copy = progress_cbk.clone();
            let cbk = move |pr| progress_cbk_copy(progress_start + pr * progress_step / 100);

            result_img.layers[layer_num] = result_img.layers[layer_num].processed_copy(filter, cbk)
        }

        result_img
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
            cur_pos: PixelPos::new(0, 0),
        }
    }

    pub fn for_rect_area(top_left: PixelPos, bottom_right_excluded: PixelPos) -> Self {
        assert!(top_left.row < bottom_right_excluded.row);
        assert!(top_left.col < bottom_right_excluded.col);

        ImgIterator {
            top_left,
            bottom_right_excluded,
            cur_pos: top_left,
        }
    }

    pub fn fits(&self, pos: PixelPos) -> bool {
        let mut val = 
            self.top_left.col <= pos.col && pos.col < self.bottom_right_excluded.col 
            && self.top_left.row <= pos.row && pos.row < self.bottom_right_excluded.row;
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

pub struct ProgressIterator<Cbk: Fn(usize)> {
    top_left: PixelPos,
    bottom_right_excluded: PixelPos,
    cur_pos: PixelPos,
    progress_cbk: Cbk,
    prev_time: time::Instant
}

impl<Cbk: Fn(usize)> ProgressIterator<Cbk> {
    pub fn for_full_image(img: &Matrix2D, progress_cbk: Cbk) -> Self {
        ProgressIterator::<Cbk> {
            top_left: PixelPos::new(0, 0),
            bottom_right_excluded: PixelPos::new(img.h(), img.w()),
            cur_pos: PixelPos::new(0, 0),
            progress_cbk,
            prev_time: time::Instant::now()
        }
    }

    pub fn for_rect_area(top_left: PixelPos, bottom_right_excluded: PixelPos, progress_cbk: Cbk) -> Self {
        assert!(top_left.row < bottom_right_excluded.row);
        assert!(top_left.col < bottom_right_excluded.col);

        ProgressIterator::<Cbk>{
            top_left,
            bottom_right_excluded,
            cur_pos: top_left,
            progress_cbk,
            prev_time: time::Instant::now()
        }
    }
}

impl<Cbk: Fn(usize)> Iterator for ProgressIterator<Cbk> {
    type Item = PixelPos;

    fn next(&mut self) -> Option<PixelPos> {
        let curr = self.cur_pos;

        const MS_DELAY: u128 = 100;

        if self.cur_pos.col < self.bottom_right_excluded.col - 1 {
            self.cur_pos.col += 1;
            return Some(curr);
        } else if self.cur_pos.row < self.bottom_right_excluded.row - 1 {
            self.cur_pos.col = self.top_left.col;
            self.cur_pos.row += 1;

            if self.prev_time.elapsed().as_millis() > MS_DELAY {
                self.prev_time = time::Instant::now();
                (self.progress_cbk)(curr.row * 100 / (self.bottom_right_excluded.row - self.top_left.row));
            }

            return Some(curr);
        } else {
            self.cur_pos = self.top_left;
            return None;
        }        
    }
}