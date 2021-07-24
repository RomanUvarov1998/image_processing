use super::*;

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







