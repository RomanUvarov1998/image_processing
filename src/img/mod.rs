use std::{ops::{Index, IndexMut}, path::PathBuf, result};
use fltk::{enums::ColorDepth, image::{self}, prelude::ImageExt};
use crate::my_err::MyError;
use self::{filter::filter_trait::WindowFilter, pixel_pos::PixelPos};
use self::filter::filter_option::*;

pub mod filter;
pub mod pixel_pos;

pub const PIXEL_VALUES_COUNT: usize = 256_usize;


#[derive(Clone)]
pub struct ImgLayer {
    layer: Matrix2D,
    channel: ImgChannel,
}

#[allow(unused)]
impl ImgLayer {
    pub fn new(layer: Matrix2D, channel: ImgChannel) -> Self {
        ImgLayer{
            layer,
            channel
        }
    }
    
    pub fn channel(&self) -> ImgChannel { self.channel }

    pub fn w(&self) -> usize { self.layer.w() }
    pub fn h(&self) -> usize { self.layer.h() }

    pub fn matrix(&self) -> &Matrix2D { &self.layer }

    pub fn matrix_mut(&mut self) -> &mut Matrix2D { &mut self.layer }

    pub fn get_iter(&self) -> PixelsIterator {
        PixelsIterator::for_full_image(self.matrix())
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

    pub fn fits(&self, pos: PixelPos) -> bool {
        pos.col <= self.max_col() && pos.row <= self.max_row()
    }

    pub fn get_pixels_iter(&self) -> PixelsIterator {
        PixelsIterator::for_full_image(self)
    }

    pub fn get_pixels_area_iter(&self, tl: PixelPos, br_excluded: PixelPos) -> PixelsIterator {
        assert!(self.fits(tl));
        assert!(br_excluded.row > 0);
        assert!(br_excluded.col > 0);
        assert!(self.fits(br_excluded - PixelPos::one()));
        PixelsIterator::for_rect_area( tl, br_excluded)
    }

    pub fn scalar_transform_into<Tr: Fn(&Matrix2D, PixelPos) -> f64>(&self, area: PixelsArea, tr: Tr, dest_matrix: &mut Matrix2D) {
        for pos in self.get_pixels_area_iter(area.top_left, area.bottom_right) {
            dest_matrix[pos] = tr(dest_matrix, pos);
        }
    }

    pub fn scalar_transform<Tr: Fn(&Matrix2D, PixelPos) -> f64>(&self, area: PixelsArea, tr: Tr) -> Self {
        let mut transformed = Self::empty_size_of(self);
        self.scalar_transform_into(area, tr, &mut transformed);
        transformed
    }

    pub fn get_drawable_copy(&self) -> Result<image::RgbImage, MyError> { 
        let im_rgb = image::RgbImage::new(
            self.pixels.iter().map(|v| *v as u8).collect::<Vec<u8>>().as_slice(), 
            self.width as i32, self.height as i32,  ColorDepth::L8)?;
        Ok(im_rgb)
    }

    pub fn pixels<'own>(&'own self) -> &'own Vec<f64> {
        &self.pixels
    }

    pub fn extended_for_window_filter<F: WindowFilter>(&self, filter: &F) -> Matrix2D {
        let left = filter.w() / 2;
        let top = filter.h() / 2;
        let right = left;
        let bottom = top;
    
        self.extended( filter.get_extend_value(), left, top, right, bottom)
    }
    
    pub fn extended(
        &self, 
        with: ExtendValue, 
        left: usize, top: usize, right: usize, bottom: usize
    ) -> Matrix2D {
        let mut mat_ext = Matrix2D::empty_with_size(left + self.w() + right, top + self.h() + bottom);
    
        let origin = PixelPos::new(0, 0);
    
        let margin_left = PixelPos::new(0, left);
        let margin_top = PixelPos::new(top, 0);
        let margin_right = PixelPos::new(0, right);
        let margin_bottom = PixelPos::new(bottom, 0);
    
        let rect_left = PixelPos::new(self.h(), left);
        let rect_top = PixelPos::new(top, self.w());
        let rect_right = PixelPos::new(self.h(), right);
        let rect_bottom = PixelPos::new(bottom, self.w());
    
        let mat_size = PixelPos::new(self.h(), self.w());
    
        // ------------------------------------ top ------------------------------------
        if top > 0 {
            // top left
            if left > 0 {
                let tl = origin;
                let br_excluded = tl + margin_left + margin_top;
                match with {
                    ExtendValue::Closest => mat_ext.set_rect(tl, br_excluded, self[origin]),
                    ExtendValue::Given(val) => mat_ext.set_rect(tl, br_excluded, val),
                }
            }
            // top middle
            let tl = margin_left;
            let br_excluded = tl + rect_top;
            match with {
                ExtendValue::Closest => {
                    for pos in mat_ext.get_pixels_area_iter(tl, br_excluded) {
                        mat_ext[pos] = self[pos.with_row(0) - margin_left];
                    }  
                },
                ExtendValue::Given(val) => mat_ext.set_rect(tl, br_excluded, val),
            }    
            // top right
            if right > 0 { 
                let tl = margin_left + rect_top.col_vec();
                let br_excluded = tl + margin_right + margin_top;
                match with {
                    ExtendValue::Closest => mat_ext.set_rect(tl, br_excluded, self[PixelPos::new(0, self.w() - 1)]),
                    ExtendValue::Given(val) => mat_ext.set_rect(tl, br_excluded, val),
                }
            }
        }
        // ------------------------------------ middle ------------------------------------   
        // middle left  
        if left > 0 {
            let tl = margin_top;
            let br_excluded = tl + rect_left;
            match with {
                ExtendValue::Closest => {
                    for pos in mat_ext.get_pixels_area_iter(tl, br_excluded) {
                        mat_ext[pos] = self[pos.with_col(0) - margin_top];
                    }
                },
                ExtendValue::Given(val) => mat_ext.set_rect(tl, br_excluded, val),
            }
        }
        // middle middle     
        let tl = margin_left + margin_top;
        let br_excluded = tl + mat_size;               
        for pos in mat_ext.get_pixels_area_iter(tl, br_excluded) {
            mat_ext[pos] = self[pos - tl];
        }    
        // middle right
        if right > 0 { 
            let tl = margin_left + rect_top;
            let br_excluded = tl + rect_right;
            match with {
                ExtendValue::Closest => {          
                    for pos in mat_ext.get_pixels_area_iter(tl, br_excluded) {
                        mat_ext[pos] = self[pos.with_col(self.w() - 1) - margin_top];
                    } 
                },
                ExtendValue::Given(val) => mat_ext.set_rect(tl, br_excluded, val),
            }
        }
        
        // ------------------------------------ bottom ------------------------------------
        if bottom > 0 {
            // bottom left
            if left > 0{
                let tl = margin_top + rect_left.row_vec();
                let br_excluded = tl + margin_left + margin_bottom;
                match with {
                    ExtendValue::Closest => mat_ext.set_rect(tl, br_excluded, self[PixelPos::new(self.h() - 1, 0)]),
                    ExtendValue::Given(val) => mat_ext.set_rect(tl, br_excluded, val),
                }
            }
            // bottom middle
            let tl = margin_top + rect_left;
            let br_excluded = tl + rect_bottom;
            match with {
                ExtendValue::Closest => {   
                    for pos in mat_ext.get_pixels_area_iter(tl, br_excluded) {
                        mat_ext[pos] = self[pos.with_row(self.h() - 1) - margin_left];
                    } 
                },
                ExtendValue::Given(val) => mat_ext.set_rect(tl, br_excluded, val),
            }       
            // bottom right
            if right > 0 {
                let tl = margin_left + margin_top + self.size_vec();
                let br_excluded = tl + margin_right + margin_bottom;
                match with {
                    ExtendValue::Closest => mat_ext.set_rect(tl, br_excluded, self[self.size_vec() - PixelPos::one()]),
                    ExtendValue::Given(val) => mat_ext.set_rect(tl, br_excluded, val),
                }
            }
        }
    
        mat_ext
    }

    fn set_rect(&mut self, tl: PixelPos, br_excluded: PixelPos, value: f64) -> () {
        for pos in self.get_pixels_area_iter(tl, br_excluded) {
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

impl Img {
    pub fn new(width: usize, height: usize, layers: Vec<ImgLayer>, color_depth: ColorDepth) -> Self {
        assert!(layers.len() > 0);
        Img { width, height, layers, color_depth }
    }

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

    pub fn from<T: ImageExt>(sh_im: T) -> Self {
        let width = sh_im.w() as usize;
        let height = sh_im.h() as usize;
        let color_depth = sh_im.depth();
        let all_pixels: Vec<f64> = sh_im.to_rgb_data().into_iter().map(|v| v as f64).collect();

        let layers_count = color_depth as u8 as usize;
        assert_eq!(all_pixels.len() % layers_count, 0);

        let mut img = Img::empty_with_size(width, height, color_depth);

        for pixel_num in 0..all_pixels.len() {
            let layer_num = pixel_num % layers_count;
            let layer_pixel_num = pixel_num / layers_count;
            img.layer_mut(layer_num).matrix_mut().pixels[layer_pixel_num] = all_pixels[pixel_num];
        }

        img
    }

    pub fn w(&self) -> usize { self.width }
    pub fn h(&self) -> usize { self.height }
    pub fn d(&self) -> usize { self.color_depth as u8 as usize }
    pub fn color_depth(&self) -> ColorDepth { self.color_depth }

    pub fn size_vec(&self) -> PixelPos { PixelPos::new(self.h(), self.w()) }

    pub fn max_col(&self) -> usize { self.width - 1 }
    pub fn max_row(&self) -> usize { self.height - 1 }
    pub fn max_layer(&self) -> usize { self.d() - 1 }

    pub fn get_description(&self) -> String {
        format!("Изображение {} (строк) x {} (столбцов) x {} (каналов)", self.h(), self.w(), self.d())
    }

    pub fn layers<'own>(&'own self) -> &'own Vec<ImgLayer> { &self.layers }
    pub fn layers_mut<'own>(&'own mut self) -> &'own mut Vec<ImgLayer> { &mut self.layers }
    pub fn layer_mut<'own>(&'own mut self, ind: usize) -> &'own mut ImgLayer { &mut self.layers[ind] }
    pub fn layer<'own>(&'own self, ind: usize) -> &'own ImgLayer { &self.layers[ind] }

    pub fn get_cropped_copy(&self, area: PixelsArea) -> Img {
        assert!(area.bottom_right.col <= self.w());
        assert!(area.bottom_right.row <= self.h());

        let mut img = Img::empty_with_size(
            area.bottom_right.col - area.top_left.col, 
            area.bottom_right.row - area.top_left.row, 
            self.color_depth());
        
        for pos in PixelsIterator::for_rect_area(area.top_left, area.bottom_right) {
            for ch_num in 0..self.d() {
                img.layer_mut(ch_num)[pos - area.top_left] = self.layer(ch_num)[pos];
            }
        }

        img
    }

    pub fn get_pixels_iter(&self) -> PixelsIterator {
        PixelsIterator::for_full_image(self.layer(0).matrix())
    }

    pub fn get_layers_iter<'own>(&'own self) -> LayersIterator<'own> {
        LayersIterator::new(self)
    }

    pub fn get_drawable_copy(&self) -> image::RgbImage { 
        let mut all_pixels = Vec::<u8>::with_capacity(self.w() * self.h() * self.d());

        let layer_length = self.w() * self.h(); 
        for pix_num in 0..layer_length {
            for layer in self.get_layers_iter() {
                all_pixels.push(layer[pix_num] as u8);
            }
        }

        let im_rgb = image::RgbImage::new(
            all_pixels.as_slice(), 
            self.width as i32, self.height as i32,  self.color_depth).unwrap();

        im_rgb
    }

    pub fn extended(&self, with: ExtendValue, left: usize, top: usize, right: usize, bottom: usize) -> Img {
        let mut ext_layers = Vec::<ImgLayer>::with_capacity(self.d());
    
        for layer in self.layers() {
            let ext_layer = match layer.channel() {
                ImgChannel::A => {
                    let mut ext_mat = Matrix2D::empty_with_size(left + layer.w() + right, top + layer.h() + bottom);
                    ext_mat.set_rect(PixelPos::new(0, 0), ext_mat.size_vec(), 255_f64);
                    ImgLayer::new(ext_mat, layer.channel())
                },
                _ => {
                    let ext_mat = layer.matrix().extended(with, left, top, right, bottom);
                    ImgLayer::new(ext_mat, layer.channel())
                },
            };
    
            ext_layers.push(ext_layer);
        }
    
        Img::new(left + self.w() + right, top + self.h() + bottom, ext_layers, self.color_depth())
    }

    pub fn try_save(&self, path: &str) -> Result<(), MyError> {
        use jpeg_encoder::{Encoder, ColorType};

        let (pixels, color_type): (Vec<u8>, ColorType) = match self.color_depth() {
            ColorDepth::L8 | ColorDepth::La8 => {
                let vals: Vec<u8> = self.layer(0).matrix().pixels
                    .iter()
                    .map(|p| *p as u8)
                    .collect();

                (vals, ColorType::Luma)
            },
            ColorDepth::Rgb8 | ColorDepth::Rgba8 => {
                let mut vals = Vec::<u8>::with_capacity(self.w() * self.h() * 3);
                
                let r = &self.layer(0).matrix().pixels;
                let g = &self.layer(1).matrix().pixels;
                let b = &self.layer(2).matrix().pixels;

                for pix_num in 0..self.w() * self.h() {
                    vals.push(r[pix_num] as u8);
                    vals.push(g[pix_num] as u8);
                    vals.push(b[pix_num] as u8);
                }

                assert_eq!(vals.len(), self.w() * self.h() * 3);
                    
                (vals, ColorType::Rgb)
            },
        };

        let encoder = Encoder::new_file(path, 100)?;
        encoder.encode(&pixels, self.w() as u16, self.h() as u16, color_type)?;

        Ok(())
    }
}


