pub mod filter_trait;
pub mod filter_option;
pub mod linear;
pub mod non_linear;

use crate::{img::{Matrix2D, pixel_pos::PixelPos}};

use self::filter_trait::WindowFilter;

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


fn filter_window<T: WindowFilter>(mut img: Matrix2D, filter: &T, buf_filt_fcn: fn(f: &T, &mut [f64]) -> f64) -> Matrix2D {
    assert!(filter.w() > 1);
    assert!(filter.h() > 1);

    let mut pixel_buf = Vec::<f64>::new();
    pixel_buf.resize(filter.w() * filter.h(), 0_f64);

    let fil_half_size = PixelPos::new(filter.h() / 2, filter.w() / 2);

    let img_extended = img.copy_with_extended_borders(
        filter.get_extend_value(), 
        fil_half_size.row, 
        fil_half_size.col);

    for pos_im in img_extended.get_area_iter(
        fil_half_size, 
        PixelPos::new(img.h(), img.w()) + fil_half_size)
    {
        for pos_w in filter.get_iterator() {            
            let buf_ind: usize = pos_w.row * filter.w() + pos_w.col;
            let pix_pos: PixelPos = pos_im + pos_w - fil_half_size;
            pixel_buf[buf_ind] = img_extended[pix_pos];
        }

        let filter_result: f64 = buf_filt_fcn(filter, &mut pixel_buf[..]);
        
        img[pos_im - fil_half_size] = filter_result;
    }

    img
}

