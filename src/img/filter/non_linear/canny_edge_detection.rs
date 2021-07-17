use fltk::enums::ColorDepth;
use crate::my_err::MyError;
use crate::processing::{ProgressProvider, Halted};
use super::super::super::*;
use super::super::filter_trait::*;
use super::super::*;
use super::super::FilterBase;


#[derive(Clone)]
pub struct CannyEdgeDetection {
    name: String,
    gaussian_filter: super::super::LinearGaussian,
    rgb2gray_filter: super::super::Rgb2Gray,
    dx_filter: super::super::LinearCustom,
    dy_filter: super::super::LinearCustom
}

impl CannyEdgeDetection {
    pub fn new() -> Self {
        let g_sz = FilterWindowSize::new(5, 5);
        let g_ext_val = ExtendValue::Closest;

        let dx_filter_coeffs: Vec<f64> = vec![
            -1.0, 0.0, 1.0,
            -2.0, 0.0, 2.0,
            -1.0, 0.0, 1.0,
        ];
        let dy_filter_coeffs: Vec<f64> = vec![
            1.0, 2.0, 1.0,
            0.0, 0.0, 0.0,
            -1.0, -2.0, -1.0,
        ];
        
        CannyEdgeDetection {
            name: "Детектор краев Канни".to_string(),
            gaussian_filter: super::super::LinearGaussian::new(g_sz, g_ext_val),
            rgb2gray_filter: super::super::Rgb2Gray::default(),
            dx_filter: super::super::LinearCustom::with_coeffs(
                dx_filter_coeffs, 
                3, 3, 
                ExtendValue::Closest, 
                NormalizeOption::NotNormalized),
            dy_filter: super::super::LinearCustom::with_coeffs(
                dy_filter_coeffs, 
                3, 3, 
                ExtendValue::Closest, 
                NormalizeOption::NotNormalized)
        }
    }
}

