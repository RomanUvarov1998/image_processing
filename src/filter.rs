use crate::{img::{Img}, my_err::MyError, pixel_pos::PixelPos};

pub const MAX_WINDOW_SIZE: usize = 11;
const MAX_WINDOW_BUFFER_SIZE: usize = MAX_WINDOW_SIZE * MAX_WINDOW_SIZE;

pub trait Filter : FilterBuffered + Default + Clone {
    fn filter(&self, img: crate::img::Img) -> crate::img::Img;
    fn w(&self) -> usize;
    fn h(&self) -> usize;
    fn get_extend_value(&self) -> ExtendValue;
}

pub trait FilterIterable {
    fn get_iterator(&self) -> FilterIterator;
}

pub trait FilterBuffered {
    fn filter_buffer(&self, fragment: &mut [f64]) -> f64;
}

pub trait StringFromTo {
    fn try_from_string(string: &str) -> Result<Self, MyError> where Self: Sized;
    fn content_to_string(&self) -> String;
}


#[derive(Clone, Copy)]
pub enum ExtendValue {
    Closest,
    Given(u8)
}

impl StringFromTo for ExtendValue {
    fn try_from_string(string: &str) -> Result<Self, MyError> {
        let ext_words: Vec<&str> = string.split(" ").into_iter().collect();

        if ext_words.len() != 2 {
            return Err(MyError::new("После матрицы должен быть указаны граничные условия: 'Ext: near' или 'Ext: 0'".to_string()));
        }

        if ext_words[0] != "Ext:" {
            return Err(MyError::new("После матрицы должен быть указаны граничные условия: 'Ext: near' или 'Ext: 0'".to_string()));
        }

        let ext_value = match ext_words[1] {
            "0" => ExtendValue::Given(0),
            "near" => ExtendValue::Closest,
            _ => { return Err(MyError::new("После матрицы должен быть указаны граничные условия: 'Ext: near' или 'Ext: 0'".to_string())); }
        };

        Ok(ext_value)
    }

    fn content_to_string(&self) -> String {
        match self {
            ExtendValue::Closest => "Ext: near".to_string(),
            ExtendValue::Given(val) => format!("Ext: {}", val)
        }        
    }
}

pub struct FilterIterator {
    width: usize,
    height: usize,
    cur_pos: PixelPos
}

impl FilterIterator {
    pub fn fits(&self, pos: PixelPos) -> bool {
        pos.col < self.width && pos.row < self.height
    }
}

impl Iterator for FilterIterator {
    type Item = PixelPos;

    fn next(&mut self) -> Option<PixelPos> {
        let curr = self.cur_pos;

        assert!(self.fits(self.cur_pos));

        if self.cur_pos.col < self.width - 1 {
            self.cur_pos.col += 1;
            return Some(curr);
        } else if self.cur_pos.row < self.height - 1 {
            self.cur_pos.col = 0;
            self.cur_pos.row += 1;
            return Some(curr);
        } else {
            self.cur_pos = PixelPos::default();
            return None;
        }        
    }
}

fn filter_border_0<T: Filter + FilterIterable>(mut img: Img, filter: &T, buf_filt_fcn: fn(f: &T, &mut [f64]) -> f64) -> Img {
    let pixel_buf_actual_size = filter.w() * filter.h();

    assert!(pixel_buf_actual_size < MAX_WINDOW_BUFFER_SIZE, 
        "filter size must be <= {}", MAX_WINDOW_SIZE);

    let mut pixel_buf = [0_f64; MAX_WINDOW_BUFFER_SIZE];

    let fil_half_size = PixelPos::new(filter.h() / 2, filter.w() / 2);

    let img_extended = img.copy_with_extended_borders(
        filter.get_extend_value(), 
        fil_half_size.row, 
        fil_half_size.col);

    for pos_im in img_extended.get_area_iter(
        fil_half_size, 
        PixelPos::new(img.h(), img.w()) + fil_half_size)
    {
        for pos_w in filter.get_iterator() {            
            let buf_ind: usize = pos_w.row * filter.w() + pos_w.col;
            let pix_pos: PixelPos = pos_im + pos_w - fil_half_size;
            pixel_buf[buf_ind] = img_extended.pixel_at(pix_pos) as f64;
        }

        let filter_result: f64 = buf_filt_fcn(filter, &mut pixel_buf[0..pixel_buf_actual_size]);
        img.set_pixel(pos_im - fil_half_size, filter_result as u8);
    }

    img
}


#[derive(Clone)]
pub struct LinearFilter {
    width: usize,
    height: usize,
    extend_value: ExtendValue,
    arr: Vec<f64>,
}

