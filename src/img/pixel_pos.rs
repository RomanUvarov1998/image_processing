use std::ops::{Add, AddAssign, Sub, SubAssign};

#[derive(Clone, Copy, Debug, PartialEq)]
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
        assert!(self.col > 0);
        PixelPos::new(self.row + 0, self.col - 1) 
    }
    pub fn upper_lefter(&self) -> Self { 
        assert!(self.col > 0);
        assert!(self.row > 0);
        PixelPos::new(self.row - 1, self.col - 1) 
    }
    pub fn upper(&self) -> Self { 
        assert!(self.row > 0);
        PixelPos::new(self.row - 1, self.col + 0) 
    }
    pub fn upper_righter(&self) -> Self { 
        assert!(self.row > 0);
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
        assert!(self.col > 0);
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


#[cfg(test)]
mod tests {
    use super::PixelPos;

    #[test]
    fn ctor_one_produces_one_one() {
        let one = PixelPos::one();
        assert_eq!(one, PixelPos::new(1, 1));
    }

    #[test]
    fn row_vec_and_col_vec() {
        let pos = PixelPos::new(4, 5);

        let row_vec = pos.row_vec();
        assert_eq!(row_vec, PixelPos::new(pos.row, 0));

        let col_vec = pos.col_vec();
        assert_eq!(col_vec, PixelPos::new(0, pos.col));
    }

    #[test]
    fn with_row_and_with_col_ctors() {
        let pos = PixelPos::new(4, 5);
        
        let v1 = pos.with_col(3);
        assert_eq!(v1, PixelPos::new(pos.row, 3));
        
        let v1 = pos.with_row(2);
        assert_eq!(v1, PixelPos::new(2, pos.col));
    }

    #[test]
    fn neighbours() {
        const ROW: usize = 4;
        const COL: usize = 5;
        let pos = PixelPos::new(ROW, COL);
        
        let p = pos.lefter();
        assert_eq!(p, PixelPos::new(ROW, COL - 1));
        
        let p = pos.upper_lefter();
        assert_eq!(p, PixelPos::new(ROW - 1, COL - 1));
        
        let p = pos.upper();
        assert_eq!(p, PixelPos::new(ROW - 1, COL));
        
        let p = pos.upper_righter();
        assert_eq!(p, PixelPos::new(ROW - 1, COL + 1));
        
        let p = pos.righter();
        assert_eq!(p, PixelPos::new(ROW, COL + 1));
        
        let p = pos.downer_righter();
        assert_eq!(p, PixelPos::new(ROW + 1, COL + 1));
        
        let p = pos.downer();
        assert_eq!(p, PixelPos::new(ROW + 1, COL));
        
        let p = pos.downer_lefter();
        assert_eq!(p, PixelPos::new(ROW + 1, COL - 1));
    }

    #[test]
    #[should_panic(expected = "assertion failed")]
    fn cannot_have_upper_left_neigbour_if_row_is_0() {
        let pos = PixelPos::new(0, 3);
        let _n1 = pos.upper_lefter();
    }

    #[test]
    #[should_panic(expected = "assertion failed")]
    fn cannot_have_upper_neigbour_if_row_is_0() {
        let pos = PixelPos::new(0, 3);
        let _n1 = pos.upper();
    }

    #[test]
    #[should_panic(expected = "assertion failed")]
    fn cannot_have_upper_righter_neigbour_if_row_is_0() {
        let pos = PixelPos::new(0, 3);
        let _n1 = pos.upper_righter();
    }
    
    #[test]
    #[should_panic(expected = "assertion failed")]
    fn cannot_have_upper_lefter_neigbour_if_col_is_0() {
        let pos = PixelPos::new(3, 0);
        let _n1 = pos.upper_lefter();
    }
    
    #[test]
    #[should_panic(expected = "assertion failed")]
    fn cannot_have_lefter_neigbour_if_col_is_0() {
        let pos = PixelPos::new(3, 0);
        let _n1 = pos.lefter();
    }
    
    #[test]
    #[should_panic(expected = "assertion failed")]
    fn cannot_have_downer_lefter_neigbour_if_col_is_0() {
        let pos = PixelPos::new(3, 0);
        let _n1 = pos.downer_lefter();
    }

    #[test]
    fn default_is_zero_zero() {
        let pos = PixelPos::default();
        assert_eq!(pos, PixelPos::new(0, 0));
    }

    #[test]
    fn operators() {
        let mut pos1 = PixelPos::new(3, 4);
        let pos2 = PixelPos::new(1, 2);

        assert_eq!(pos1 + pos2, PixelPos::new(4, 6));
        assert_eq!(pos1 - pos2, PixelPos::new(2, 2));

        pos1 += pos2;
        assert_eq!(pos1, PixelPos::new(4, 6));

        pos1 -= pos2;
        assert_eq!(pos1, PixelPos::new(3, 4));
    }
}