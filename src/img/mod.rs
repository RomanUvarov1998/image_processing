use std::{ops::{Index, IndexMut}, path::PathBuf, result};
use fltk::{enums::ColorDepth, image, prelude::ImageExt};
use crate::{filter::{filter_option::ExtendValue}, my_err::MyError};
use self::pixel_pos::PixelPos;

pub mod pixel_pos;
pub mod color_ops;

pub const PIXEL_VALUES_COUNT: usize = 256_usize;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum ImgChannel { L, R, G, B, A }

impl PartialEq for ImgChannel {
    fn eq(&self, other: &Self) -> bool {
        *self as u8 == *other as u8
    }
}


#[derive(Clone)]
pub struct ImgLayer {
    layer: Matrix2D,
    channel: ImgChannel,
}

impl ImgLayer {
    pub fn new(layer: Matrix2D, channel: ImgChannel) -> Self {
        ImgLayer{
            layer,
            channel
        }
    }
    
    pub fn empty_size_of(other: &ImgLayer) -> Self {      
        ImgLayer { layer: Matrix2D::empty_size_of(other.matrix()), channel: other.channel() }
    }

    pub fn channel(&self) -> ImgChannel { self.channel }

    pub fn w(&self) -> usize { self.layer.w() }
    pub fn h(&self) -> usize { self.layer.h() }

    pub fn matrix(&self) -> &Matrix2D { &self.layer }

    pub fn matrix_mut(&mut self) -> &mut Matrix2D { &mut self.layer }

    pub fn get_iter(&self) -> LayerIterator {
        LayerIterator::for_full_image(self.matrix())
    }

    pub fn get_area_iter(&self, from: PixelPos, to_excluded: PixelPos) -> ImgIterator {
        ImgIterator::for_rect_area( from, to_excluded)
    }

}

impl Index<usize> for ImgLayer {
    type Output = f64;

    fn index(&self, index: usize) -> &Self::Output {
        &self.layer.pixels[index]
    }
}

impl Index<PixelPos> for ImgLayer {
    type Output = f64;

    fn index(&self, index: PixelPos) -> &Self::Output {
        &self.layer[index]
    }
}

impl IndexMut<PixelPos> for ImgLayer {
    fn index_mut(&mut self, index: PixelPos) -> &mut Self::Output {
        &mut self.layer[index]
    }
}



#[derive(Clone)]
pub struct Matrix2D {
    width: usize,
    height: usize,
    pixels: Vec<f64>,
}

#[allow(unused)]
impl Matrix2D {
    pub fn empty_with_size(width: usize, height: usize) -> Self {
        let mut pixels = Vec::<f64>::new();
        pixels.resize(width * height, 0_f64);        
        Matrix2D { width, height, pixels }
    }

    pub fn empty_size_of(other: &Matrix2D) -> Self {
        let mut pixels = Vec::<f64>::new();
        pixels.resize(other.w() * other.h(), 0_f64);        
        Matrix2D { width: other.w(), height: other.h(), pixels }
    }

    pub fn w(&self) -> usize { self.width }
    pub fn h(&self) -> usize { self.height }

    pub fn size_vec(&self) -> PixelPos { PixelPos::new(self.h(), self.w()) }

    pub fn max_col(&self) -> usize { self.width - 1 }
    pub fn max_row(&self) -> usize { self.height - 1 }

    pub fn get_description(&self) -> String {
        format!("изображение {}x{}", self.h(), self.w())
    }

    pub fn fits(&self, pos: PixelPos) -> bool {
        pos.col <= self.max_col() && pos.row <= self.max_row()
    }

    pub fn get_iter(&self) -> LayerIterator {
        LayerIterator::for_full_image(self)
    }

    pub fn get_area_iter(&self, from: PixelPos, to_excluded: PixelPos) -> ImgIterator {
        ImgIterator::for_rect_area( from, to_excluded)
    }

    pub fn scalar_transform<Tr: Fn(f64) -> f64>(&self, tr: Tr) -> Self {
        let mut transformed = Self::empty_size_of(self);

        for pos in self.get_iter() {
            transformed[pos] = tr(self[pos]);
        }

        transformed
    }

    pub fn get_drawable_copy(&self) -> Result<image::RgbImage, MyError> { 
        let im_rgb = image::RgbImage::new(
            self.pixels.iter().map(|v| *v as u8).collect::<Vec<u8>>().as_slice(), 
            self.width as i32, self.height as i32,  ColorDepth::L8)?;
        Ok(im_rgb)
    }