impl LinearFilter {
    pub fn with_coeffs(coeffs: Vec<f64>, width: usize, height: usize, extend_value: ExtendValue) -> Self {
        assert!(width > 0);
        assert!(height > 0);
        assert!(coeffs.len() > 0);
        LinearFilter { width, height, arr: coeffs, extend_value }
    }
        
    pub fn mean_filter_of_size(size: usize, extend_value: ExtendValue) -> Self {
        assert_eq!(size % 2, 1);

        let mut arr = Vec::<f64>::new();
        let coeff = 1_f64 / ((size * size) as f64);
        arr.resize(size * size, coeff);
        LinearFilter { width: size, height: size, arr, extend_value }
    }
}

impl FilterIterable for LinearFilter {
    fn get_iterator(&self) -> FilterIterator {
        FilterIterator {
            width: self.w(),
            height: self.h(),
            cur_pos: PixelPos::default()
        }
    }
}

impl FilterBuffered for LinearFilter {
    fn filter_buffer(&self, fragment: &mut [f64]) -> f64 {
        let mut sum: f64 = 0_f64;

        for pos in self.get_iterator() {
            let ind = pos.row * self.width + pos.col;
            sum += fragment[ind] * self.arr[ind];
        }
        
        sum
    }
}

impl Filter for LinearFilter {
    fn filter(&self, img: crate::img::Img) -> crate::img::Img {
        filter_border_0(img, self, LinearFilter::filter_buffer)
    }

    fn w(&self) -> usize { self.width }

    fn h(&self) -> usize { self.height }

    fn get_extend_value(&self) -> ExtendValue {
        self.extend_value
    }
}

impl StringFromTo for LinearFilter {
    fn try_from_string(string: &str) -> Result<Self, MyError> {
        let mut rows = Vec::<Vec<f64>>::new();

        let lines: Vec<&str> = string.split('\n')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .into_iter()
            .collect();

        for line in &lines[0..lines.len() - 1] {
            let mut row = Vec::<f64>::new();
            for word in line.trim().split(',').map(|w| w.trim()) {
                if word.is_empty() { continue; }
                match word.trim().parse::<f64>() {
                    Ok(value) => { row.push(value) }
                    Err(_) => {
                        return Err(MyError::new("Некорректный формат чисел".to_string()));
                    }
                }
            }
            if rows.len() > 0 && row.len() != rows.last().unwrap().len() {
                return Err(MyError::new("Некорректная разменость матрицы".to_string()));
            }
            if row.len() == 0 {
                return Err(MyError::new("Некорректная разменость матрицы".to_string()));
            }
            rows.push(row);
        }

        if rows.len() == 0 {
            return Err(MyError::new("Матрица должна иметь ненулевой размер".to_string()));
        }

        let ext_value = ExtendValue::try_from_string(lines.last().unwrap())?;

        let mut coeffs = Vec::<f64>::new();
        for mut row in rows.clone() {
            coeffs.append(&mut row);
        }
        let width = rows.last().unwrap().len();
        let height = rows.len();

        Ok(LinearFilter::with_coeffs(coeffs, width, height, ext_value))
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

        fil_string
    }
}

impl Default for LinearFilter {
    fn default() -> Self {
        LinearFilter::mean_filter_of_size(3, ExtendValue::Closest)
    }
}


#[derive(Clone)]
pub struct MedianFilter {
    width: usize,
    height: usize,
    extend_value: ExtendValue
}

impl MedianFilter {
    pub fn new(width: usize, height: usize, extend_value: ExtendValue) -> Self {
        MedianFilter { width, height, extend_value }
    }
}

impl FilterIterable for MedianFilter {
    fn get_iterator(&self) -> FilterIterator {
        FilterIterator {
            width: self.w(),
            height: self.h(),
            cur_pos: PixelPos::default()
        }
    }
}

impl FilterBuffered for MedianFilter {
    fn filter_buffer(&self, fragment: &mut [f64]) -> f64 {        
        /*
        * Algorithm from N. Wirth's book, implementation by N. Devillard.
        * This code in public domain.
        */
        let mut outer_beg: usize = 0;
        let mut outer_end: usize = fragment.len() - 1;
        let mut inner_beg: usize;
        let mut inner_end: usize;
        let med_ind: usize = fragment.len() / 2;
        let mut median: f64;
        
        while outer_beg < outer_end {
            median = fragment[med_ind];
            inner_beg = outer_beg;
            inner_end = outer_end;
            
            loop {
                while fragment[inner_beg] < median { inner_beg += 1; }
                while median < fragment[inner_end] { inner_end -= 1; }

                if inner_beg <= inner_end {
                    fragment.swap(inner_beg, inner_end);
                    inner_beg += 1; inner_end -= 1;
                }

                if inner_beg > inner_end { break; }
            }

            if inner_end < med_ind { outer_beg = inner_beg; }
            if med_ind < inner_beg { outer_end = inner_end; }
        }

        fragment[med_ind]
    }
}

