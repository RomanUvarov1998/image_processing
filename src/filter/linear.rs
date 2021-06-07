use crate::{img::pixel_pos::PixelPos, my_err::MyError, utils::{LinesIter, WordsIter}};
use super::{FilterIterator, filter_option::{ExtendValue, FilterWindowSize, NormalizeOption}, filter_trait::{Filter, StringFromTo, WindowFilter}};

#[derive(Clone)]
pub struct LinearGaussian {
    size: FilterWindowSize,
    extend_value: ExtendValue,
    coeffs: Vec<f64>
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

        LinearGaussian { size, extend_value, coeffs }
    }
}

impl Filter for LinearGaussian {
    fn filter<Cbk: Fn(usize)>(&self, img: crate::img::Matrix2D, progress_cbk: Cbk) -> crate::img::Matrix2D {
        super::filter_window(img, self, LinearGaussian::process_window, progress_cbk)
    }
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


#[derive(Clone)]
pub struct LinearCustom {
    width: usize,
    height: usize,
    extend_value: ExtendValue,
    arr: Vec<f64>,
    normalized: NormalizeOption
}

impl LinearCustom {
    pub fn with_coeffs(mut coeffs: Vec<f64>, width: usize, height: usize, extend_value: ExtendValue, normalized: NormalizeOption) -> Self {
        assert!(width > 0);
        assert!(height > 0);
        assert!(coeffs.len() > 0);

        normalized.normalize(&mut coeffs[..]);

        LinearCustom { width, height, arr: coeffs, extend_value, normalized }
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
    fn filter<Cbk: Fn(usize)>(&self, img: crate::img::Matrix2D, progress_cbk: Cbk) -> crate::img::Matrix2D {
        super::filter_window(img, self, LinearCustom::process_window, progress_cbk)
    }
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


#[derive(Clone)]
pub struct LinearMean {
    size: FilterWindowSize,
    extend_value: ExtendValue
}

impl LinearMean {
    pub fn new(size: FilterWindowSize, extend_value: ExtendValue) -> Self {
        LinearMean { size, extend_value }
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
    fn filter<Cbk: Fn(usize)>(&self, img: crate::img::Matrix2D, progress_cbk: Cbk) -> crate::img::Matrix2D {
        super::filter_window(img, self, Self::process_window, progress_cbk)
    }
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
