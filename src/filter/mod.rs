pub mod filter_trait;
pub mod utils;
pub mod filter_option;
pub mod linear;
pub mod non_linear;
pub mod channel;

use crate::{img::{Img, ImgLayer, Matrix2D, img_ops, pixel_pos::PixelPos}, processing::progress_provider::{HaltError, ProgressProvider}};

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


fn process_with_window<T: WindowFilter>(
    init: &Matrix2D,      
    filter: &T, 
    buf_filt_fcn: fn(f: &T, &mut [f64]) -> f64, 
    prog_prov: &mut ProgressProvider) 
    -> Result<Matrix2D, HaltError>
{
    assert!(filter.w() > 1);
    assert!(filter.h() > 1);

    let mut res = Matrix2D::empty_size_of(init);

    let mut pixel_buf = Vec::<f64>::new();
    pixel_buf.resize(filter.w() * filter.h(), 0_f64);

    let fil_half_size = PixelPos::new(filter.h() / 2, filter.w() / 2);

    let layer_ext = img_ops::extend_matrix_for_window_filter(init, filter);

    for pos_im in layer_ext.get_pixels_area_iter(
        fil_half_size, 
        PixelPos::new(init.h(), init.w()) + fil_half_size)
    {
        for pos_w in filter.get_iter() {            
            let buf_ind: usize = pos_w.row * filter.w() + pos_w.col;
            let pix_pos: PixelPos = pos_im + pos_w - fil_half_size;
            pixel_buf[buf_ind] = layer_ext[pix_pos];
        }

        let filter_result: f64 = buf_filt_fcn(filter, &mut pixel_buf[..]);
        
        res[pos_im - fil_half_size] = filter_result;

        prog_prov.complete_action()?;
    }

    Ok(res)
}


// AnyFilter : ByLayer -> filter<>() -> filter_layers<>() -> process_layer<>()

trait ByLayer {
    fn process_layer(
        &self,
        layer: &ImgLayer, 
        prog_prov: &mut ProgressProvider) -> Result<ImgLayer, HaltError>;
}

fn process_each_layer<F: ByLayer>(
    img: &Img, 
    filter: &F, 
    prog_prov: &mut ProgressProvider) -> Result<Img, HaltError> 
{
    let mut res_layers = Vec::<ImgLayer>::with_capacity(img.d());

    for layer in img.get_layers_iter() {
        let res_layer = filter.process_layer(layer, prog_prov)?;        
        res_layers.push(res_layer);
    }        

    Ok(Img::new(img.w(), img.h(), res_layers, img.color_depth()))
}