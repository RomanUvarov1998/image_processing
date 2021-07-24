use crate::img::Matrix2D;
use super::{PixelPos, PixelsIter};


#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PixelsArea {
    top_left: PixelPos,
    bottom_right: PixelPos
}

impl PixelsArea {
    pub fn new(top_left: PixelPos, bottom_right: PixelPos) -> Self {
        assert!(top_left.row <= bottom_right.row);
        assert!(top_left.col <= bottom_right.col);
        PixelsArea { top_left, bottom_right }
    }

    pub fn size_of(matrix: &Matrix2D) -> Self {
        PixelsArea::new(
            PixelPos::new(0, 0),
            PixelPos::new(matrix.h() - 1, matrix.w() - 1))
    }

    pub fn with_size(height: usize, width: usize) -> Self {
        PixelsArea::new(
            PixelPos::new(0, 0),
            PixelPos::new(height - 1, width - 1))
    }

    pub fn with_pos(self, top_left_row: usize, top_left_col: usize) -> Self {
        let top_left = PixelPos::new(top_left_row, top_left_col);
        PixelsArea::new(
            top_left,
            top_left + self.bottom_right)
    }

    pub fn apply_margin(self, margin: Margin) -> Self {
        let (left, top, right, bottom) = margin.get_margins();
        let top_left_offset = PixelPos::new(top, left);
        let bottom_right_offset = PixelPos::new(bottom, right);
        PixelsArea::new(
            self.top_left + top_left_offset,
            self.bottom_right - bottom_right_offset)
    }


    pub fn contains(&self, pos: PixelPos) -> bool {
        self.top_left.row <= pos.row && pos.row <= self.bottom_right.row
        && self.top_left.col <= pos.col && pos.col <= self.bottom_right.col
    }


    pub fn top_left(&self) -> PixelPos {
        self.top_left
    }
    pub fn bottom_right(&self) -> PixelPos {
        self.bottom_right
    }

    pub fn w(&self) -> usize { self.bottom_right.col - self.top_left.col + 1 }
    pub fn h(&self) -> usize { self.bottom_right.row - self.top_left.row + 1 }


    pub fn get_rows_range(&self) -> std::ops::RangeInclusive<usize> {
        self.top_left.row..=self.bottom_right.row
    }
    pub fn get_cols_range(&self) -> std::ops::RangeInclusive<usize> {
        self.top_left.col..=self.bottom_right.col
    }


    pub fn get_pixels_iter(&self) -> PixelsIter {
        PixelsIter::for_area(self)
    }
}


pub enum Margin {
    Sides { left: usize, top: usize, right: usize, bottom: usize },
    TwoPoints { top_left: PixelPos, bottom_right: PixelPos },
    All(usize)
}

