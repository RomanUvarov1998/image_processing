use std::ops::{Add, AddAssign, Sub, SubAssign};

#[derive(Debug, Clone, Copy)]
pub struct PixelPos { pub col: usize, pub row: usize }

impl PixelPos {
    pub fn new(row: usize, col: usize) -> Self {
        PixelPos { row, col }
    }

    pub fn negative_if_substract(&self, other: PixelPos) -> bool {
        self.col < other.col || self.row < other.col
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
