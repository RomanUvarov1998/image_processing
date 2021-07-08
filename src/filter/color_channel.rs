use fltk::enums::ColorDepth;

use crate::{img::{Img, ImgLayer, Matrix2D, PIXEL_VALUES_COUNT}, my_err::MyError, processing::{Halted, ProgressProvider}};
use super::{FilterBase, filter_option::{ImgChannel, Parceable}, filter_trait::{Filter, StringFromTo}, utils::{HistBuf, count_histogram}};


#[derive(Clone)]
pub struct ExtractChannel {
    channel: ImgChannel
}

impl ExtractChannel {
    pub fn new(channel: ImgChannel) -> Self {
        ExtractChannel { channel }
    }
}

impl Filter for ExtractChannel {
    fn filter(&self, img: &Img, prog_prov: &mut ProgressProvider) -> Result<Img, Halted> {
        prog_prov.reset_and_set_total_actions_count(img.d());

        let mut img_res = img.clone();

        for layer in img_res.layers_mut() {
            if layer.channel() != self.channel && layer.channel() != ImgChannel::A { 
                for pos in layer.get_iter() {
                    layer[pos] = 0_f64;
                }
            }

            prog_prov.complete_action()?;
        }
        
        Ok(img_res)
    }

    fn get_description(&self) -> String {
        format!("Выделение канала {}", self.channel)
    }

    fn get_save_name(&self) -> String {
        "ExtractChannel".to_string()
    }
    
    fn get_copy(&self) -> FilterBase {
        let copy = self.clone();
        Box::new(copy) as FilterBase
    }
}

impl StringFromTo for ExtractChannel {
    fn try_set_from_string(&mut self, string: &str) -> Result<(), MyError> {
        let channel = ImgChannel::try_from_string(string)?;

        self.channel = channel;

        Ok(())
    }

    fn params_to_string(&self) -> Option<String> {
        Some(self.channel.content_to_string())
    }
}

impl Default for ExtractChannel {
    fn default() -> Self {
        ExtractChannel::new(ImgChannel::R)
    }
}


#[derive(Clone)]
pub struct NeutralizeChannel {
    channel: ImgChannel
}

impl NeutralizeChannel {
    pub fn new(channel: ImgChannel) -> Self {
        NeutralizeChannel { channel }
    }
}

impl Filter for NeutralizeChannel {
    fn filter(&self, img: &Img, prog_prov: &mut ProgressProvider) -> Result<Img, Halted> {
        prog_prov.reset_and_set_total_actions_count(1);

        let mut img_res = img.clone();

        if let Some(layer) = img_res.layers_mut().into_iter().find(|layer| layer.channel() == self.channel) {
            for pos in layer.get_iter() {
                layer[pos] = 0_f64;
            }
        }

        prog_prov.complete_action()?;
        
        Ok(img_res)
    }

    fn get_description(&self) -> String {
        format!("Подавление канала {}", self.channel)
    }

    fn get_save_name(&self) -> String {
        "NeutralizeChannel".to_string()
    }
    
    fn get_copy(&self) -> FilterBase {
        let copy = self.clone();
        Box::new(copy) as FilterBase
    }
}

impl StringFromTo for NeutralizeChannel {
    fn params_to_string(&self) -> Option<String> {
        Some(self.channel.content_to_string())
    }

    fn try_set_from_string(&mut self, string: &str) -> Result<(), MyError> {
        let channel = ImgChannel::try_from_string(string)?;

        self.channel = channel;

        Ok(())
    }
}

impl Default for NeutralizeChannel {
    fn default() -> Self {
        NeutralizeChannel::new(ImgChannel::R)
    }
}


#[derive(Clone)]
pub struct Rgb2Gray {

}