pub struct PixelsIterator {
    top_left: PixelPos,
    bottom_right_excluded: PixelPos,
    cur_pos: PixelPos,
}

impl PixelsIterator {
    pub fn for_full_image(layer: &Matrix2D) -> Self {
        PixelsIterator {
            top_left: PixelPos::new(0, 0),
            bottom_right_excluded: PixelPos::new(layer.h(), layer.w()),
            cur_pos: PixelPos::new(0, 0),
        }
    }

    pub fn for_rect_area(top_left: PixelPos, bottom_right_excluded: PixelPos) -> Self {
        assert!(top_left.row < bottom_right_excluded.row);
        assert!(top_left.col < bottom_right_excluded.col);

        PixelsIterator {
            top_left,
            bottom_right_excluded,
            cur_pos: top_left,
        }
    }

    pub fn fits(&self, pos: PixelPos) -> bool {
        let val = 
            self.top_left.col <= pos.col && pos.col < self.bottom_right_excluded.col 
            && self.top_left.row <= pos.row && pos.row < self.bottom_right_excluded.row;
        val
    }
}

impl Iterator for PixelsIterator {
    type Item = PixelPos;

    fn next(&mut self) -> Option<PixelPos> {
        let curr = self.cur_pos;

        self.cur_pos.col += 1;

        if self.cur_pos.col >= self.bottom_right_excluded.col {
            self.cur_pos.col = self.top_left.col;
            self.cur_pos.row += 1;
        }

        return if self.fits(curr) {
            Some(curr)
        } else {
            None
        };
    }
}


