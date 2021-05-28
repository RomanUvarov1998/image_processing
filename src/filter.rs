use std::ops::Range;

use crate::{my_err::MyError, pixel_pos::PixelPos};

pub const MAX_WINDOW_SIZE: usize = 11;
pub const MAX_WINDOW_BUFFER_SIZE: usize = MAX_WINDOW_SIZE * MAX_WINDOW_SIZE;

pub trait Filter : Default + Clone {
    fn filter(&mut self, fragment: &mut [f64]) -> f64;
    fn window_size(&self) -> usize;
    fn get_iterator(&self) -> FilterIterator;
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

#[derive(Clone)]
pub struct LinearFilter {
    width: usize,
    height: usize,
    arr: Vec<f64>,
}

impl LinearFilter {
    pub fn with_coeffs(coeffs: Vec<f64>, width: usize, height: usize) -> Self {
        assert!(width > 0);
        assert!(height > 0);
        assert!(coeffs.len() > 0);
        LinearFilter { width, height, arr: coeffs }
    }

    pub fn mean_filter_of_size(size: usize) -> Self {
        assert_eq!(size % 2, 1);

        let mut arr = Vec::<f64>::new();
        let coeff = 1_f64 / ((size * size) as f64);
        arr.resize(size * size, coeff);
        LinearFilter { width: size, height: size, arr }
    }

    pub fn w(&self) -> usize { self.width }
    pub fn h(&self) -> usize { self.height }
    
    pub fn to_string(&self) -> String {
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

        fil_string
    }

    pub fn try_from_string(string: &str) -> Result<Self, MyError> {
        let mut rows = Vec::<Vec<f64>>::new();

        for line in string.split('\n') {
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
        
        let mut coeffs = Vec::<f64>::new();
        for mut row in rows.clone() { 
            coeffs.append(&mut row); 
        }
        let width = rows.last().unwrap().len();
        let height = rows.len();

        Ok(LinearFilter::with_coeffs(coeffs, width, height))
    }

    pub fn get_coeff(&self, row: usize, col: usize) -> f64 { self.arr[row * self.width + col] }

    pub fn add_row(&mut self) {
        self.arr.resize(self.arr.len() + self.width, 0_f64);
        self.height += 1;
    }
    
    pub fn add_col(&mut self) {
        self.arr.reserve(self.height);

        for row_offset in (0..3).map(|i: usize| i * 4 as usize) {
            self.arr.insert(row_offset + self.width, 0_f64);
        }
        
        self.width += 1;
    }

    pub fn set_coeff(&mut self, row: usize, col: usize, coeff: f64) { 
        self.arr[row * self.width + col] = coeff;
    }
}

impl Filter for LinearFilter {
    fn filter(&mut self, fragment: &mut [f64]) -> f64 {
        let mut sum: f64 = 0_f64;

        for pos in self.get_iterator() {
            let ind = pos.row * self.width + pos.col;
            sum += fragment[ind] * self.arr[ind];
        }
        
        sum
    }

    fn window_size(&self) -> usize { self.h() }

    fn get_iterator(&self) -> FilterIterator {
        FilterIterator {
            width: self.w(),
            height: self.h(),
            cur_pos: PixelPos::default()
        }
    }
}

impl Default for LinearFilter {
    fn default() -> Self {
        LinearFilter::mean_filter_of_size(3)
    }
}

#[derive(Clone)]
pub struct MedianFilter {
    size: usize,
}

impl MedianFilter {
    pub fn new(size: usize) -> Self {
        assert!(size > 0);
        assert_eq!(size % 2, 1);
        MedianFilter { size }
    }

    pub fn w(&self) -> usize { self.size }
    pub fn h(&self) -> usize { self.size }

    pub fn set_size(&mut self, new_size: usize) {
        self.size = new_size;
    }
}

impl Filter for MedianFilter {
    fn filter(&mut self, fragment: &mut [f64]) -> f64 {
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

    fn window_size(&self) -> usize { self.h() }

    fn get_iterator(&self) -> FilterIterator {
        FilterIterator {
            width: self.w(),
            height: self.h(),
            cur_pos: PixelPos::default()
        }
    }
}

impl Default for MedianFilter {
    fn default() -> Self {
        MedianFilter::new(3)
    }
}