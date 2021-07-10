use fltk::enums::ColorDepth;
use crate::{filter::{*, filter_option::*, filter_trait::*}, img::Img, processing::*};


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