pub struct RowsIter {
    left_col: usize,
    right_col_excluded: usize,
    max_row_excluded: usize,
    cur_row: usize,
}

impl RowsIter {
    pub fn new(area: PixelsArea) -> Self {
        RowsIter {
            left_col: area.top_left().col,
            right_col_excluded: area.bottom_right().col,
            max_row_excluded: area.bottom_right().row,
            cur_row: area.top_left().col,
        }
    }
}

impl Iterator for RowsIter {
    type Item = PixelsRow;

    fn next(&mut self) -> Option<Self::Item> {
        let curr_row = self.cur_row;

        self.cur_row += 1;

        if curr_row < self.max_row_excluded {
            let left = PixelPos::new(curr_row, self.left_col);
            let right = PixelPos::new(curr_row, self.right_col_excluded);
            Some(PixelsRow::new(left, right))
        } else {
            None
        }
    }
}


#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PixelsRow {
    left: PixelPos,
    right: PixelPos
}

impl PixelsRow {
    pub fn new(left: PixelPos, right: PixelPos) -> Self {
        assert!(left.row == right.row);
        assert!(left.col <= right.col);
        PixelsRow {
            left,
            right
        }
    }

    pub fn get_cols_iter(&self) -> ColsIter {
        ColsIter::new(self)
    }
}


