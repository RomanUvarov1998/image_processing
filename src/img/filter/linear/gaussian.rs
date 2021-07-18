use fltk::enums::ColorDepth;
use crate::my_err::MyError;
use crate::processing::{ProgressProvider, Halted};
use crate::utils::LinesIter;
use super::super::super::*;
use super::super::filter_trait::*;
use super::super::*;
use super::super::FilterBase;


#[derive(Clone)]
pub struct LinearGaussian {
    size: FilterWindowSize,
    extend_value: ExtendValue,
    coeffs: Vec<f64>,
    name: String
}

impl LinearGaussian {
    pub fn new(size: FilterWindowSize, extend_value: ExtendValue) -> Self {
        assert_eq!(size.width % 2, 1);
        assert_eq!(size.width, size.height);

        let coeffs = Self::count_coeffs(size);

        LinearGaussian { size, extend_value, coeffs, name: "Линейный фильтр (гауссовский)".to_string() }
    }

    fn count_coeffs(size: FilterWindowSize) -> Vec<f64> {
        let mut coeffs = Vec::<f64>::new();
        coeffs.resize(size.width * size.height, 0_f64);

        let r = size.width / 2;
        let one_over_pi: f64 = 1_f64 / 3.14159265359_f64;
        let one_over_2_r_squared: f64 =  1_f64 / (2_f64 * f64::powi(r as f64, 2));

        for row in 0..size.width {
            for col in 0..size.width {
                coeffs[row * size.width + col] = one_over_pi * one_over_2_r_squared 
                    * f64::exp(
                        -(f64::powi(col as f64, 2) + f64::powi(row as f64, 2)) 
                        * one_over_2_r_squared);
            }   
        }

        let sum: f64 = coeffs.iter().map(|v| *v).sum();

        for c in coeffs.iter_mut() { *c /= sum; }

        coeffs
    }
}

impl Filter for LinearGaussian {
    fn process(&self, img: &Img, prog_prov: &mut ProgressProvider) -> Result<Img, Halted> {
        process_each_layer(img, self, prog_prov)
    }

    fn get_steps_num(&self, img: &Img) -> usize {
        let rows_per_layer = img.h();
        let layers_count = match img.color_depth() {
            ColorDepth::L8 => img.d(),
            ColorDepth::La8 => img.d() - 1,
            ColorDepth::Rgb8 => img.d(),
            ColorDepth::Rgba8 => img.d() - 1,
        };
        
        layers_count * rows_per_layer
    }

    fn get_description(&self) -> String { format!("{} {}x{}", &self.name, self.h(), self.w()) }
    
    fn get_save_name(&self) -> String {
        "LinearGaussian".to_string()
    }

    fn get_copy(&self) -> FilterBase {
        let copy = self.clone();
        Box::new(copy) as FilterBase
    }
}

impl ByLayer for LinearGaussian {
    fn process_layer(
        &self,
        layer: &ImgLayer, 
        prog_prov: &mut ProgressProvider) -> Result<ImgLayer, Halted>
    {
        let result_mat = match layer.channel() {
            ImgChannel::A => layer.matrix().clone(),
            _ => process_with_window(
                layer.matrix(), 
                self, 
                prog_prov)?,
        };

        Ok(ImgLayer::new(result_mat, layer.channel()))
    }
}

impl WindowFilter for LinearGaussian {
    fn process_window(&self, window_buffer: &mut [f64]) -> f64 {
        let mut sum = 0_f64;
        for pos in self.get_iter() {
            sum += window_buffer[pos.row * self.w() + pos.col] * self.coeffs[pos.row * self.w() + pos.col];
        }
        sum
    }

    fn w(&self) -> usize { self.size.width }

    fn h(&self) -> usize { self.size.height }

    fn get_extend_value(&self) -> ExtendValue { self.extend_value }

    fn get_iter(&self) -> FilterIterator {
        FilterIterator {
            width: self.w(),
            height: self.h(),
            cur_pos: PixelPos::default()
        }
    }
}

impl Default for LinearGaussian {
    fn default() -> Self {
        LinearGaussian::new(FilterWindowSize::new(5, 5), ExtendValue::Closest)
    }
}

impl StringFromTo for LinearGaussian {
    fn try_set_from_string(&mut self, string: &str) -> Result<(), MyError> {
        let mut lines_iter = LinesIter::new(string);
        if lines_iter.len() != 2 { return Err(MyError::new("Должно быть 2 строки".to_string())); }

        let size = FilterWindowSize::try_from_string(lines_iter.next_or_empty())?
            .check_size_be_3()?
            .check_w_equals_h()?
            .check_w_h_odd()?;

        let extend_value: ExtendValue = ExtendValue::try_from_string(lines_iter.next_or_empty())?;

        self.size = size;
        self.extend_value = extend_value;
        self.coeffs = Self::count_coeffs(size);

        Ok(())
    }

    fn params_to_string(&self) -> Option<String> {
        let params_str = format!("{}\n{}", self.size.content_to_string(), self.extend_value.content_to_string());
        Some(params_str)
    }
}
