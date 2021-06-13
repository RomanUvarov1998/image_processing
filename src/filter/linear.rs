use crate::{img::{Img, ImgChannel, pixel_pos::PixelPos}, my_err::MyError, proc_steps::StepAction, progress_provider::ProgressProvider, utils::{LinesIter, WordsIter}};
use super::{FilterIterator, filter_option::{ExtendValue, FilterWindowSize, NormalizeOption}, filter_trait::{Filter, StringFromTo, WindowFilter}};


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

        LinearGaussian { size, extend_value, coeffs, name: "Линейный фильтр (гауссовский)".to_string() }
    }
}

impl Filter for LinearGaussian {
    fn filter<Cbk: Fn(usize)>(&self, img: &Img, progress_cbk: Cbk) -> Img {
        let mut prog_prov = ProgressProvider::new(
            progress_cbk, 
            img.layers().len() * (img.h() * img.w()));

        let mut result_img = img.clone();

        'out: for layer_num in 0..img.d() {
            let layer = img.layer(layer_num);

            if layer.channel() == ImgChannel::A {
                continue 'out;
            }

            let res_layer = result_img.layer_mut(layer_num);
            super::filter_window(layer, res_layer, self, LinearGaussian::process_window, &mut prog_prov);
        }

        result_img
    }

    fn get_description(&self) -> String { format!("{} {}x{}", &self.name, self.h(), self.w()) }
}

impl WindowFilter for LinearGaussian {
    fn process_window(&self, window_buffer: &mut [f64]) -> f64 {
        let mut sum = 0_f64;
        for pos in self.get_iterator() {
            sum += window_buffer[pos.row * self.w() + pos.col] * self.coeffs[pos.row * self.w() + pos.col];
        }
        sum
    }

    fn w(&self) -> usize { self.size.width }

    fn h(&self) -> usize { self.size.height }

    fn get_extend_value(&self) -> ExtendValue {
        self.extend_value
    }

    fn get_iterator(&self) -> FilterIterator {
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
    fn try_from_string(string: &str) -> Result<Self, MyError> where Self: Sized {
        let mut lines_iter = LinesIter::new(string);
        if lines_iter.len() != 2 { return Err(MyError::new("Должно быть 2 строки".to_string())); }

        let size = FilterWindowSize::try_from_string(lines_iter.next())?
            .check_size_be_3()?
            .check_w_equals_h()?
            .check_w_h_odd()?;

        let ext_value: ExtendValue = ExtendValue::try_from_string(lines_iter.next())?;

        Ok(LinearGaussian::new(size, ext_value))
    }

    fn content_to_string(&self) -> String {
        format!("{}\n{}", self.size.content_to_string(), self.extend_value.content_to_string())
    }
}

impl Into<StepAction> for LinearGaussian {
    fn into(self) -> StepAction {
        StepAction::LinearGaussian(self)
    }
}


#[derive(Clone)]
pub struct LinearCustom {
    width: usize,
    height: usize,
    extend_value: ExtendValue,
    arr: Vec<f64>,
    normalized: NormalizeOption,
    name: String
}

impl LinearCustom {
    pub fn with_coeffs(mut coeffs: Vec<f64>, width: usize, height: usize, extend_value: ExtendValue, normalized: NormalizeOption) -> Self {
        assert!(width > 0);
        assert!(height > 0);
        assert!(coeffs.len() > 0);

        normalized.normalize(&mut coeffs[..]);

        LinearCustom { width, height, arr: coeffs, extend_value, normalized, name: "Линейный фильтр".to_string() }
    }
}

impl WindowFilter for LinearCustom {
    fn process_window(&self, window_buffer: &mut [f64]) -> f64 {
        let mut sum: f64 = 0_f64;

        for pos in self.get_iterator() {
            let ind = pos.row * self.width + pos.col;
            sum += window_buffer[ind] * self.arr[ind];
        }
        
        sum
    }

    fn w(&self) -> usize { self.width }

    fn h(&self) -> usize { self.height }

    fn get_extend_value(&self) -> ExtendValue {
        self.extend_value
    }

    fn get_iterator(&self) -> FilterIterator {
        FilterIterator {
            width: self.w(),
            height: self.h(),
            cur_pos: PixelPos::default()
        }
    }
}

impl Filter for LinearCustom {
    fn filter<Cbk: Fn(usize)>(&self, img: &Img, progress_cbk: Cbk) -> Img {
        let mut prog_prov = ProgressProvider::new(
            progress_cbk, 
            img.layers().len() * (img.h() * img.w()));

        let mut result_img = img.clone();

        'out: for layer_num in 0..img.d() {
            let layer = img.layer(layer_num);

            if layer.channel() == ImgChannel::A {
                continue 'out;
            }

            let res_layer = result_img.layer_mut(layer_num);
            super::filter_window(layer, res_layer, self, LinearCustom::process_window, &mut prog_prov)
        }

        result_img
    }

    fn get_description(&self) -> String { format!("{} {}x{}", &self.name, self.h(), self.w()) }
}