pub struct ColsIter {
    row: usize,
    right_col_excluded: usize,
    cur_col: usize,
}

impl ColsIter {
    pub fn new(row: &PixelsRow) -> Self {
        ColsIter {
            row: row.left.row,
            right_col_excluded: row.right.col,
            cur_col: row.left.col,
        }
    }
}

impl Iterator for ColsIter {
    type Item = PixelPos;

    fn next(&mut self) -> Option<Self::Item> {
        let cur_col = self.cur_col;

        self.cur_col += 1;

        if cur_col < self.right_col_excluded {
            Some(PixelPos::new(self.row, cur_col))
        } else {
            None
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


#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PixelsArea {
    top_left: PixelPos,
    bottom_right: PixelPos
}

impl PixelsArea {
    pub fn new(top_left: PixelPos, bottom_right: PixelPos) -> Self {
        PixelsArea { top_left, bottom_right }
    }

    pub fn top_left(&self) -> PixelPos {
        self.top_left
    }

    pub fn bottom_right(&self) -> PixelPos {
        self.bottom_right
    }
}


#[cfg(test)]
mod tests {
    use super::PixelPos;

    #[test]
    fn pixels_iter_for_area_returns_all_positions() {
        let mut iter = super::PixelsIterator::for_rect_area(
            PixelPos::new(0, 0),
            PixelPos::new(3, 3));

        assert_eq!(iter.next().unwrap(), PixelPos::new(0, 0));
        assert_eq!(iter.next().unwrap(), PixelPos::new(0, 1));
        assert_eq!(iter.next().unwrap(), PixelPos::new(0, 2));

        assert_eq!(iter.next().unwrap(), PixelPos::new(1, 0));
        assert_eq!(iter.next().unwrap(), PixelPos::new(1, 1));
        assert_eq!(iter.next().unwrap(), PixelPos::new(1, 2));

        assert_eq!(iter.next().unwrap(), PixelPos::new(2, 0));
        assert_eq!(iter.next().unwrap(), PixelPos::new(2, 1));
        assert_eq!(iter.next().unwrap(), PixelPos::new(2, 2));

        assert_eq!(iter.next(), None);
    }

    #[test]
    fn rows_iter_for_area_returns_all_positions() {
        let area = super::PixelsArea::new(
            PixelPos::new(0, 0),
            PixelPos::new(3, 3));
        let mut iter = super::RowsIter::new(area);
    
        use super::PixelsRow;

        assert_eq!(iter.next().unwrap(), PixelsRow::new(PixelPos::new(0, 0), PixelPos::new(0, 3)));
        assert_eq!(iter.next().unwrap(), PixelsRow::new(PixelPos::new(1, 0), PixelPos::new(1, 3)));
        assert_eq!(iter.next().unwrap(), PixelsRow::new(PixelPos::new(2, 0), PixelPos::new(2, 3)));
        
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn cols_iter_returns_all_cols() {
        const ROW: usize = 5;
        let row = super::PixelsRow::new(
            PixelPos::new(ROW, 0), 
            PixelPos::new(ROW, 4));
        let mut iter = super::ColsIter::new(&row);

        assert_eq!(iter.next().unwrap(), PixelPos::new(ROW, 0));
        assert_eq!(iter.next().unwrap(), PixelPos::new(ROW, 1));
        assert_eq!(iter.next().unwrap(), PixelPos::new(ROW, 2));
        assert_eq!(iter.next().unwrap(), PixelPos::new(ROW, 3));
        assert_eq!(iter.next(), None);
    }
}
