use crate::pixel_pos::PixelPos;

pub trait Filter {
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
    pub fn mean_filter_of_size(size: usize) -> Self {
        assert_eq!(size % 2, 1);

        let mut arr = Vec::<f64>::new();
        let coeff = 1_f64 / ((size * size) as f64);
        arr.resize(size * size, coeff);
        LinearFilter { width: size, height: size, arr }
    }

    pub fn w(&self) -> usize { self.width }
    pub fn h(&self) -> usize { self.height }
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

pub struct MedianFilter {
    size: usize,
}

impl MedianFilter {
    pub fn new(size: usize) -> Self {
        assert_eq!(size % 2, 1);
        MedianFilter { size }
    }

    pub fn w(&self) -> usize { self.size }
    pub fn h(&self) -> usize { self.size }
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