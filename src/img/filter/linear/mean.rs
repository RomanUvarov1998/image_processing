use fltk::enums::ColorDepth;
use crate::my_err::MyError;
use crate::processing::{ProgressProvider, Halted};
use crate::utils::LinesIter;
use super::super::super::*;
use super::super::filter_trait::*;
use super::super::*;
use super::super::FilterBase;


#[derive(Clone)]
pub struct LinearMean {
    size: FilterWindowSize,
    extend_value: ExtendValue,
    name: String
}

impl LinearMean {
    pub fn new(size: FilterWindowSize, extend_value: ExtendValue) -> Self {
        LinearMean { size, extend_value, name: "Линейный фильтр (усредняющий)".to_string() }
    }
}

impl WindowFilter for LinearMean {
    fn process_window(&self, window_buffer: &mut [f64]) -> f64 {
        let sum: f64 = window_buffer.into_iter().map(|v| *v).sum();
        sum / (self.w() * self.h()) as f64
    }

    fn w(&self) -> usize { self.size.width }

    fn h(&self) -> usize { self.size.height }

    fn get_extend_value(&self) -> ExtendValue { self.extend_value }

    fn get_iter(&self) -> FilterIterator {
        FilterIterator {
            width: self.w(),
            height: self.h(),
            cur_pos: PixelPos::new(0, 0),
        }
    }
}

impl Filter for LinearMean {
    fn process(&self, img: &Img, prog_prov: &mut ProgressProvider) -> Result<Img, Halted> {
        process_each_layer(img, self, prog_prov)
    }

    fn get_steps_num(&self, img: &Img) -> usize {
        let row_sums = img.w() + 1;
        let col_sums = img.h() + 1;
        let rows = img.h();
        let layers_count = match img.color_depth() {
            ColorDepth::L8 => img.d(),
            ColorDepth::La8 => img.d() - 1,
            ColorDepth::Rgb8 => img.d(),
            ColorDepth::Rgba8 => img.d() - 1,
        };
        
        layers_count * (row_sums + col_sums + rows)
    }

    fn get_description(&self) -> String { format!("{} {}x{}", &self.name, self.h(), self.w()) }

    fn get_save_name(&self) -> String {
        "LinearMean".to_string()
    }

    fn get_copy(&self) -> FilterBase {
        let copy = self.clone();
        Box::new(copy) as FilterBase
    }
}

impl Default for LinearMean {
    fn default() -> Self {
        LinearMean::new(FilterWindowSize::new(3, 3), ExtendValue::Closest)
    }
}

impl StringFromTo for LinearMean {
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

        Ok(())
    }

    fn params_to_string(&self) -> Option<String> {
        let params_str = format!("{}\n{}", self.size.content_to_string(), self.extend_value.content_to_string());
        Some(params_str)
    }
}

impl ByLayer for LinearMean {
    fn process_layer(&self, layer: &ImgLayer, prog_prov: &mut ProgressProvider) -> Result<ImgLayer, Halted> {
        let mat = match layer.channel() {
            ImgChannel::A => {
                return Ok(layer.clone())
            },
            _ => {
                layer.matrix().clone()
            }
        };
        
        let mut sum_res = mat.extended(
            ExtendValue::Given(0_f64), 
            0, 0, 1, 1);

        // sum along rows
        for row in 0..sum_res.h() {
            let mut row_sum = 0_f64;
            for col in 0..sum_res.w() {
                let pos = PixelPos::new(row, col);
                row_sum += sum_res[pos];
                sum_res[pos] = row_sum;
            }

            prog_prov.complete_action()?;
        }
        
        // sum along cols
        for col in 0..sum_res.w() {
            let mut col_sum = 0_f64;
            for row in 0..sum_res.h() {
                let pos = PixelPos::new(row, col);
                col_sum += sum_res[pos];
                sum_res[pos] = col_sum;
            }

            prog_prov.complete_action()?;
        }

        let win_half = PixelPos::new(self.h() / 2, self.w() / 2);

        // filter
        let mat_sum_ext = sum_res.extended(
            ExtendValue::Closest,
            win_half.row, win_half.col, win_half.row, win_half.col);

        let mut mat_res = mat.clone();

        let one = PixelPos::one();

        let coeff = 1_f64 / (self.w() * self.h()) as f64;
        
        for row in 0..mat_res.h() {
            for col in 0..mat_res.w() {
                let pos = PixelPos::new(row, col);
                let ext_pos = pos + one + win_half;
    
                let sum_top_left        = mat_sum_ext[ext_pos - win_half - one];
                let sum_top_right       = mat_sum_ext[ext_pos - win_half.row_vec() + win_half.col_vec() - one.row_vec()];
                let sum_bottom_left     = mat_sum_ext[ext_pos + win_half.row_vec() - win_half.col_vec() - one.col_vec()];
                let sum_bottom_right    = mat_sum_ext[ext_pos + win_half];
    
                let result = (sum_bottom_right - sum_top_right - sum_bottom_left + sum_top_left) * coeff;
                mat_res[ext_pos - win_half - one] = result;
            }

            prog_prov.complete_action()?;
        }

        // create layer
        Ok(ImgLayer::new(mat_res, layer.channel()))
    }
}