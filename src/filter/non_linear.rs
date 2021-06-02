use crate::{img::{Matrix2D}, my_err::MyError, img::pixel_pos::PixelPos, utils::{LinesIter}};
use super::{FilterIterator, filter_option::{ARange, CutBrightnessRange, ExtendValue, FilterWindowSize, ValueRepaceWith}, filter_trait::{Filter, StringFromTo, WindowFilter}, linear::LinearMean};


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
    fn filter(&self, img: crate::img::Matrix2D) -> crate::img::Matrix2D {
        super::filter_window(img, self, MedianFilter::process_window)
    }
}

impl StringFromTo for MedianFilter {
    fn try_from_string(string: &str) -> Result<Self, MyError> {
        let mut lines_iter = LinesIter::new(string);
        if lines_iter.len() != 2 { return Err(MyError::new("Должно быть 2 строки".to_string())); }

        let size = FilterWindowSize::try_from_string(lines_iter.next())?
            .check_size_be_3()?
            .check_w_equals_h()?
            .check_w_h_odd()?;

        let ext_value = ExtendValue::try_from_string(lines_iter.next())?;

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
    size: FilterWindowSize,
    ext_value: ExtendValue,
    mean_filter: LinearMean,
    a_values: ARange,
}

impl HistogramLocalContrast {
    pub fn new(size: FilterWindowSize, ext_value: ExtendValue, a_values: ARange) -> Self 
    {
        HistogramLocalContrast { 
            size, 
            ext_value, 
            mean_filter: LinearMean::new(FilterWindowSize::new(3, 3), ExtendValue::Given(0_f64)),
            a_values
        }
    }

    pub fn w(&self) -> usize { self.size.width }
    pub fn h(&self) -> usize { self.size.height }
}

impl Filter for HistogramLocalContrast {
    fn filter(&self, img: Matrix2D) -> Matrix2D {
        let mut pixel_buf = Vec::<f64>::new();
        pixel_buf.resize(self.w() * self.h(), 0_f64);

        let fil_half_size = PixelPos::new(self.h() / 2, self.w() / 2);

        let ext_copy = img.copy_with_extended_borders(ExtendValue::Closest, 
            fil_half_size.row, fil_half_size.col);
        let mut hist_matrix = Matrix2D::empty_with_size(
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

        let mut c_mat = Matrix2D::empty_with_size(img_filtered_ext.w(), img_filtered_ext.h());
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
            
            c_power = self.a_values.min + (self.a_values.max - self.a_values.min) * c_power;
            
            c_mat[m_pos] = c_mat[m_pos].powf(c_power);
        }
        
        let mut img_result = Matrix2D::empty_with_size(img.w(), img.h());

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
    
    fn w(&self) -> usize { self.size.width }

    fn h(&self) -> usize { self.size.height }

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
        let mut lines_iter = LinesIter::new(string);
        if lines_iter.len() != 3 { return Err(MyError::new("Должно быть 2 строки".to_string())); }

        let size = FilterWindowSize::try_from_string(lines_iter.next())?
            .check_size_be_3()?
            .check_w_equals_h()?
            .check_w_h_odd()?;

        let ext_value = ExtendValue::try_from_string(lines_iter.next())?;

        let a_values = ARange::try_from_string(&lines_iter.next())?;

        return Ok(HistogramLocalContrast::new(size, ext_value, a_values));
    }
    
    fn content_to_string(&self) -> String {
        format!("{}\n{}\n{}", self.size.content_to_string(), self.ext_value.content_to_string(), self.a_values.content_to_string())
    }
}

impl Default for HistogramLocalContrast {
    fn default() -> Self {
        HistogramLocalContrast::new(FilterWindowSize::new(3, 3), ExtendValue::Closest, ARange::new(0.5, 0.5))
    }
}



#[derive(Clone)]
pub struct CutBrightness {
    cut_range: CutBrightnessRange,
    replace_with: ValueRepaceWith
}

impl CutBrightness {
    pub fn new(cut_range: CutBrightnessRange, replace_with: ValueRepaceWith) -> Self {
        CutBrightness { cut_range, replace_with }
    }
}

impl Filter for CutBrightness {
    fn filter(&self, mut img: crate::img::Matrix2D) -> crate::img::Matrix2D {
        for pos in img.get_iterator() {
            if img[pos] >= self.cut_range.min as f64 && img[pos] <= self.cut_range.max as f64 {
                img[pos] = self.replace_with.value as f64
            }
        }
        img
    }
}

impl Default for CutBrightness {
    fn default() -> Self {
        Self::new(CutBrightnessRange::new(100, 200), ValueRepaceWith::new(0))
    }
}

impl StringFromTo for CutBrightness {
    fn try_from_string(string: &str) -> Result<Self, MyError> where Self: Sized {
        let mut lines_iter = LinesIter::new(string);
        if lines_iter.len() != 2 { return Err(MyError::new("Должно быть 2 строки".to_string())); }

        let cut_range = CutBrightnessRange::try_from_string(lines_iter.next())?;

        let replace_with = ValueRepaceWith::try_from_string(lines_iter.next())?;

        Ok(CutBrightness::new(cut_range, replace_with))
    }

    fn content_to_string(&self) -> String {
        format!("{}\n{}", self.cut_range.content_to_string(), self.replace_with.content_to_string())
    }
}
