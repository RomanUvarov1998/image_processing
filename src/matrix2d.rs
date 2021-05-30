use std::ops::{Index, IndexMut};
use crate::pixel_pos::PixelPos;

pub struct Matrix2D {
    width: usize,
    height: usize,
    values: Vec<f64>
}

impl Matrix2D {
    pub fn empty(width: usize, height: usize) -> Self {
        let mut values = Vec::<f64>::new();
        values.resize(width * height, 0_f64);
        Matrix2D { width, height, values }
    }
    
    pub fn w(&self) -> usize { self.width }
    pub fn h(&self) -> usize { self.height }

    #[allow(dead_code)]
    pub fn get_iterator(&self) -> Matrix2DIterator {
        Matrix2DIterator::for_full(self)
    }

    pub fn get_area_iter(&self, from: PixelPos, to_excluded: PixelPos) -> Matrix2DIterator 
    {
        Matrix2DIterator::for_rect_area(from, to_excluded)
    }
}

type TupIdx = (usize, usize);
impl Index<TupIdx> for Matrix2D {
    type Output = f64;

    fn index(&self, index: TupIdx) -> &Self::Output {
        assert!(index.0 < self.width);
        assert!(index.1 < self.height);

        &self.values[index.1 * self.width + index.0]
    }
}
impl IndexMut<TupIdx> for Matrix2D {
    fn index_mut(&mut self, index: TupIdx) -> &mut Self::Output {
        assert!(index.0 < self.width);
        assert!(index.1 < self.height);
        
        &mut self.values[index.1 * self.width + index.0]
    }
}

impl Index<PixelPos> for Matrix2D {
    type Output = f64;

    fn index(&self, index: PixelPos) -> &Self::Output {
        assert!(index.col < self.width);
        assert!(index.row < self.height);

        &self.values[index.row * self.width + index.col]
    }
}
impl IndexMut<PixelPos> for Matrix2D {
    fn index_mut(&mut self, index: PixelPos) -> &mut Self::Output {
        assert!(index.col < self.width);
        assert!(index.row < self.height);

        &mut self.values[index.row * self.width + index.col]
    }
}

pub struct Matrix2DIterator {
    top_left: PixelPos,
    bottom_right_excluded: PixelPos,
    cur_pos: PixelPos
}

impl Matrix2DIterator {
    #[allow(dead_code)]
    pub fn for_full(img: &Matrix2D) -> Self {
        Matrix2DIterator {
            top_left: PixelPos::new(0, 0),
            bottom_right_excluded: PixelPos::new(img.h(), img.w()),
            cur_pos: PixelPos::new(0, 0)
        }
    }

    pub fn for_rect_area(top_left: PixelPos, bottom_right_excluded: PixelPos) -> Self {
        assert!(top_left.row < bottom_right_excluded.row);
        assert!(top_left.col < bottom_right_excluded.col);

        Matrix2DIterator {
            top_left,
            bottom_right_excluded,
            cur_pos: top_left
        }
    }

    pub fn fits(&self, pos: PixelPos) -> bool {
        let mut val = 
        self.top_left.col <= pos.col && pos.col < self.bottom_right_excluded.col 
        && self.top_left.row <= pos.row && pos.row < self.bottom_right_excluded.row ;
        if val == false {
            val = true;
        }
        val
    }
}

impl Iterator for Matrix2DIterator {
    type Item = PixelPos;

    fn next(&mut self) -> Option<PixelPos> {
        let curr = self.cur_pos;

        assert!(self.fits(self.cur_pos));

        if self.cur_pos.col < self.bottom_right_excluded.col - 1 {
            self.cur_pos.col += 1;
            return Some(curr);
        } else if self.cur_pos.row < self.bottom_right_excluded.row - 1 {
            self.cur_pos.col = self.top_left.col;
            self.cur_pos.row += 1;
            return Some(curr);
        } else {
            self.cur_pos = self.top_left;
            return None;
        }        
    }
}