impl Filter for MedianFilter {
    fn filter(&self, img: crate::img::Img) -> crate::img::Img {
        filter_border_0(img, self, MedianFilter::filter_buffer)
    }

    fn w(&self) -> usize { self.width }

    fn h(&self) -> usize { self.height }
    fn get_extend_value(&self) -> ExtendValue {
        todo!()
    }
}

impl StringFromTo for MedianFilter {
    fn try_from_string(string: &str) -> Result<Self, MyError> {
        let lines: Vec<&str> = string.split('\n').into_iter().collect();
        if lines.len() != 1 {
            return Err(MyError::new("Должна быть 1 строка. Формат (кол-во строк, кол-во столбцов): 'X, X'.".to_string()));
        }

        let format_err_msg = "Формат (кол-во строк (число), кол-во столбцов (число), граничные условия): 'X, X, Ext: near/0'.".to_string();

        let words: Vec<&str> = lines[0].split(',').map(|w| w.trim() ).filter(|w| !w.is_empty() ).into_iter().collect();
        if words.len() != 3 {
            return Err(MyError::new(format_err_msg));
        }

        let height = match words[0].parse::<usize>() {
            Err(_) => return Err(MyError::new(format_err_msg)),
            Ok(val) => val
        };

        let width = match words[1].parse::<usize>() {
            Err(_) => return Err(MyError::new(format_err_msg)),
            Ok(val) => val
        };

        let ext_value = ExtendValue::try_from_string(words[2])?;

        return Ok(MedianFilter::new(width, height, ext_value));
    }

    fn content_to_string(&self) -> String {
        format!("{}, {}, {}", self.height, self.width, self.extend_value.content_to_string())
    }
}

impl Default for MedianFilter {
    fn default() -> Self {
        MedianFilter::new(3, 3, ExtendValue::Closest)
    }
}

/*
#[derive(Clone)]
pub struct HistogramLocalContrast {
    width: usize,
    height: usize
}

impl HistogramLocalContrast {
    pub fn new(width: usize, height: usize) -> Self {
        HistogramLocalContrast { width, height }
    }

    pub fn w(&self) -> usize { self.width }
    pub fn h(&self) -> usize { self.height }
}

impl FilterIterable for HistogramLocalContrast {
    fn get_iterator(&self) -> FilterIterator {
        FilterIterator {
            width: self.w(),
            height: self.h(),
            cur_pos: PixelPos::default()
        }
    }
}

impl FilterBuffered for HistogramLocalContrast {
    fn filter_buffer(&self, fragment: &[f64]) -> f64 {
        
    }
}

impl Filter for HistogramLocalContrast {
    fn filter(&self, fragment: &mut [f64]) -> f64 {
        let n: usize = 15_usize;
        let m: usize = 3_usize;
        let amin: f64 = 0.2_f64;
        let amax: f64 = 0.7_f64;

        let n1: usize = n / 2;
        let m1: usize = m / 2;

        // краевой эффект
        //...

        0.0_f64
    }

    fn w(&self) -> usize { self.width }

    fn h(&self) -> usize { self.height }

    fn try_from_string(string: &str) -> Result<Self, MyError> {
        let lines: Vec<&str> = string.split('\n').into_iter().collect();
        if lines.len() != 1 {
            return Err(MyError::new("Должна быть 1 строка. Формат (кол-во строк, кол-во столбцов): 'X, X'.".to_string()));
        }

        let words: Vec<&str> = lines[0].split(',').map(|w| w.trim() ).filter(|w| !w.is_empty() ).into_iter().collect();
        if words.len() != 2 {
            return Err(MyError::new("Должно быть 2 числа. Формат (кол-во строк, кол-во столбцов): 'X, X'.".to_string()));
        }

        let height = match words[0].parse::<usize>() {
            Err(_) => return Err(MyError::new("Кол-во строк должно быть целым числом. Формат (кол-во строк, кол-во столбцов): 'X, X'.".to_string())),
            Ok(val) => val
        };

        let width = match words[1].parse::<usize>() {
            Err(_) => return Err(MyError::new("Кол-во столбцов должно быть целым числом. Формат (кол-во строк, кол-во столбцов): 'X, X'.".to_string())),
            Ok(val) => val
        };

        return Ok(HistogramLocalContrast::new(width, height));
    }
    fn content_to_string(&self) -> String {
        format!("{}, {}", self.height, self.width)
    }
}

impl Default for HistogramLocalContrast {
    fn default() -> Self {
        HistogramLocalContrast::new(3, 3)
    }
}
*/