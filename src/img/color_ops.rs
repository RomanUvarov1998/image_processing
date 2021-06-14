use fltk::enums::ColorDepth;
use crate::{filter::utils::{HistBuf, count_histogram}, img::{ImgChannel, ImgLayer}, progress_provider::ProgressProvider};
use super::{Matrix2D, Img, PIXEL_VALUES_COUNT};

pub fn rgb_to_gray(img: &Img) -> Img {
    match img.color_depth {
        ColorDepth::L8 | ColorDepth::La8 => { 
            img.clone()
        },
        ColorDepth::Rgb8 | ColorDepth::Rgba8 => {
            let mut img_res = img.clone();
            let layers = img_res.layers_mut();

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
            }

            let (new_layers, color_depth) = match img.color_depth {
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

            Img { width: img.w(), height: img.h(), layers: new_layers, color_depth }
        },
    }
}

pub fn equalize_histogram<Cbk: Fn(usize)>(img: &Img, progress_cbk: Cbk) -> Img {
    let pixels_per_layer = img.h() * img.w();
    let layers_count = match img.color_depth() {
        ColorDepth::L8 => img.d(),
        ColorDepth::La8 => img.d() - 1,
        ColorDepth::Rgb8 => img.d(),
        ColorDepth::Rgba8 => img.d() - 1,
    };

    let mut prog_prov = ProgressProvider::new(progress_cbk,
        layers_count * (super::PIXEL_VALUES_COUNT * 2 + pixels_per_layer));
    
    prog_prov.start();

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

            prog_prov.complete_action();
        }

        // equalize
        let max_color_over_max_value = 255_f64 / buffer.last().unwrap();
        for bin in buffer.iter_mut() {
            *bin *= max_color_over_max_value;

            prog_prov.complete_action();
        }

        // apply coeff        
        for pos in layer.matrix().get_pixels_iter() {
            let pix_value = layer[pos] as u8 as usize;
            layer[pos] = buffer[pix_value];

            prog_prov.complete_action();
        }
    }

    img_res
}

pub fn neutralize_channel(img: &Img, channel: ImgChannel) -> Img {
    let mut img_res = img.clone();

    if let Some(layer) = img_res.layers_mut().into_iter().find(|layer| layer.channel() == channel) {
        for pos in layer.get_iter() {
            layer[pos] = 0_f64;
        }
    }
    
    img_res
}

pub fn extract_channel(img: &Img, channel: ImgChannel) -> Img {
    let mut img_res = img.clone();

    for layer in img_res.layers_mut() {
        if layer.channel() == channel || layer.channel() == ImgChannel::A { continue; }
        for pos in layer.get_iter() {
            layer[pos] = 0_f64;
        }
    }
    
    img_res
}