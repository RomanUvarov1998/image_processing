use std::ops::{Add, AddAssign, Sub, SubAssign};

#[derive(Debug, Clone, Copy)]
pub struct PixelPos { pub col: usize, pub row: usize }

impl PixelPos {
    pub fn new(row: usize, col: usize) -> Self {
        PixelPos { row, col }
    }

    pub fn one() -> Self {
        PixelPos { col: 1, row: 1 }
    }

    pub fn row_vec(&self) -> Self { PixelPos::new(self.row, 0) }
    pub fn col_vec(&self) -> Self { PixelPos::new(0, self.col) }

    pub fn with_row(&self, row: usize) -> Self { PixelPos::new(row, self.col) }
    pub fn with_col(&self, col: usize) -> Self { PixelPos::new(self.row, col) }

    pub fn lefter(&self) -> Self { 
        PixelPos::new(self.row + 0, self.col - 1) 
    }
    pub fn upper_lefter(&self) -> Self { 
        PixelPos::new(self.row - 1, self.col - 1) 
    }
    pub fn upper(&self) -> Self { 
        PixelPos::new(self.row - 1, self.col + 0) 
    }
    pub fn upper_righter(&self) -> Self { 
        PixelPos::new(self.row - 1, self.col + 1) 
    }
    pub fn righter(&self) -> Self { 
        PixelPos::new(self.row + 0, self.col + 1) 
    }
    pub fn downer_righter(&self) -> Self { 
        PixelPos::new(self.row + 1, self.col + 1) 
    }
    pub fn downer(&self) -> Self { 
        PixelPos::new(self.row + 1, self.col + 0) 
    }
    pub fn downer_lefter(&self) -> Self { 
        PixelPos::new(self.row + 1, self.col - 1) 
    }
}

impl Default for PixelPos {
    fn default() -> Self {
        PixelPos { col: 0, row: 0 }
    }
}

impl Add for PixelPos {
    type Output = PixelPos;

    fn add(self, rhs: Self) -> Self::Output {
        PixelPos::new(self.row + rhs.row, self.col + rhs.col)
    }
}

impl AddAssign for PixelPos {
    fn add_assign(&mut self, rhs: Self) {
        self.row += rhs.row;
        self.col += rhs.col;
    }
}

impl Sub for PixelPos {
    type Output = PixelPos;

    fn sub(self, rhs: Self) -> Self::Output {
        PixelPos::new(self.row - rhs.row, self.col - rhs.col)
    }
}

impl SubAssign for PixelPos {
    fn sub_assign(&mut self, rhs: Self) {
        self.row -= rhs.row;
        self.col -= rhs.col;
    }
}
