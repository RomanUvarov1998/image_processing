use std::{fmt, ops::{Index, IndexMut}, path::PathBuf, result};
use fltk::{enums::ColorDepth, image, prelude::ImageExt};
use crate::{filter::{filter_trait::{ImgFilter, OneLayerFilter, StringFromTo}}, my_err::MyError, progress_provider::ProgressProvider, utils};
use self::pixel_pos::PixelPos;

pub mod pixel_pos;
pub mod color_ops;
pub mod img_ops;

pub const PIXEL_VALUES_COUNT: usize = 256_usize;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum ImgChannel { L, R, G, B, A }

impl StringFromTo for ImgChannel {
    fn try_from_string(string: &str) -> Result<Self, MyError> where Self: Sized {
        let format_err_msg = "Должна быть одна строка: 'Channel: <Название канала A, R, G, B L>".to_string();
        
        let mut lines = utils::LinesIter::new(string);
        if lines.len() != 1 { return Err(MyError::new(format_err_msg)); }

        let mut words = utils::WordsIter::new(lines.next(), " ");
        if words.len() != 2 { return Err(MyError::new(format_err_msg)); }
        if words.next() != "Channel:" { return Err(MyError::new(format_err_msg)); }
        let channel = match words.next() {
            "A" => ImgChannel::A,
            "R" => ImgChannel::R,
            "G" => ImgChannel::G,
            "B" => ImgChannel::B,
            "L" => ImgChannel::L,
            _ => { return Err(MyError::new(format_err_msg)); }
        };

        Ok(channel)
    }

    fn content_to_string(&self) -> String {
        format!("Channel: {}", self)
    }
}

impl PartialEq for ImgChannel {
    fn eq(&self, other: &Self) -> bool {
        *self as u8 == *other as u8
    }
}

impl fmt::Display for ImgChannel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let channel_str: &str = match self {
            ImgChannel::L => "L",
            ImgChannel::R => "B",
            ImgChannel::G => "G",
            ImgChannel::B => "B",
            ImgChannel::A => "A",
        };

        write!(f, "{}", channel_str)
    }
}


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

    pub fn scalar_transform<Tr: Fn(f64) -> f64>(&self, tr: Tr) -> Self {
        let mut transformed = Self::empty_size_of(self);

        for pos in self.get_pixels_iter() {
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

        for pos in self.get_pixels_iter() {
            let pix = bmp::Pixel::new(self[pos] as u8, self[pos] as u8, self[pos] as u8);
            img_to_save.set_pixel(pos.col as u32, pos.row as u32, pix);
        }

        img_to_save.save(path)?;

        Ok(())
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

#[allow(unused)]
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

    pub fn process_by_layer<F: OneLayerFilter, Cbk: Fn(usize)>(&self, filter: &F, progress_cbk: Cbk) -> Img {
        let mut prog_prov: ProgressProvider<Cbk> = filter.create_progress_provider(&self, progress_cbk);
            
        let mut res_layers = Vec::<ImgLayer>::with_capacity(self.d());

        for layer in self.layers() {
            let res_mat = match layer.channel() {
                ImgChannel::A => {
                    layer.matrix().clone()
                },
                _ => {
                    filter.filter(layer.matrix(), &mut prog_prov)
                },
            };
            res_layers.push(ImgLayer::new(res_mat, layer.channel()));
        }

        Img::new(self.w(), self.h(), res_layers, self.color_depth())
    }

    pub fn process_all_layers<F: ImgFilter, Cbk: Fn(usize)>(&self, filter: &F, progress_cbk: Cbk) -> Img {
        let mut prog_prov: ProgressProvider<Cbk> = filter.create_progress_provider(&self, progress_cbk);

        filter.filter(self, &mut prog_prov)
    }

    pub fn layers<'own>(&'own self) -> &'own Vec<ImgLayer> { &self.layers }
    pub fn layers_mut<'own>(&'own mut self) -> &'own mut Vec<ImgLayer> { &mut self.layers }
    pub fn layer_mut<'own>(&'own mut self, ind: usize) -> &'own mut ImgLayer { &mut self.layers[ind] }
    pub fn layer<'own>(&'own self, ind: usize) -> &'own ImgLayer { &self.layers[ind] }

    pub fn get_pixels_iter(&self) -> PixelsIterator {
        PixelsIterator::for_full_image(self.layer(0).matrix())
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