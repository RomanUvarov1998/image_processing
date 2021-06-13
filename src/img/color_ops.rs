use fltk::enums::ColorDepth;

use super::{Matrix2D, Matrix3D};

pub fn rgb_to_gray(img: Matrix3D) -> Matrix3D {
    match img.color_depth {
        ColorDepth::L8 | ColorDepth::La8 => { 
            img 
        },
        ColorDepth::Rgb8 | ColorDepth::Rgba8 => {
            let layers: &Vec<Matrix2D> = img.layers();

            const RGB_2_GRAY_RED: f64 = 0.299;
            const RGB_2_GRAY_GREEN: f64 = 0.587;
            const RGB_2_GRAY_BLUE: f64 = 0.114;

            let mut grayed_layer = Matrix2D::empty_with_size(img.w(), img.h());

            for pos in img.get_iterator() {
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
                    let mut new_layers = Vec::<Matrix2D>::with_capacity(1);
                    new_layers.push(grayed_layer);
                    println!("no a");
                    (new_layers, ColorDepth::L8)
                },
                ColorDepth::Rgba8 => {
                    let mut new_layers = Vec::<Matrix2D>::with_capacity(2);
                    new_layers.push(grayed_layer);
                    new_layers.push(layers[3].clone());
                    println!("has a");
                    (new_layers, ColorDepth::La8)
                },
            };

            Matrix3D { width: img.w(), height: img.h(), layers: new_layers, color_depth }
        },
    }
}