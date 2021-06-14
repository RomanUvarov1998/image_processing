use crate::{filter::{filter_option::ExtendValue, filter_trait::WindowFilter}, img::pixel_pos::PixelPos};
use super::{Img, ImgChannel, ImgLayer, Matrix2D};

pub fn copy_with_extended_borders(img: &Img, with: ExtendValue, left: usize, top: usize, right: usize, bottom: usize) -> Img {
    let mut ext_layers = Vec::<ImgLayer>::with_capacity(img.d());

    for layer in img.layers() {
        let ext_layer = match layer.channel() {
            ImgChannel::A => {
                let mut ext_mat = Matrix2D::empty_with_size(left + layer.w() + right, top + layer.h() + bottom);
                ext_mat.set_rect(PixelPos::new(0, 0), ext_mat.size_vec(), 255_f64);
                ImgLayer::new(ext_mat, layer.channel())
            },
            _ => {
                let ext_mat = extend_matrix(layer.matrix(), with, left, top, right, bottom);
                ImgLayer::new(ext_mat, layer.channel())
            },
        };

        ext_layers.push(ext_layer);
    }

    Img::new(left + img.w() + right, top + img.h() + bottom, ext_layers, img.color_depth())
}


pub fn extend_matrix_for_window_filter<F: WindowFilter>(mat_init: &Matrix2D, filter: &F) -> Matrix2D {
    let left = filter.w() / 2;
    let top = filter.h() / 2;
    let right = left;
    let bottom = top;

    extend_matrix(mat_init, filter.get_extend_value(), left, top, right, bottom)
}

pub fn extend_matrix(mat_init: &Matrix2D, with: ExtendValue, 
    left: usize, top: usize, right: usize, bottom: usize) -> Matrix2D 
{
    let mut mat_ext = Matrix2D::empty_with_size(left + mat_init.w() + right, top + mat_init.h() + bottom);

    let origin = PixelPos::new(0, 0);

    let margin_left = PixelPos::new(0, left);
    let margin_top = PixelPos::new(top, 0);
    let margin_right = PixelPos::new(0, right);
    let margin_bottom = PixelPos::new(bottom, 0);

    let rect_left = PixelPos::new(mat_init.h(), left);
    let rect_top = PixelPos::new(top, mat_init.w());
    let rect_right = PixelPos::new(mat_init.h(), right);
    let rect_bottom = PixelPos::new(bottom, mat_init.w());

    let mat_size = PixelPos::new(mat_init.h(), mat_init.w());

    // ------------------------------------ top ------------------------------------
    if top > 0 {
        // top left
        if left > 0 {
            let tl = origin;
            let br_excluded = tl + margin_left + margin_top;
            match with {
                ExtendValue::Closest => mat_ext.set_rect(tl, br_excluded, mat_init[origin]),
                ExtendValue::Given(val) => mat_ext.set_rect(tl, br_excluded, val),
            }
        }
        // top middle
        let tl = margin_left;
        let br_excluded = tl + rect_top;
        match with {
            ExtendValue::Closest => {
                for pos in mat_ext.get_area_iter(tl, br_excluded) {
                    mat_ext[pos] = mat_init[pos.with_row(0) - margin_left];
                }  
            },
            ExtendValue::Given(val) => mat_ext.set_rect(tl, br_excluded, val),
        }    
        // top right
        if right > 0 { 
            let tl = margin_left + rect_top.col_vec();
            let br_excluded = tl + margin_right + margin_top;
            match with {
                ExtendValue::Closest => mat_ext.set_rect(tl, br_excluded, mat_init[PixelPos::new(0, mat_init.w() - 1)]),
                ExtendValue::Given(val) => mat_ext.set_rect(tl, br_excluded, val),
            }
        }
    }
    // ------------------------------------ middle ------------------------------------   
    // middle left  
    if left > 0 {
        let tl = margin_top;
        let br_excluded = tl + rect_left;
        match with {
            ExtendValue::Closest => {
                for pos in mat_ext.get_area_iter(tl, br_excluded) {
                    mat_ext[pos] = mat_init[pos.with_col(0) - margin_top];
                }
            },
            ExtendValue::Given(val) => mat_ext.set_rect(tl, br_excluded, val),
        }
    }
    // middle middle     
    let tl = margin_left + margin_top;
    let br_excluded = tl + mat_size;               
    for pos in mat_ext.get_area_iter(tl, br_excluded) {
        mat_ext[pos] = mat_init[pos - tl];
    }    
    // middle right
    if right > 0 { 
        let tl = margin_left + rect_top;
        let br_excluded = tl + rect_right;
        match with {
            ExtendValue::Closest => {          
                for pos in mat_ext.get_area_iter(tl, br_excluded) {
                    mat_ext[pos] = mat_init[pos.with_col(mat_init.w() - 1) - margin_top];
                } 
            },
            ExtendValue::Given(val) => mat_ext.set_rect(tl, br_excluded, val),
        }
    }
    
    // ------------------------------------ bottom ------------------------------------
    if bottom > 0 {
        // bottom left
        if left > 0{
            let tl = margin_top + rect_left.row_vec();
            let br_excluded = tl + margin_left + margin_bottom;
            match with {
                ExtendValue::Closest => mat_ext.set_rect(tl, br_excluded, mat_init[PixelPos::new(mat_init.h() - 1, 0)]),
                ExtendValue::Given(val) => mat_ext.set_rect(tl, br_excluded, val),
            }
        }
        // bottom middle
        let tl = margin_top + rect_left;
        let br_excluded = tl + rect_bottom;
        match with {
            ExtendValue::Closest => {   
                for pos in mat_ext.get_area_iter(tl, br_excluded) {
                    mat_ext[pos] = mat_init[pos.with_row(mat_init.h() - 1) - margin_left];
                } 
            },
            ExtendValue::Given(val) => mat_ext.set_rect(tl, br_excluded, val),
        }       
        // bottom right
        if right > 0 {
            let tl = margin_left + margin_top + mat_init.size_vec();
            let br_excluded = tl + margin_right + margin_bottom;
            match with {
                ExtendValue::Closest => mat_ext.set_rect(tl, br_excluded, mat_init[mat_init.size_vec() - PixelPos::one()]),
                ExtendValue::Given(val) => mat_ext.set_rect(tl, br_excluded, val),
            }
        }
    }

    mat_ext
}