    pub fn try_save(&self, path: &str) -> Result<(), MyError> {
        let mut img_to_save = bmp::Image::new(self.w() as u32, self.h() as u32);

        for pos in self.get_iter() {
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
pub struct Img {
    width: usize,
    height: usize,
    layers: Vec<ImgLayer>,
    color_depth: ColorDepth
}

#[allow(unused)]
impl Img {
    pub fn empty_with_size(width: usize, height: usize, color_depth: ColorDepth) -> Self {
        let mut layers = Vec::<ImgLayer>::new();

        match color_depth {
            ColorDepth::L8 => {
                layers.push(ImgLayer::new(
                    Matrix2D::empty_with_size(width, height), 
                    ImgChannel::L));
            },
            ColorDepth::La8 => {
                layers.push(ImgLayer::new(
                    Matrix2D::empty_with_size(width, height), 
                    ImgChannel::L));
                layers.push(ImgLayer::new(
                    Matrix2D::empty_with_size(width, height), 
                    ImgChannel::A));
            },
            ColorDepth::Rgb8 => {
                layers.push(ImgLayer::new(
                    Matrix2D::empty_with_size(width, height), 
                    ImgChannel::R));
                layers.push(ImgLayer::new(
                    Matrix2D::empty_with_size(width, height), 
                    ImgChannel::G));
                layers.push(ImgLayer::new(
                    Matrix2D::empty_with_size(width, height), 
                    ImgChannel::B));
            },
            ColorDepth::Rgba8 => {
                layers.push(ImgLayer::new(
                    Matrix2D::empty_with_size(width, height), 
                    ImgChannel::R));
                layers.push(ImgLayer::new(
                    Matrix2D::empty_with_size(width, height), 
                    ImgChannel::G));
                layers.push(ImgLayer::new(
                    Matrix2D::empty_with_size(width, height), 
                    ImgChannel::B));
                layers.push(ImgLayer::new(
                    Matrix2D::empty_with_size(width, height), 
                    ImgChannel::A));
            },
        }
        
        Img { width, height, layers, color_depth }
    }

    pub fn empty_size_of(other: &Img) -> Self {
        Self::empty_with_size(other.width, other.height, other.color_depth)
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

        let mut img = Img::empty_with_size(width, height, color_depth);

        for pixel_num in 0..all_pixels.len() {
            let layer_num = pixel_num % layers_count;
            let layer_pixel_num = pixel_num / layers_count;
            img.layer_mut(layer_num).matrix_mut().pixels[layer_pixel_num] = all_pixels[pixel_num];
        }

        Ok(img)
    }

    pub fn w(&self) -> usize { self.width }
    pub fn h(&self) -> usize { self.height }
    pub fn d(&self) -> usize { self.color_depth as u8 as usize }

    pub fn size_vec(&self) -> PixelPos { PixelPos::new(self.h(), self.w()) }

    pub fn max_col(&self) -> usize { self.width - 1 }
    pub fn max_row(&self) -> usize { self.height - 1 }
    pub fn max_layer(&self) -> usize { self.height - 1 }

    pub fn get_description(&self) -> String {
        format!("изображение {}x{}x{}", self.h(), self.w(), self.d())
    }

    pub fn layers<'own>(&'own self) -> &'own Vec<ImgLayer> { &self.layers }
    pub fn layers_mut<'own>(&'own mut self) -> &'own mut Vec<ImgLayer> { &mut self.layers }
    pub fn layer_mut<'own>(&'own mut self, ind: usize) -> &'own mut ImgLayer { &mut self.layers[ind] }
    pub fn layer<'own>(&'own self, ind: usize) -> &'own ImgLayer { &self.layers[ind] }

    pub fn get_pixels_iter(&self) -> ImgIterator {
        ImgIterator::for_full_image(self)
    }

    pub fn get_layers_iter<'own>(&'own self) -> LayersIterator<'own> {
        LayersIterator::new(self)
    }

    pub fn get_drawable_copy(&self) -> Result<image::RgbImage, MyError> { 
        let mut all_pixels = Vec::<u8>::with_capacity(self.w() * self.h() * self.d());

        let layer_length = self.w() * self.h(); 
        for pix_num in 0..layer_length {
            for layer in self.get_layers_iter() {
                all_pixels.push(layer[pix_num] as u8);
            }
        }

        let im_rgb = image::RgbImage::new(
            all_pixels.as_slice(), 
            self.width as i32, self.height as i32,  self.color_depth)?;

        Ok(im_rgb)
    }

    pub fn try_save(&self, path: &str) -> Result<(), MyError> {
        let mut img_to_save = bmp::Image::new(self.w() as u32, self.h() as u32);

        for pos in self.get_pixels_iter() {
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
        let mut layers_ext = Vec::<ImgLayer>::new();

        for layer in self.layers().iter() {
            let ext_layer = layer.matrix().copy_with_extended_borders(with, by_rows, by_cols);
            layers_ext.push(ImgLayer::new(ext_layer, layer.channel));
        }

        Img { width: self.w(), height: self.h(), layers: layers_ext, color_depth: self.color_depth }
    }
}


pub struct LayerIterator {
    top_left: PixelPos,
    bottom_right_excluded: PixelPos,
    cur_pos: PixelPos
}

#[allow(unused)]
impl LayerIterator {
    pub fn for_full_image(layer: &Matrix2D) -> Self {
        LayerIterator {
            top_left: PixelPos::new(0, 0),
            bottom_right_excluded: PixelPos::new(layer.h(), layer.w()),
            cur_pos: PixelPos::new(0, 0),
        }
    }

    pub fn for_rect_area(top_left: PixelPos, bottom_right_excluded: PixelPos) -> Self {
        assert!(top_left.row < bottom_right_excluded.row);
        assert!(top_left.col < bottom_right_excluded.col);

        LayerIterator {
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

impl Iterator for LayerIterator {
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


pub struct LayersIterator<'own> {
    img: &'own Img,
    curr_layer_num: usize
}

impl<'own> LayersIterator<'own> {
    pub fn new(img: &'own Img) -> Self {
        LayersIterator {
            img,
            curr_layer_num: 0
        }
    }
}

impl<'own> Iterator for LayersIterator<'own> {
    type Item = &'own ImgLayer;

    fn next(&mut self) -> Option<&'own ImgLayer> {
        let curr_num = self.curr_layer_num;

        if self.curr_layer_num < self.img.layers().len() {
            self.curr_layer_num += 1;
            let layer = self.img.layer(curr_num);
            return Some(layer);
        } else {
            self.curr_layer_num = 0;
            return None;
        }        
    }
}