impl Margin {
    fn get_margins(&self) -> (usize, usize, usize, usize) {
        match self {
            Margin::Sides { left, top, right, bottom } => 
                (*left, *top, *right, *bottom),
            Margin::All(m) => (*m, *m, *m, *m),
            Margin::TwoPoints { top_left, bottom_right } => 
                (top_left.col, top_left.row, bottom_right.col, bottom_right.row),
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::img::PixelPos;
    use super::{Margin, PixelsArea};

    #[test]
    #[should_panic(expected = "assertion failed")]
    fn cannot_create_area_with_negative_width() {
        PixelsArea::new(
            PixelPos::new(3, 4), 
            PixelPos::new(2, 6));
    }

    #[test]
    #[should_panic(expected = "assertion failed")]
    fn cannot_create_area_with_negative_height() {
        PixelsArea::new(
            PixelPos::new(3, 4), 
            PixelPos::new(4, 3));
    }

    #[test]
    #[should_panic(expected = "assertion failed")]
    fn cannot_create_area_with_negative_width_and_height() {
        PixelsArea::new(
            PixelPos::new(3, 4), 
            PixelPos::new(1, 2));
    }

    #[test]
    fn can_create_area_with_zero_dimensions() {
        PixelsArea::new(
            PixelPos::new(3, 4), 
            PixelPos::new(3, 5));

        PixelsArea::new(
            PixelPos::new(3, 4), 
            PixelPos::new(4, 4));

        PixelsArea::new(
            PixelPos::new(3, 4), 
            PixelPos::new(3, 4));
    }

    #[test]
    fn with_size() {
        let area = PixelsArea::with_size(4, 3);

        assert_eq!(area.top_left(), PixelPos::new(0, 0));
        assert_eq!(area.bottom_right(), PixelPos::new(3, 2));
    }

    #[test]
    fn with_pos() {
        let area = PixelsArea::with_size(4, 3);
        assert_eq!(area.top_left, PixelPos::new(0, 0));
        assert_eq!(area.bottom_right, PixelPos::new(3, 2));

        let area2 = area.with_pos(1, 2);
        assert_eq!(area2.top_left, PixelPos::new(1, 2));
        assert_eq!(area2.bottom_right, PixelPos::new(4, 4));
    }

    #[test]
    fn apply_margin() {
        let area = PixelsArea::with_size(12, 11);
        {
            let area2 = area.apply_margin(Margin::All(2));
            assert_eq!(area2.top_left(), PixelPos::new(2, 2));
            assert_eq!(area2.bottom_right(), PixelPos::new(12 - 1 - 2, 11 - 1 - 2));
        }
        {
            let m = Margin::Sides { left: 1, top: 2, right: 3, bottom: 4 };
            let area2 = area.apply_margin(m);
            assert_eq!(area2.top_left(), PixelPos::new(2, 1));
            assert_eq!(area2.bottom_right(), PixelPos::new(12 - 1 - 4, 11 - 1 - 3));
        }
        {
            let top_left = PixelPos::new(2, 1);
            let bottom_right = PixelPos::new(4, 3);
            let m = Margin::TwoPoints { top_left, bottom_right };
            let area2 = area.apply_margin(m);
            assert_eq!(area2.top_left(), PixelPos::new(2, 1));
            assert_eq!(area2.bottom_right(), PixelPos::new(12 - 1 - 4, 11 - 1 - 3));
        }
    }
    
    #[test]
    fn contains() {
        let area = PixelsArea::new(
            PixelPos::new(1, 2),
            PixelPos::new(3, 4));
            
        assert!(!area.contains(PixelPos::new(0, 1)));
        assert!(!area.contains(PixelPos::new(0, 2)));
        assert!(!area.contains(PixelPos::new(0, 3)));
        assert!(!area.contains(PixelPos::new(0, 4)));
        assert!(!area.contains(PixelPos::new(0, 5)));

        assert!(!area.contains(PixelPos::new(1, 1)));
        assert!(area.contains(PixelPos::new(1, 2)));
        assert!(area.contains(PixelPos::new(1, 3)));
        assert!(area.contains(PixelPos::new(1, 4)));
        assert!(!area.contains(PixelPos::new(1, 5)));
        
        assert!(!area.contains(PixelPos::new(2, 1)));
        assert!(area.contains(PixelPos::new(2, 2)));
        assert!(area.contains(PixelPos::new(2, 3)));
        assert!(area.contains(PixelPos::new(2, 4)));
        assert!(!area.contains(PixelPos::new(2, 5)));

        assert!(!area.contains(PixelPos::new(3, 1)));
        assert!(area.contains(PixelPos::new(3, 2)));
        assert!(area.contains(PixelPos::new(3, 3)));
        assert!(area.contains(PixelPos::new(3, 4)));
        assert!(!area.contains(PixelPos::new(3, 5)));
            
        assert!(!area.contains(PixelPos::new(4, 1)));
        assert!(!area.contains(PixelPos::new(4, 2)));
        assert!(!area.contains(PixelPos::new(4, 3)));
        assert!(!area.contains(PixelPos::new(4, 4)));
        assert!(!area.contains(PixelPos::new(4, 5)));
    }

    #[test]
    fn top_left_bottom_right() {
        let area = PixelsArea::new(
            PixelPos::new(1, 2),
            PixelPos::new(3, 4));
            
        assert_eq!(area.top_left(), PixelPos::new(1, 2));
        assert_eq!(area.bottom_right(), PixelPos::new(3, 4));
    }

    #[test]
    fn w_h() {
        let area = PixelsArea::new(
            PixelPos::new(1, 2),
            PixelPos::new(3, 5));
            
        assert_eq!(area.w(), 4);
        assert_eq!(area.h(), 3);
    }

    #[test]
    fn get_rows_range_get_cols_range() {
        let area = PixelsArea::new(
            PixelPos::new(1, 2),
            PixelPos::new(3, 4));
            
        let mut rows_range = area.get_rows_range();
        assert_eq!(rows_range.next(), Some(1));
        assert_eq!(rows_range.next(), Some(2));
        assert_eq!(rows_range.next(), Some(3));
        assert_eq!(rows_range.next(), None);
            
        let mut cols_range = area.get_cols_range();
        assert_eq!(cols_range.next(), Some(2));
        assert_eq!(cols_range.next(), Some(3));
        assert_eq!(cols_range.next(), Some(4));
        assert_eq!(cols_range.next(), None);
    }

    #[test]
    fn get_pixels_iter() {
        let area = PixelsArea::new(
            PixelPos::new(1, 2),
            PixelPos::new(3, 4));
            
        let mut iter = area.get_pixels_iter();

        assert_eq!(iter.next(), Some(PixelPos::new(1, 2)));
        assert_eq!(iter.next(), Some(PixelPos::new(1, 3)));
        assert_eq!(iter.next(), Some(PixelPos::new(1, 4)));

        assert_eq!(iter.next(), Some(PixelPos::new(2, 2)));
        assert_eq!(iter.next(), Some(PixelPos::new(2, 3)));
        assert_eq!(iter.next(), Some(PixelPos::new(2, 4)));

        assert_eq!(iter.next(), Some(PixelPos::new(3, 2)));
        assert_eq!(iter.next(), Some(PixelPos::new(3, 3)));
        assert_eq!(iter.next(), Some(PixelPos::new(3, 4)));

        assert_eq!(iter.next(), None);
    }
}