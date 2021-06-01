
use crate::{filter_option::{ExtendValue, FilterWindowSize, NormalizeOption}, filter_trait::{Filter, StringFromTo, WindowFilter}, img::{Img}, matrix2d::{Matrix2D}, my_err::MyError, pixel_pos::PixelPos, utils};

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


fn filter_window<T: WindowFilter>(mut img: Img, filter: &T, buf_filt_fcn: fn(f: &T, &mut [f64]) -> f64) -> Img {
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
    fn filter(&self, img: crate::img::Img) -> crate::img::Img {
        filter_window(img, self, LinearCustom::process_window)
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
    fn filter(&self, img: crate::img::Img) -> crate::img::Img {
        filter_window(img, self, Self::process_window)
    }
}

impl Default for LinearMean {
    fn default() -> Self {
        LinearMean::new(FilterWindowSize::new(3, 3), ExtendValue::Closest)
    }
}

impl StringFromTo for LinearMean {
    fn try_from_string(string: &str) -> Result<Self, MyError> where Self: Sized {
        let lines: Vec<&str> = utils::text_to_lines(string);
        if lines.len() != 2 { return Err(MyError::new("Должно быть 2 строки".to_string())); }

        let size = FilterWindowSize::try_from_string(lines[0])?
            .check_size_be_3()?
            .check_w_equals_h()?
            .check_w_h_odd()?;

        let ext_value: ExtendValue = ExtendValue::try_from_string(&lines[1])?;

        Ok(LinearMean::new(size, ext_value))
    }

    fn content_to_string(&self) -> String {
        format!("{}\n{}", self.size.content_to_string(), self.extend_value.content_to_string())
    }
}


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
    fn filter(&self, img: crate::img::Img) -> crate::img::Img {
        filter_window(img, self, LinearGaussian::process_window)
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
        let lines: Vec<&str> = utils::text_to_lines(string);
        if lines.len() != 2 { return Err(MyError::new("Должно быть 2 строки".to_string())); }

        let size = FilterWindowSize::try_from_string(lines[0])?
            .check_size_be_3()?
            .check_w_equals_h()?
            .check_w_h_odd()?;

        let ext_value: ExtendValue = ExtendValue::try_from_string(&lines[1])?;

        Ok(LinearGaussian::new(size, ext_value))
    }

    fn content_to_string(&self) -> String {
        format!("{}\n{}", self.size.content_to_string(), self.extend_value.content_to_string())
    }
}


#[derive(Clone)]
pub struct MedianFilter {
    size: FilterWindowSize,
    extend_value: ExtendValue
}

impl MedianFilter {
    pub fn new(size: FilterWindowSize, extend_value: ExtendValue) -> Self {        
        MedianFilter { size, extend_value }
    }
}

