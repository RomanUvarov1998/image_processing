use super::*;


pub struct PixelsIterator {
    top_left: PixelPos,
    bottom_right_excluded: PixelPos,
    cur_pos: PixelPos,
}

impl PixelsIterator {
    pub fn for_full_image(layer: &Matrix2D) -> Self {
        PixelsIterator {
            top_left: PixelPos::new(0, 0),
            bottom_right_excluded: PixelPos::new(layer.h(), layer.w()),
            cur_pos: PixelPos::new(0, 0),
        }
    }

    pub fn for_rect_area(top_left: PixelPos, bottom_right_excluded: PixelPos) -> Self {
        assert!(top_left.row < bottom_right_excluded.row);
        assert!(top_left.col < bottom_right_excluded.col);

        PixelsIterator {
            top_left,
            bottom_right_excluded,
            cur_pos: top_left,
        }
    }

    pub fn fits(&self, pos: PixelPos) -> bool {
        let val = 
            self.top_left.col <= pos.col && pos.col < self.bottom_right_excluded.col 
            && self.top_left.row <= pos.row && pos.row < self.bottom_right_excluded.row;
        val
    }
}

impl Iterator for PixelsIterator {
    type Item = PixelPos;

    fn next(&mut self) -> Option<PixelPos> {
        let curr = self.cur_pos;

        self.cur_pos.col += 1;

        if self.cur_pos.col >= self.bottom_right_excluded.col {
            self.cur_pos.col = self.top_left.col;
            self.cur_pos.row += 1;
        }

        return if self.fits(curr) {
            Some(curr)
        } else {
            None
        };
    }
}


/*
pub struct RowsIter {
    left_col: usize,
    right_col_excluded: usize,
    max_row_excluded: usize,
    cur_row: usize,
}

impl RowsIter {
    pub fn new(range: Range<usize>) -> Self {
        RowsIter {
            left_col: area.top_left().col,
            right_col_excluded: area.bottom_right().col,
            max_row_excluded: area.bottom_right().row,
            cur_row: area.top_left().col,
        }
    }
}

impl Iterator for RowsIter {
    type Item = PixelsRow;

    fn next(&mut self) -> Option<Self::Item> {
        let curr_row = self.cur_row;

        self.cur_row += 1;

        if curr_row < self.max_row_excluded {
            let left = PixelPos::new(curr_row, self.left_col);
            let right = PixelPos::new(curr_row, self.right_col_excluded);
            Some(PixelsRow::new(left, right))
        } else {
            None
        }
    }
}


#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PixelsRow {
    left: PixelPos,
    right: PixelPos
}

impl PixelsRow {
    pub fn new(left: PixelPos, right: PixelPos) -> Self {
        assert!(left.row == right.row);
        assert!(left.col <= right.col);
        PixelsRow {
            left,
            right
        }
    }

    pub fn get_cols_iter(&self) -> ColsIter {
        ColsIter::new(self)
    }
}


pub struct ColsIter {
    row: usize,
    right_col_excluded: usize,
    cur_col: usize,
}

impl ColsIter {
    pub fn new(row: &PixelsRow) -> Self {
        ColsIter {
            row: row.left.row,
            right_col_excluded: row.right.col,
            cur_col: row.left.col,
        }
    }
}

impl Iterator for ColsIter {
    type Item = PixelPos;

    fn next(&mut self) -> Option<Self::Item> {
        let cur_col = self.cur_col;

        self.cur_col += 1;

        if cur_col < self.right_col_excluded {
            Some(PixelPos::new(self.row, cur_col))
        } else {
            None
        }
    }
}
 */


pub struct LayersIterator<'own> {
    img: &'own Img,
    curr_layer_num: usize
}

impl<'own> LayersIterator<'own> {
    pub fn new(img: &'own Img) -> Self {
        LayersIterator {
            img,
            curr_layer_num: 0
        }
    }
}

impl<'own> Iterator for LayersIterator<'own> {
    type Item = &'own ImgLayer;

    fn next(&mut self) -> Option<&'own ImgLayer> {
        let curr_num = self.curr_layer_num;

        if self.curr_layer_num < self.img.layers().len() {
            self.curr_layer_num += 1;
            let layer = self.img.layer(curr_num);
            return Some(layer);
        } else {
            self.curr_layer_num = 0;
            return None;
        }        
    }
}


#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PixelsArea {
    top_left: PixelPos,
    bottom_right: PixelPos
}

impl PixelsArea {
    pub fn new(top_left: PixelPos, bottom_right: PixelPos) -> Self {
        PixelsArea { top_left, bottom_right }
    }

    pub fn from_zero_to(bottom: usize, right: usize) -> Self {
        PixelsArea {
            top_left: PixelPos::new(0, 0),
            bottom_right: PixelPos::new(bottom, right),
        }
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

    pub fn get_rows_range(&self) -> std::ops::Range<usize> {
        self.top_left.row..self.bottom_right.row
    }

    pub fn get_cols_range(&self) -> std::ops::Range<usize> {
        self.top_left.col..self.bottom_right.col
    }
}


#[cfg(test)]
mod tests {
    use super::super::PixelPos;

    #[test]
    fn pixels_iter_for_area_returns_all_positions() {
        let mut iter = super::PixelsIterator::for_rect_area(
            PixelPos::new(0, 0),
            PixelPos::new(3, 3));

        assert_eq!(iter.next().unwrap(), PixelPos::new(0, 0));
        assert_eq!(iter.next().unwrap(), PixelPos::new(0, 1));
        assert_eq!(iter.next().unwrap(), PixelPos::new(0, 2));

        assert_eq!(iter.next().unwrap(), PixelPos::new(1, 0));
        assert_eq!(iter.next().unwrap(), PixelPos::new(1, 1));
        assert_eq!(iter.next().unwrap(), PixelPos::new(1, 2));

        assert_eq!(iter.next().unwrap(), PixelPos::new(2, 0));
        assert_eq!(iter.next().unwrap(), PixelPos::new(2, 1));
        assert_eq!(iter.next().unwrap(), PixelPos::new(2, 2));

        assert_eq!(iter.next(), None);
    }

    /*
    #[test]
    fn rows_iter_for_area_returns_all_positions() {
        let area = super::PixelsArea::new(
            PixelPos::new(0, 0),
            PixelPos::new(3, 3));
        let mut iter = super::RowsIter::new(area);
    
        use super::PixelsRow;

        assert_eq!(iter.next().unwrap(), PixelsRow::new(PixelPos::new(0, 0), PixelPos::new(0, 3)));
        assert_eq!(iter.next().unwrap(), PixelsRow::new(PixelPos::new(1, 0), PixelPos::new(1, 3)));
        assert_eq!(iter.next().unwrap(), PixelsRow::new(PixelPos::new(2, 0), PixelPos::new(2, 3)));
        
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn cols_iter_returns_all_cols() {
        const ROW: usize = 5;
        let row = super::PixelsRow::new(
            PixelPos::new(ROW, 0), 
            PixelPos::new(ROW, 4));
        let mut iter = super::ColsIter::new(&row);

        assert_eq!(iter.next().unwrap(), PixelPos::new(ROW, 0));
        assert_eq!(iter.next().unwrap(), PixelPos::new(ROW, 1));
        assert_eq!(iter.next().unwrap(), PixelPos::new(ROW, 2));
        assert_eq!(iter.next().unwrap(), PixelPos::new(ROW, 3));
        assert_eq!(iter.next(), None);
    }
     */
}