impl StringFromTo for LinearCustom {
    fn try_from_string(string: &str) -> Result<Self, MyError> {
        let mut rows = Vec::<Vec<f64>>::new();

        let mut lines_iter = LinesIter::new(string);

        if lines_iter.len() < 3 { return Err(MyError::new("Нужно ввести матрицу и параметры на следующей строке".to_string())); }

        for _ in 0..lines_iter.len() - 2 {
            let mut row = Vec::<f64>::new();
            let mut words_iter = WordsIter::new(lines_iter.next(), ",");
            loop {
                match words_iter.next() {
                    "" => break,
                    word => match word.parse::<f64>() {
                        Ok(value) => { row.push(value) }
                        Err(_) => { return Err(MyError::new("Некорректный формат чисел".to_string())); }
                    },
                }
            }
            match rows.last() {
                Some(last_row) => if row.len() != last_row.len() { return Err(MyError::new("Некорректная разменость матрицы".to_string())); },
                None => {}
            }
            if row.len() < 2 { return Err(MyError::new("Матрица должна иметь размеры > 1".to_string())); }
            rows.push(row);
        }

        if rows.len() < 2 {
            return Err(MyError::new("Матрица должна иметь размеры > 1".to_string()));
        }

        let ext_value = ExtendValue::try_from_string(lines_iter.next())?;

        let normalized_value = NormalizeOption::try_from_string(lines_iter.next())?;

        let mut coeffs = Vec::<f64>::new();
        for mut row in rows.clone() {
            coeffs.append(&mut row);
        }
        let width = rows.last().unwrap().len();
        let height = rows.len();

        Ok(LinearCustom::with_coeffs(coeffs, width, height, ext_value, normalized_value))
    }

    fn content_to_string(&self) -> String {
        let mut fil_string = String::new();

        for row in 0..self.height {
            for col in 0..self.width {
                fil_string.push_str(&self.arr[row * self.width + col].to_string());
                if col < self.width - 1 {
                    fil_string.push_str(", ");
                }
            }
            if row < self.height - 1 {
                fil_string.push_str("\n");
            }
        }

        fil_string.push_str(&format!("\n{}", self.extend_value.content_to_string()));

        fil_string.push_str(&format!("\n{}", self.normalized.content_to_string()));

        fil_string
    }
}

impl Default for LinearCustom {
    fn default() -> Self {
        let coeffs = vec![1_f64];
        LinearCustom::with_coeffs(coeffs, 1, 1, ExtendValue::Closest, NormalizeOption::Normalized)
    }
}

impl Into<StepAction> for LinearCustom {
    fn into(self) -> StepAction {
        StepAction::LinearCustom(self)
    }
}


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

    fn get_iterator(&self) -> FilterIterator {
        FilterIterator {
            width: self.w(),
            height: self.h(),
            cur_pos: PixelPos::new(0, 0),
        }
    }
}

impl Filter for LinearMean {
    fn filter<Cbk: Fn(usize)>(&self, img: &Img, progress_cbk: Cbk) -> Img {
        let mut prog_prov = ProgressProvider::new(
            progress_cbk, 
            img.layers().len() * (img.h() + img.w() + img.h() * img.w()));

        let mut result_img = img.clone();

        'out: for layer_num in 0..img.d() {
            let layer = img.layer(layer_num);

            if layer.channel() == ImgChannel::A {
                continue 'out;
            }

            let res_layer = result_img.layer_mut(layer_num);

            // sum along rows
            for row in 0..img.h() {
                let mut row_sum = 0_f64;
                for col in 0..img.w() {
                    let pos = PixelPos::new(row, col);
                    row_sum += res_layer[pos];
                    res_layer[pos] = row_sum;
                }

                prog_prov.complete_action();
            }
            
            // sum along cols
            for col in 0..img.w() {
                let mut col_sum = 0_f64;
                for row in 0..img.h() {
                    let pos = PixelPos::new(row, col);
                    col_sum += res_layer[pos];
                    res_layer[pos] = col_sum;
                }

                prog_prov.complete_action();
            }
            
            // filter
            let layer_ext = res_layer.matrix().copy_with_extended_borders(
                ExtendValue::Closest, 
                self.h() / 2 + 1, self.w() / 2 + 1);
            let one = PixelPos::new(1, 1);
            let win_half = PixelPos::new(self.h() / 2, self.w() / 2);

            let left_top = win_half + one;
            let right_bottom = left_top + img.size_vec();
            let coeff = 1_f64 / (self.w() * self.h()) as f64;
            
            for ext_pos in layer_ext.get_area_iter(left_top, right_bottom) {
                let sum_top_left        = layer_ext[ext_pos - win_half - one];
                let sum_top_right       = layer_ext[ext_pos - win_half.row_vec() + win_half.col_vec() - one.row_vec()];
                let sum_bottom_left     = layer_ext[ext_pos + win_half.row_vec() - win_half.col_vec() - one.col_vec()];
                let sum_bottom_right    = layer_ext[ext_pos + win_half];

                let result = (sum_bottom_right - sum_top_right - sum_bottom_left + sum_top_left) * coeff;
                res_layer[ext_pos - win_half - one] = result;

                prog_prov.complete_action();
            }
        }

        result_img.clone()
    }

    fn get_description(&self) -> String { format!("{} {}x{}", &self.name, self.h(), self.w()) }
}

impl Default for LinearMean {
    fn default() -> Self {
        LinearMean::new(FilterWindowSize::new(3, 3), ExtendValue::Closest)
    }
}

impl StringFromTo for LinearMean {
fn try_from_string(string: &str) -> Result<Self, MyError> where Self: Sized {
    let mut lines_iter = LinesIter::new(string);
    if lines_iter.len() != 2 { return Err(MyError::new("Должно быть 2 строки".to_string())); }

    let size = FilterWindowSize::try_from_string(lines_iter.next())?
        .check_size_be_3()?
        .check_w_equals_h()?
        .check_w_h_odd()?;

    let ext_value: ExtendValue = ExtendValue::try_from_string(lines_iter.next())?;

    Ok(LinearMean::new(size, ext_value))
}

fn content_to_string(&self) -> String {
    format!("{}\n{}", self.size.content_to_string(), self.extend_value.content_to_string())
}
}

impl Into<StepAction> for LinearMean {
    fn into(self) -> StepAction {
        StepAction::LinearMean(self)
    }
}