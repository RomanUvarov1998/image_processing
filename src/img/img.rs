use super::*;


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
            img.layer_mut(layer_num).matrix_mut()[layer_pixel_num] = all_pixels[pixel_num];
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
            img.layer_mut(layer_num).matrix_mut()[layer_pixel_num] = all_pixels[pixel_num];
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
        assert!(area.bottom_right().col <= self.w());
        assert!(area.bottom_right().row <= self.h());

        let mut img = Img::empty_with_size(
            area.bottom_right().col - area.top_left().col, 
            area.bottom_right().row - area.top_left().row, 
            self.color_depth());
        
        for pos in area.get_pixels_iter() {
            for ch_num in 0..self.d() {
                img.layer_mut(ch_num)[pos - area.top_left()] = self.layer(ch_num)[pos];
            }
        }

        img
    }

    pub fn get_area(&self) -> PixelsArea {
        PixelsArea::with_size(self.h(), self.w())
    }

    pub fn get_drawable_copy(&self) -> image::RgbImage { 
        let mut all_pixels = Vec::<u8>::with_capacity(self.w() * self.h() * self.d());

        let layer_length = self.w() * self.h(); 
        for pix_num in 0..layer_length {
            for layer in self.layers().iter() {
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
                    ext_mat.set_rect(ext_mat.get_area(), 255_f64);
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
                let vals: Vec<u8> = self.layer(0).matrix().pixels()
                    .iter()
                    .map(|p| *p as u8)
                    .collect();

                (vals, ColorType::Luma)
            },
            ColorDepth::Rgb8 | ColorDepth::Rgba8 => {
                let mut vals = Vec::<u8>::with_capacity(self.w() * self.h() * 3);
                
                let r = &self.layer(0).matrix().pixels();
                let g = &self.layer(1).matrix().pixels();
                let b = &self.layer(2).matrix().pixels();

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