impl WindowFilter for MedianFilter {
    fn process_window(&self, window_buffer: &mut [f64]) -> f64 {        
        /*
        * Algorithm from N. Wirth's book, implementation by N. Devillard.
        * This code in public domain.
        */
        let mut outer_beg: usize = 0;
        let mut outer_end: usize = window_buffer.len() - 1;
        let mut inner_beg: usize;
        let mut inner_end: usize;
        let med_ind: usize = window_buffer.len() / 2;
        let mut median: f64;
        
        while outer_beg < outer_end {
            median = window_buffer[med_ind];
            inner_beg = outer_beg;
            inner_end = outer_end;
            
            loop {
                while window_buffer[inner_beg] < median { inner_beg += 1; }
                while median < window_buffer[inner_end] { inner_end -= 1; }

                if inner_beg <= inner_end {
                    window_buffer.swap(inner_beg, inner_end);
                    inner_beg += 1; inner_end -= 1;
                }

                if inner_beg > inner_end { break; }
            }

            if inner_end < med_ind { outer_beg = inner_beg; }
            if med_ind < inner_beg { outer_end = inner_end; }
        }

        window_buffer[med_ind]
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

impl Filter for MedianFilter {
    fn filter(&self, img: crate::img::Img) -> crate::img::Img {
        filter_window(img, self, MedianFilter::process_window)
    }
}

impl StringFromTo for MedianFilter {
    fn try_from_string(string: &str) -> Result<Self, MyError> {
        let lines: Vec<&str> = utils::text_to_lines(string);
        if lines.len() != 2 { return Err(MyError::new("Должно быть 2 строки".to_string())); }

        let size = FilterWindowSize::try_from_string(lines[0])?
            .check_size_be_3()?
            .check_w_equals_h()?
            .check_w_h_odd()?;

        let ext_value = ExtendValue::try_from_string(lines[1])?;

        return Ok(MedianFilter::new(size, ext_value));
    }

    fn content_to_string(&self) -> String {
        format!("{}\n{}", self.size.content_to_string(), self.extend_value.content_to_string())
    }
}

impl Default for MedianFilter {
    fn default() -> Self {
        MedianFilter::new(FilterWindowSize::new(3, 3), ExtendValue::Closest)
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
    pub fn new(width: usize, height: usize, ext_value: ExtendValue, a_values: AValues) -> Self 
    {
        HistogramLocalContrast { 
            width, 
            height, 
            ext_value, 
            mean_filter: LinearMean::new(FilterWindowSize::new(3, 3), ExtendValue::Given(0_f64)),
            a_values
        }
    }

    pub fn w(&self) -> usize { self.width }
    pub fn h(&self) -> usize { self.height }
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

        for pos_im in img.get_area_iter(fil_half_size, 
            fil_half_size + PixelPos::new(img.h(), img.w())) 
        {
            for pos_w in self.get_iterator() {
                let buf_ind: usize = pos_w.row * self.w() + pos_w.col;
                let pix_pos: PixelPos = pos_im + pos_w - fil_half_size;
                pixel_buf[buf_ind] = ext_copy[pix_pos];
            }
            
            hist_matrix[pos_im] = self.process_window(&mut pixel_buf[..]);
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
}

impl WindowFilter for HistogramLocalContrast {
    fn process_window(&self, window_buffer: &mut [f64]) -> f64 {
        //count histogram bins            
        let mut hist_counts: [u32; 256_usize] = [0; 256_usize];
        for v in &window_buffer[..] {
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
        
        return if min_value == max_value {
            0_f64
        } else {
            (max_value as f64 - min_value as f64) / max_value as f64
        }
    }
    
    fn w(&self) -> usize { self.width }

    fn h(&self) -> usize { self.height }

    fn get_extend_value(&self) -> ExtendValue {
        self.ext_value
    }

    fn get_iterator(&self) -> FilterIterator {
        FilterIterator {
            width: self.w(),
            height: self.h(),
            cur_pos: PixelPos::default()
        }
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

        return Ok(HistogramLocalContrast::new(width, height, ext_value, a_values));
    }
    
    fn content_to_string(&self) -> String {
        format!("rows: {}\ncols: {}\n{}\n{}", self.height, self.width, self.ext_value.content_to_string(), self.a_values.content_to_string())
    }
}

impl Default for HistogramLocalContrast {
    fn default() -> Self {
        HistogramLocalContrast::new(3, 3, ExtendValue::Closest, AValues::new(0.5, 0.5))
    }
}


#[derive(Clone)]
pub struct CutBrightness {
    br_min: u8,
    br_max: u8,
    replace_with: u8
}

impl CutBrightness {
    pub fn new(br_min: u8, br_max: u8, replace_with: u8) -> Self {
        assert!(br_min < br_max);
        CutBrightness { br_min, br_max, replace_with }
    }
}

impl Filter for CutBrightness {
    fn filter(&self, mut img: crate::img::Img) -> crate::img::Img {
        for pos in img.get_iterator() {
            if img[pos] >= self.br_min as f64 && img[pos] <= self.br_max as f64 {
                img[pos] = self.replace_with as f64
            }
        }
        img
    }
}

impl Default for CutBrightness {
    fn default() -> Self {
        Self::new(100, 200, 0)
    }
}

impl StringFromTo for CutBrightness {
    fn try_from_string(string: &str) -> Result<Self, MyError> where Self: Sized {
        let format_err_msg = "Должно быть 3 строки: 
        'Min: <целое число между 0 и 255 включительно>', 
        'Max: <целое число между 0 и 255 включительно>', 
        'ReplaceWith: <целое число между 0 и 255 включительно>'".to_string();
        
        let lines = utils::text_to_lines(string);
        if lines.len() != 3 { return Err(MyError::new(format_err_msg)); }

        let words_br_min = utils::line_to_words(lines[0], " ");
        if words_br_min.len() != 2 { return Err(MyError::new(format_err_msg)); }
        if words_br_min[0] != "Min:" { return Err(MyError::new(format_err_msg)); }
        let br_min = match words_br_min[1].parse::<u8>() {
            Ok(val) => val,
            Err(_) => { return Err(MyError::new(format_err_msg)); }
        };
        
        let words_br_max = utils::line_to_words(lines[1], " ");
        if words_br_max.len() != 2 { return Err(MyError::new(format_err_msg)); }
        if words_br_max[0] != "Max:" { return Err(MyError::new(format_err_msg)); }
        let br_max = match words_br_max[1].parse::<u8>() {
            Ok(val) => val,
            Err(_) => { return Err(MyError::new(format_err_msg)); }
        };

        if br_min > br_max { return Err(MyError::new(format_err_msg)); }
        
        let words_replace_with = utils::line_to_words(lines[2], " ");
        if words_replace_with.len() != 2 { return Err(MyError::new(format_err_msg)); }
        if words_replace_with[0] != "ReplaceWith:" { return Err(MyError::new(format_err_msg)); }
        let replace_with = match words_replace_with[1].parse::<u8>() {
            Ok(val) => val,
            Err(_) => { return Err(MyError::new(format_err_msg)); }
        };

        Ok(CutBrightness::new(br_min, br_max, replace_with))
    }

    fn content_to_string(&self) -> String {
        format!("Min: {}\nMax: {}\nReplaceWith: {}", self.br_min, self.br_max, self.replace_with)
    }
}