impl Filter for Rgb2Gray {
    fn filter(&self, img: &Img, prog_prov: &mut ProgressProvider) -> Result<Img, Halted> {
        match img.color_depth() {
            ColorDepth::L8 | ColorDepth::La8 => { 
                prog_prov.reset_and_set_total_actions_count(1);
                let res = img.clone();
                prog_prov.complete_action()?;
                Ok(res)
            },
            ColorDepth::Rgb8 | ColorDepth::Rgba8 => {
                let mut img_res = img.clone();
                let layers = img_res.layers_mut();

                {
                    let pixels_per_layer = img.h() * img.w();
                    let actions_count = pixels_per_layer;
                    prog_prov.reset_and_set_total_actions_count(actions_count);
                }
    
                const RGB_2_GRAY_RED: f64 = 0.299;
                const RGB_2_GRAY_GREEN: f64 = 0.587;
                const RGB_2_GRAY_BLUE: f64 = 0.114;
    
                let mut grayed_layer = Matrix2D::empty_with_size(img.w(), img.h());
    
                for pos in img.get_pixels_iter() {
                    let r = layers[0][pos];
                    let g = layers[1][pos];
                    let b = layers[2][pos];
    
                    grayed_layer[pos] = 
                        r * RGB_2_GRAY_RED
                        + g * RGB_2_GRAY_GREEN
                        + b * RGB_2_GRAY_BLUE;

                    prog_prov.complete_action()?;
                }
    
                let (new_layers, color_depth) = match img.color_depth() {
                    ColorDepth::L8 | ColorDepth::La8 => { unreachable!(""); },
                    ColorDepth::Rgb8 => {
                        let mut new_layers = Vec::<ImgLayer>::with_capacity(1);
                        new_layers.push(ImgLayer::new(grayed_layer, ImgChannel::L));
                        (new_layers, ColorDepth::L8)
                    },
                    ColorDepth::Rgba8 => {
                        let mut new_layers = Vec::<ImgLayer>::with_capacity(2);
                        new_layers.push(ImgLayer::new(grayed_layer, ImgChannel::L));
                        new_layers.push(img.layer(3).clone());
                        (new_layers, ColorDepth::La8)
                    },
                };
    
                Ok(Img::new(img.w(),img.h(), new_layers, color_depth))
            },
        }
    
    }

    fn get_description(&self) -> String {
        "Цветное => ч/б".to_string()
    }

    fn get_save_name(&self) -> String {
        "Rgb2Gray".to_string()
    }
    
    fn get_copy(&self) -> FilterBase {
        let copy = self.clone();
        Box::new(copy) as FilterBase
    }
}

impl StringFromTo for Rgb2Gray {
    fn params_to_string(&self) -> Option<String> {
        None
    }

    fn try_set_from_string(&mut self, string: &str) -> Result<(), MyError> {
        if string.trim().is_empty() {
            Ok(())
        } else {
            Err(MyError::new("У данного фильтра нет настроек".to_string()))
        }
    }
}

impl Default for Rgb2Gray {
    fn default() -> Self {
        Rgb2Gray {}
    }
}


#[derive(Clone)]
pub struct EqualizeHist {

}

impl Filter for EqualizeHist {
    fn filter(&self, img: &Img, prog_prov: &mut ProgressProvider) -> Result<Img, Halted> {
        {
            let pixels_per_layer = img.h() * img.w();
            let layers_count = match img.color_depth() {
                ColorDepth::L8 => img.d(),
                ColorDepth::La8 => img.d() - 1,
                ColorDepth::Rgb8 => img.d(),
                ColorDepth::Rgba8 => img.d() - 1,
            };
            let actions_count = layers_count * (PIXEL_VALUES_COUNT * 2 + pixels_per_layer);
            prog_prov.reset_and_set_total_actions_count(actions_count);
        }

        let mut buffer: HistBuf = [0_f64; PIXEL_VALUES_COUNT];

        let mut img_res = img.clone();
        
        'out: for layer in img_res.layers_mut() {
            if layer.channel() == ImgChannel::A {
                continue 'out;
            }

            // count histogram
            count_histogram(layer.matrix(), &mut buffer);

            // cumulate histogram
            let mut sum = 0_f64;
            for bin in buffer.iter_mut() {
                sum += *bin;
                *bin = sum;

                prog_prov.complete_action()?;
            }

            // equalize
            let max_color_over_max_value = 255_f64 / buffer.last().unwrap();
            for bin in buffer.iter_mut() {
                *bin *= max_color_over_max_value;

                prog_prov.complete_action()?;
            }

            // apply coeff        
            for pos in layer.matrix().get_pixels_iter() {
                let pix_value = layer[pos] as u8 as usize;
                layer[pos] = buffer[pix_value];

                prog_prov.complete_action()?;
            }
        }

        Ok(img_res)
    }

    fn get_description(&self) -> String {
        "Эквализация гистограммы".to_string()
    }

    fn get_save_name(&self) -> String {
        "EqualizeHist".to_string()
    }
    
    fn get_copy(&self) -> FilterBase {
        let copy = self.clone();
        Box::new(copy) as FilterBase
    }
}

impl StringFromTo for EqualizeHist {
    fn params_to_string(&self) -> Option<String> {
        None
    }

    fn try_set_from_string(&mut self, string: &str) -> Result<(), MyError> {
        if string.trim().is_empty() {
            Ok(())
        } else {
            Err(MyError::new("У данного фильтра нет настроек".to_string()))
        }
    }
}

impl Default for EqualizeHist {
    fn default() -> Self {
        EqualizeHist {}
    }
}