impl Filter for CannyEdgeDetection {
    fn process(&self, img: &Img, prog_prov: &mut ProgressProvider) -> Result<Img, Halted> {
        let grayed: Img;
        let img = match img.color_depth() {
            ColorDepth::L8 | ColorDepth::La8 => img,
            ColorDepth::Rgb8 | ColorDepth::Rgba8 => {
                grayed = self.rgb2gray_filter.process(img, prog_prov)?;
                &grayed
            },
        };

        // bluring
        let layer_blured = {
            let l_layer: &ImgLayer = img.layers()
                .iter()
                .find(|l| l.channel() == ImgChannel::L)
                .as_ref()
                .unwrap();

            self.gaussian_filter.process_layer(l_layer, prog_prov)?
        };

        // derivatives by X and Y
        let dx = self.dx_filter.process_layer(&layer_blured, prog_prov)?;
        let dy = self.dy_filter.process_layer(&layer_blured, prog_prov)?;

        // gradient
        let grad: Matrix2D = {
            let mut mat = Matrix2D::empty_size_of(layer_blured.matrix());
            for pos in mat.get_pixels_iter() {
                mat[pos] = (dx[pos].powi(2) + dy[pos].powi(2)).sqrt();
                prog_prov.complete_action()?;
            }
            let mut g_max = mat[0];
            for val in mat.pixels() {
                if *val > g_max {
                    g_max = *val;
                }
                prog_prov.complete_action()?;
            }
            for pos in mat.get_pixels_iter() {
                mat[pos] = mat[pos] / g_max * 255.0;
                prog_prov.complete_action()?;
            }
            mat
        };

        // angles
        let angles: Matrix2D = {
            let mut mat = Matrix2D::empty_size_of(layer_blured.matrix());
            for pos in mat.get_pixels_iter() {
                // top left: -3pi/4
                // top right: -1pi/4
                // bottom left: 3pi/4
                // bottom right: 1pi/4
                mat[pos] = dy[pos].atan2(dx[pos]);
                prog_prov.complete_action()?;
            }
            mat
        };
        
        // non-max supression
        let mat_non_max_supressed: Matrix2D = {
            let mut mat = Matrix2D::empty_size_of(layer_blured.matrix());
            for pos in mat.get_pixels_area_iter(
                PixelPos::new(1, 1), 
                PixelPos::new(layer_blured.h() - 1, layer_blured.w() - 1)
            ) {
                let angle: f64 = angles[pos];
                const PI_OVER_8: f64 = std::f64::consts::PI / 8.0;
    
                let is_between = |val: f64, min: f64, max_ex: f64| -> bool {
                    min <= val && val < max_ex
                };
    
                let (pos1, pos2) = 
                    if is_between(angle, -7.0 * PI_OVER_8, -5.0 * PI_OVER_8) { // top left
                        (pos.upper_lefter(), pos.downer_righter())
                    } else if is_between(angle, -5.0 * PI_OVER_8, -3.0 * PI_OVER_8) { // top
                        (pos.upper(), pos.downer())
                    } else if is_between(angle, -3.0 * PI_OVER_8, -1.0 * PI_OVER_8) { // top right
                        (pos.upper_righter(), pos.downer_lefter())
                    } else if is_between(angle, -1.0 * PI_OVER_8, 1.0 * PI_OVER_8) { // right
                        (pos.lefter(), pos.righter())
                    } else if is_between(angle, 1.0 * PI_OVER_8, 3.0 * PI_OVER_8) { // down right
                        (pos.downer_righter(), pos.upper_lefter())
                    } else if is_between(angle, 3.0 * PI_OVER_8, 5.0 * PI_OVER_8) { // down
                        (pos.downer(), pos.upper())
                    } else if is_between(angle, 5.0 * PI_OVER_8, 7.0 * PI_OVER_8) { // down left
                        (pos.downer_lefter(), pos.upper_righter())
                    } else { // left
                        (pos.lefter(), pos.righter())
                    };
    
                mat[pos] = 
                    if grad[pos1] > grad[pos] || grad[pos2] > grad[pos] {
                        0.0
                    } else {
                        grad[pos]
                    };
                
                prog_prov.complete_action()?;
            }
            mat
        };

        // double thesholding and hysteresis
        let mat_hysteresis: Matrix2D = {
            let max_pix_value: f64 = {
                let mut max_value: f64 = mat_non_max_supressed.pixels()[0];
                for val in mat_non_max_supressed.pixels().iter() {
                    if *val > max_value {
                        max_value = *val;
                    }
                    prog_prov.complete_action()?;
                }
                max_value
            };
            let high_tr = 0.09 * max_pix_value;
            let low_tr = 0.05 * high_tr;
    
            let mut mat = Matrix2D::empty_size_of(&mat_non_max_supressed);
            for pos in mat.get_pixels_area_iter(
                PixelPos::new(1, 1), 
                PixelPos::new(layer_blured.h() - 1, layer_blured.w() - 1)
            ) {
                let is_non_relevant = |pos: PixelPos| -> bool {
                    mat_non_max_supressed[pos] <= low_tr
                };
                let is_weak = |pos: PixelPos| -> bool {
                    low_tr <= mat_non_max_supressed[pos] && mat_non_max_supressed[pos] <= high_tr
                };
                let has_strong_neigbour = |pos: PixelPos| -> bool {
                    let pixels_around: [f64; 8] = [
                        mat_non_max_supressed[pos.upper()],
                        mat_non_max_supressed[pos.upper_righter()],
                        mat_non_max_supressed[pos.righter()],
                        mat_non_max_supressed[pos.downer_righter()],
                        mat_non_max_supressed[pos.downer()],
                        mat_non_max_supressed[pos.downer_lefter()],
                        mat_non_max_supressed[pos.lefter()],
                        mat_non_max_supressed[pos.upper_lefter()], 
                    ];
                    pixels_around.iter().any(|p| *p >= high_tr)
                };
                mat[pos] = 
                    if is_non_relevant(pos) {
                        0.0
                    } else if is_weak(pos) {
                        if has_strong_neigbour(pos) {
                            255.0
                        } else {
                            0.0
                        }
                    } else { // is_strong
                        255.0
                    };
                
                prog_prov.complete_action()?;
            }

            mat
        };

        // creating result
        let layer_l = ImgLayer::new(mat_hysteresis, ImgChannel::L);
        let layer_a: ImgLayer = {
            let mut layer_a = Matrix2D::empty_size_of(layer_l.matrix());
            for pos in layer_a.get_pixels_iter() {
                layer_a[pos] = 255.0;
            }
            let layer_a = ImgLayer::new(layer_a, ImgChannel::A);
            layer_a
        };
        let img_res = Img::new(img.w(), img.h(), vec![layer_l, layer_a], ColorDepth::La8);

        Ok(img_res)
    }

    fn get_steps_num(&self, img: &Img) -> usize {
        let pixels_count: usize = img.w() * img.h();
        let inner_area_pixels_count: usize = (img.w() - 2) * (img.h() - 2);

        let count = 
            // to make grayed
            match img.color_depth() {
                ColorDepth::L8 | ColorDepth::La8 => 0,
                ColorDepth::Rgb8 | ColorDepth::Rgba8 => self.rgb2gray_filter.get_steps_num(img),
            }

            // to make blured
            + pixels_count
            
            // for dx
            + pixels_count
            
            // for dy
            + pixels_count
            
            // for grad
            + 3 * pixels_count
            
            // for angles
            + pixels_count

            // for non-max supression
            + inner_area_pixels_count

            // for hysteresis
            + pixels_count + inner_area_pixels_count;
        count
    }

    fn get_description(&self) -> String {
        format!("{} {}", self.name, self.gaussian_filter.get_description())
    }

    fn get_save_name(&self) -> String {
        "CannyEdgeDetection".to_string()
    }

    fn get_copy(&self) -> FilterBase {
        Box::new(self.clone()) as FilterBase
    }
}

impl StringFromTo for CannyEdgeDetection {
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

impl Default for CannyEdgeDetection {
    fn default() -> Self {
        CannyEdgeDetection::new()
    }
}