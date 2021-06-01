
use crate::{img::{Img}, matrix2d::{Matrix2D}, my_err::MyError, pixel_pos::PixelPos, utils};

pub trait Filter : Default + Clone {
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
    Given(f64)
}

impl StringFromTo for ExtendValue {
    fn try_from_string(string: &str) -> Result<Self, MyError> {
        let ext_words: Vec<&str> = utils::line_to_words(string, " ");

        let foemat_err_msg = "После матрицы должны быть указаны граничные условия: 'Ext: near' или 'Ext: 0'".to_string();

        if ext_words.len() != 2 {
            return Err(MyError::new(foemat_err_msg));
        }

        if ext_words[0] != "Ext:" {
            return Err(MyError::new(foemat_err_msg));
        }

        let ext_value = match ext_words[1] {
            "0" => ExtendValue::Given(0_f64),
            "near" => ExtendValue::Closest,
            _ => { return Err(MyError::new(foemat_err_msg)); }
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


#[derive(Clone, Copy)]
pub enum NormalizeOption {
    Normalized,
    NotNormalized
}

impl NormalizeOption {
    fn normalize(&self, values: &mut [f64]) {
        match self {            
            NormalizeOption::Normalized => {
                let mut sum = 0_f64;
        
                for v in values.iter() { sum += v; }
                
                if f64::abs(sum) > f64::EPSILON{
                    for v in values.iter_mut() { *v /= sum; }
                }
            }
            NormalizeOption::NotNormalized => {}
        }
    }
}

impl StringFromTo for NormalizeOption {
    fn try_from_string(string: &str) -> Result<Self, MyError> {
        let ext_words: Vec<&str> = utils::line_to_words(string, " ");

        let format_err_msg = "После граничных условий должно быть указано условие нормализации коэффициентов: 'Normalize: true' или 'Normalize: false'".to_string();

        if ext_words.len() != 2 {
            return Err(MyError::new(format_err_msg));
        }

        if ext_words[0] != "Normalize:" {
            return Err(MyError::new(format_err_msg));
        }

        let norm = match ext_words[1] {
            "true" => NormalizeOption::Normalized,
            "false" => NormalizeOption::NotNormalized,
            _ => { return Err(MyError::new(format_err_msg)); }
        };

        Ok(norm)
    }

    fn content_to_string(&self) -> String {
        match self {
            NormalizeOption::Normalized => "Normalize: true".to_string(),
            NormalizeOption::NotNormalized => "Normalize: false".to_string()
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


fn filter_window<T: Filter + FilterIterable>(mut img: Img, filter: &T, buf_filt_fcn: fn(f: &T, &mut [f64]) -> f64) -> Img {
    assert!(filter.w() > 1);
    assert!(filter.h() > 1);

    let mut pixel_buf = Vec::<f64>::new();
    pixel_buf.resize(filter.w() * filter.h(), 0_f64);

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
            pixel_buf[buf_ind] = img_extended[pix_pos];
        }

        let filter_result: f64 = buf_filt_fcn(filter, &mut pixel_buf[..]);
        
        img[pos_im - fil_half_size] = filter_result;
    }

    img
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

impl FilterIterable for LinearCustom {
    fn get_iterator(&self) -> FilterIterator {
        FilterIterator {
            width: self.w(),
            height: self.h(),
            cur_pos: PixelPos::default()
        }
    }
}

impl FilterBuffered for LinearCustom {
    fn filter_buffer(&self, fragment: &mut [f64]) -> f64 {
        let mut sum: f64 = 0_f64;

        for pos in self.get_iterator() {
            let ind = pos.row * self.width + pos.col;
            sum += fragment[ind] * self.arr[ind];
        }
        
        sum
    }
}

impl Filter for LinearCustom {
    fn filter(&self, img: crate::img::Img) -> crate::img::Img {
        filter_window(img, self, LinearCustom::filter_buffer)
    }

    fn w(&self) -> usize { self.width }

    fn h(&self) -> usize { self.height }

    fn get_extend_value(&self) -> ExtendValue {
        self.extend_value
    }
}

impl StringFromTo for LinearCustom {
    fn try_from_string(string: &str) -> Result<Self, MyError> {
        let mut rows = Vec::<Vec<f64>>::new();

        let lines: Vec<&str> = utils::text_to_lines(string);

        if lines.len() < 3 { return Err(MyError::new("Нужно ввести матрицу и параметры на следующей строке".to_string())); }

        for line in &lines[0..lines.len() - 2] {
            let mut row = Vec::<f64>::new();
            for word in utils::line_to_words(line, ",") {
                if word.is_empty() { continue; }
                match word.trim().parse::<f64>() {
                    Ok(value) => { row.push(value) }
                    Err(_) => {
                        return Err(MyError::new("Некорректный формат чисел".to_string()));
                    }
                }
            }
            match rows.last() {
                Some(last_row) => {
                    if row.len() != last_row.len() {
                        return Err(MyError::new("Некорректная разменость матрицы".to_string()));
                    }
                },
                None => {}
            }
            if row.len() < 2 {
                return Err(MyError::new("Матрица должна иметь размеры > 1".to_string()));
            }
            rows.push(row);
        }

        if rows.len() < 2 {
            return Err(MyError::new("Матрица должна иметь размеры > 1".to_string()));
        }

        let ext_value = ExtendValue::try_from_string(lines[lines.len() - 2])?;

        let normalized_value = NormalizeOption::try_from_string(lines[lines.len() - 1])?;

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
    width: usize,
    height: usize,
    extend_value: ExtendValue
}

impl LinearMean {
    pub fn new(width: usize, height: usize, extend_value: ExtendValue) -> Self {
        LinearMean { width, height, extend_value }
    }
}

impl FilterBuffered for LinearMean {
    fn filter_buffer(&self, fragment: &mut [f64]) -> f64 {
        let sum: f64 = fragment.into_iter().map(|v| *v).sum();
        sum / (self.width * self.height) as f64
    }
}

impl Filter for LinearMean {
    fn filter(&self, img: crate::img::Img) -> crate::img::Img {
        filter_window(img, self, Self::filter_buffer)
    }

    fn w(&self) -> usize { self.width }

    fn h(&self) -> usize { self.height }

    fn get_extend_value(&self) -> ExtendValue { self.extend_value }
}

impl Default for LinearMean {
    fn default() -> Self {
        LinearMean::new(3, 3, ExtendValue::Closest)
    }
}

impl FilterIterable for LinearMean {
    fn get_iterator(&self) -> FilterIterator {
        FilterIterator {
            width: self.width,
            height: self.height,
            cur_pos: PixelPos::new(0, 0),
        }
    }
}

impl StringFromTo for LinearMean {
    fn try_from_string(string: &str) -> Result<Self, MyError> where Self: Sized {
        let formar_err_msg = "Должно быть 3 строки: 
            'rows: <нечетное целое число число больше 2>', 
            'cols: <нечетное целое число число больше 2>', 
            'Ext: near/0'".to_string();
        
        let lines: Vec<&str> = utils::text_to_lines(string);
        if lines.len() != 3 { return Err(MyError::new(formar_err_msg)); }

        let height_words: Vec<&str> = utils::line_to_words(lines[0], " ");
        if height_words.len() != 2 { return Err(MyError::new(formar_err_msg)); }
        if height_words[0] != "rows:" { return Err(MyError::new(formar_err_msg)); }
        let height = match height_words[1].parse::<usize>() {
            Ok(val) => val,
            Err(_) => { return Err(MyError::new(formar_err_msg)); }
        };
        if height % 2 == 0 { return Err(MyError::new(formar_err_msg)); }
        if height < 3 { return Err(MyError::new(formar_err_msg)); }

        let width_words: Vec<&str> = utils::line_to_words(lines[1], " ");
        if width_words.len() != 2 { return Err(MyError::new(formar_err_msg)); }
        if width_words[0] != "cols:" { return Err(MyError::new(formar_err_msg)); }
        let width = match width_words[1].parse::<usize>() {
            Ok(val) => val,
            Err(_) => { return Err(MyError::new(formar_err_msg)); }
        };
        if width % 2 == 0 { return Err(MyError::new(formar_err_msg)); }
        if width < 3 { return Err(MyError::new(formar_err_msg)); }

        let ext_value: ExtendValue = ExtendValue::try_from_string(&lines[2])?;

        Ok(LinearMean::new(width, height, ext_value))
    }

    fn content_to_string(&self) -> String {
        format!("rows: {}\ncols: {}\n{}", self.height, self.width, self.extend_value.content_to_string())
    }
}


#[derive(Clone)]
pub struct LinearGaussian {
    size: usize,
    extend_value: ExtendValue,
    coeffs: Vec<f64>
}

impl LinearGaussian {
    pub fn new(size: usize, extend_value: ExtendValue) -> Self {
        assert_eq!(size % 2, 1);

        let mut coeffs = Vec::<f64>::new();
        coeffs.resize(size * size, 0_f64);
        let r = size / 2;
        let one_over_pi: f64 = 1_f64 / 3.14159265359_f64;
        let one_over_2_r_squared: f64 =  1_f64 / (2_f64 * f64::powi(r as f64, 2));

        for row in 0..size {
            for col in 0..size {
                coeffs[row * size + col] = one_over_pi * one_over_2_r_squared 
                    * f64::exp(
                        -(f64::powi(col as f64, 2) + f64::powi(row as f64, 2)) 
                        * one_over_2_r_squared);
            }   
        }

        let sum: f64 = coeffs.iter().map(|v| *v).sum();

        for c in coeffs.iter_mut() { *c /= sum; }

        println!("sum {}", sum);

        LinearGaussian { size, extend_value, coeffs }
    }
}

impl Filter for LinearGaussian {
    fn filter(&self, img: crate::img::Img) -> crate::img::Img {
        filter_window(img, self, LinearGaussian::filter_buffer)
    }

    fn w(&self) -> usize { self.size }

    fn h(&self) -> usize { self.size }

    fn get_extend_value(&self) -> ExtendValue {
        self.extend_value
    }
}

impl FilterBuffered for LinearGaussian {
    fn filter_buffer(&self, fragment: &mut [f64]) -> f64 {
        let mut sum = 0_f64;
        for pos in self.get_iterator() {
            sum += fragment[pos.row * self.w() + pos.col] * self.coeffs[pos.row * self.w() + pos.col];
        }
        sum
    }
}

impl FilterIterable for LinearGaussian {
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
        LinearGaussian::new(5, ExtendValue::Closest)
    }
}

impl StringFromTo for LinearGaussian {
    fn try_from_string(string: &str) -> Result<Self, MyError> where Self: Sized {
        let formar_err_msg = "Должно быть 2 строки: 'size: <нечетное целое число больше 2>', 'Ext: near/0'".to_string();
        
        let lines: Vec<&str> = utils::text_to_lines(string);
        if lines.len() != 2 { return Err(MyError::new(formar_err_msg)); }

        let size_words: Vec<&str> = utils::line_to_words(lines[0], " ");
        if size_words.len() != 2 { return Err(MyError::new(formar_err_msg)); }
        if size_words[0] != "size:" { return Err(MyError::new(formar_err_msg)); }
        let size = match size_words[1].parse::<usize>() {
            Ok(val) => val,
            Err(_) => { return Err(MyError::new(formar_err_msg)); }
        };
        if size % 2 == 0 { return Err(MyError::new(formar_err_msg)); }
        if size < 3 { return Err(MyError::new(formar_err_msg)); }

        let ext_value: ExtendValue = ExtendValue::try_from_string(&lines[1])?;

        Ok(LinearGaussian::new(size, ext_value))
    }

    fn content_to_string(&self) -> String {
        format!("size: {}\n{}", self.size, self.extend_value.content_to_string())
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
        filter_window(img, self, MedianFilter::filter_buffer)
    }

    fn w(&self) -> usize { self.width }

    fn h(&self) -> usize { self.height }

    fn get_extend_value(&self) -> ExtendValue {
        self.extend_value
    }
}

impl StringFromTo for MedianFilter {
    fn try_from_string(string: &str) -> Result<Self, MyError> {
        let format_err_msg = "Должно быть 3 строки: 
            'rows: <нечетное целое число число больше 2>', 
            'cols: <нечетное целое число число больше 2>', 
            'Ext: near/0'".to_string();

        let lines: Vec<&str> = utils::text_to_lines(string);
        if lines.len() != 3 {
            return Err(MyError::new(format_err_msg));
        }

        let height_words: Vec<&str> = utils::line_to_words(lines[0], " ");
        if height_words.len() != 2 {
            return Err(MyError::new(format_err_msg));
        }
        if height_words[0] != "rows:" { return Err(MyError::new(format_err_msg)); }
        let height = match height_words[1].parse::<usize>() {
            Err(_) => return Err(MyError::new(format_err_msg)),
            Ok(val) => val
        };
        if height < 3 { return Err(MyError::new(format_err_msg)); }

        let width_words: Vec<&str> = utils::line_to_words(lines[1], " ");
        if width_words.len() != 2 {
            return Err(MyError::new(format_err_msg));
        }
        if width_words[0] != "cols:" { return Err(MyError::new(format_err_msg)); }
        let width = match width_words[1].parse::<usize>() {
            Err(_) => return Err(MyError::new(format_err_msg)),
            Ok(val) => val
        };
        if width < 3 { return Err(MyError::new(format_err_msg)); }

        let ext_value = ExtendValue::try_from_string(lines[2])?;

        return Ok(MedianFilter::new(width, height, ext_value));
    }

    fn content_to_string(&self) -> String {
        format!("rows: {}\ncols: {}\n{}", self.height, self.width, self.extend_value.content_to_string())
    }
}

impl Default for MedianFilter {
    fn default() -> Self {
        MedianFilter::new(3, 3, ExtendValue::Closest)
    }
}


#[derive(Clone)]
pub struct HistogramLocalContrast {
    width: usize,
    height: usize,
    ext_value: ExtendValue,
    mean_filter: LinearMean,
    a_values: AValues,
}

#[derive(Clone, Copy)]
pub struct AValues { amin: f64, amax: f64 }
impl AValues {
    pub fn new(amin: f64, amax: f64) -> Self {
        assert!(amin <= amax);
        AValues { amin, amax }
    }
}
impl StringFromTo for AValues {
    fn try_from_string(string: &str) -> Result<Self, MyError> {
        let vals_strings: Vec<&str> = utils::line_to_words(string, ",");

        let format_err_msg = "После нормализации коэффициентов должны быть указаны границы А: 'AMin: <дробное число>, AMax: <дробное число>'".to_string();

        if vals_strings.len() != 2 { return Err(MyError::new(format_err_msg)); }

        let amin_words: Vec<&str> = utils::line_to_words(vals_strings[0], " ");
        if amin_words.len() != 2  { return Err(MyError::new(format_err_msg)); }
        if amin_words[0] != "AMin:"  { return Err(MyError::new(format_err_msg)); }
        let amin_val = match amin_words[1].parse::<f64>() {
            Ok(val) => val,
            Err(_) => { return Err(MyError::new(format_err_msg)); }
        };

        let amax_words: Vec<&str> = utils::line_to_words(vals_strings[1], " ");
        if amax_words.len() != 2  { return Err(MyError::new(format_err_msg)); }
        if amax_words[0] != "AMax:"  { return Err(MyError::new(format_err_msg)); }
        let amax_val = match amax_words[1].parse::<f64>() {
            Ok(val) => val,
            Err(_) => { return Err(MyError::new(format_err_msg)); }
        };

        Ok(AValues { amin: amin_val, amax: amax_val } )
    }

    fn content_to_string(&self) -> String {
        format!("AMin: {}, AMax: {}", self.amin, self.amax)
    }
}

impl HistogramLocalContrast {
    pub fn new(width: usize, height: usize, ext_value: ExtendValue, mean_filter_size: usize, 
        a_values: AValues) -> Self 
    {
        HistogramLocalContrast { 
            width, 
            height, 
            ext_value, 
            mean_filter: LinearMean::new(mean_filter_size, mean_filter_size, ExtendValue::Given(0_f64)),
            a_values
        }
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


impl Filter for HistogramLocalContrast {
    fn filter(&self, img: Img) -> Img {
        let mut pixel_buf = Vec::<f64>::new();
        pixel_buf.resize(self.w() * self.h(), 0_f64);

        let fil_half_size = PixelPos::new(self.h() / 2, self.w() / 2);

        let ext_copy = img.copy_with_extended_borders(ExtendValue::Closest, 
            fil_half_size.row, fil_half_size.col);
        let mut hist_matrix = Matrix2D::empty(
            img.w() + self.w(), img.h() + self.h());

        let mut hist_counts: [u32; 256_usize] = [0; 256_usize];

        for pos_im in img.get_area_iter(fil_half_size, 
            fil_half_size + PixelPos::new(img.h(), img.w())) 
        {
            for pos_w in self.get_iterator() {
                let buf_ind: usize = pos_w.row * self.w() + pos_w.col;
                let pix_pos: PixelPos = pos_im + pos_w - fil_half_size;
                pixel_buf[buf_ind] = ext_copy[pix_pos];
            }

            //count histogram bins            
            for v in &pixel_buf[..] {
                hist_counts[(*v as u8) as usize] += 1;
            }

            //count min and max 
            let mut max_value = hist_counts[0];
            let mut min_value = hist_counts[0];
            for v in &hist_counts[1..] {
                if *v == 0 { continue; }
                if max_value < *v { max_value = *v; }
                if min_value < *v { min_value = *v; }
            }

            let val: f64;
            if min_value == max_value {
                val = 0_f64;
            } else {
                val = (max_value as f64 - min_value as f64) / max_value as f64;
            }
            
            hist_matrix[pos_im] = val;
        }

        let img_filtered_ext = ext_copy.processed_copy(&self.mean_filter);

        let mut c_mat = Matrix2D::empty(img_filtered_ext.w(), img_filtered_ext.h());
        for pos in img_filtered_ext.get_iterator() {
            let mut val = ext_copy[pos] - img_filtered_ext[pos];
            val /= ext_copy[pos] + img_filtered_ext[pos] + f64::EPSILON;
            c_mat[pos] = f64::abs(val)
        }

        for m_pos in hist_matrix.get_area_iter(fil_half_size, 
            PixelPos::new(img.h(), img.w()) + fil_half_size) 
        {
            let mut max_value = hist_matrix[m_pos];
            let mut min_value = hist_matrix[m_pos];

            for w_pos in hist_matrix.get_area_iter(
                m_pos - fil_half_size, 
                m_pos + fil_half_size) 
            {
                let v = hist_matrix[w_pos];
                if f64::abs(v) < f64::EPSILON { continue; }
                if max_value < v { max_value = v; }
                if min_value < v { min_value = v; }
            }

            let mut c_power = (hist_matrix[m_pos] - min_value) 
                / (max_value - min_value + f64::EPSILON);
            
            c_power = self.a_values.amin + (self.a_values.amax - self.a_values.amin) * c_power;
            
            c_mat[m_pos] = c_mat[m_pos].powf(c_power);
        }
        
        let mut img_result = Img::empty_with_size(img.w(), img.h());

        for pos in hist_matrix.get_area_iter(fil_half_size, 
            PixelPos::new(img.h(), img.w()) + fil_half_size) 
        {
            let mut val = if ext_copy[pos] > img_filtered_ext[pos] {
                img_filtered_ext[pos] * (1_f64 + c_mat[pos]) / (1_f64 - c_mat[pos])
            } else {
                img_filtered_ext[pos] * (1_f64 - c_mat[pos]) / (1_f64 + c_mat[pos])
            };

            if val < 0_f64 { val = 0_f64; }
            if val > 255_f64 { val = 255_f64; }

            img_result[pos - fil_half_size] = val;
        }

        img_result
    }

    fn w(&self) -> usize { self.width }

    fn h(&self) -> usize { self.height }

    fn get_extend_value(&self) -> ExtendValue {
        self.ext_value
    }
}

impl StringFromTo for HistogramLocalContrast {
    fn try_from_string(string: &str) -> Result<Self, MyError> {
        let format_err_msg = "Должно быть 4 строки: 
        'rows: <нечетное целое число число больше 2>', 
        'cols: <нечетное целое число число больше 2>', 
        'Ext: near/0', 
        'AMin: <дробное число х.хх>, AMax: <дробное число х.хх>'".to_string();

        let lines: Vec<&str> = utils::text_to_lines(string);
        if lines.len() != 4 {
            return Err(MyError::new(format_err_msg));
        }

        let height_words: Vec<&str> = utils::line_to_words(lines[0], " ");
        if height_words.len() != 2 {
            return Err(MyError::new(format_err_msg));
        }
        if height_words[0] != "rows:" { return Err(MyError::new(format_err_msg)); }
        let height = match height_words[1].parse::<usize>() {
            Err(_) => return Err(MyError::new(format_err_msg)),
            Ok(val) => val
        };
        if height < 3 { return Err(MyError::new(format_err_msg)); }

        let width_words: Vec<&str> = utils::line_to_words(lines[1], " ");
        if width_words.len() != 2 {
            return Err(MyError::new(format_err_msg));
        }
        if width_words[0] != "cols:" { return Err(MyError::new(format_err_msg)); }
        let width = match width_words[1].parse::<usize>() {
            Err(_) => return Err(MyError::new(format_err_msg)),
            Ok(val) => val
        };
        if width < 3 { return Err(MyError::new(format_err_msg)); }

        let ext_value = ExtendValue::try_from_string(lines[2])?;

        let a_values = AValues::try_from_string(&lines[3])?;

        return Ok(HistogramLocalContrast::new(width, height, ext_value, 3, a_values ));
    }
    
    fn content_to_string(&self) -> String {
        format!("rows: {}\ncols: {}\n{}\n{}", self.height, self.width, self.ext_value.content_to_string(), self.a_values.content_to_string())
    }
}

impl Default for HistogramLocalContrast {
    fn default() -> Self {
        HistogramLocalContrast::new(3, 3, ExtendValue::Closest, 3, AValues::new(0.5, 0.5))
    }
}