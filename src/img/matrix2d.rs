use super::*;


#[derive(Clone)]
pub struct Matrix2D {
    width: usize,
    height: usize,
    pixels: Vec<f64>,
}

#[allow(unused)]
impl Matrix2D {
    pub fn empty_with_size(width: usize, height: usize) -> Self {
        let mut pixels = Vec::<f64>::new();
        pixels.resize(width * height, 0_f64);        
        Matrix2D { width, height, pixels }
    }

    pub fn empty_size_of(other: &Matrix2D) -> Self {
        let mut pixels = Vec::<f64>::new();
        pixels.resize(other.w() * other.h(), 0_f64);        
        Matrix2D { width: other.w(), height: other.h(), pixels }
    }

    pub fn w(&self) -> usize { self.width }
    pub fn h(&self) -> usize { self.height }

    pub fn size_vec(&self) -> PixelPos { PixelPos::new(self.h(), self.w()) }

    pub fn max_col(&self) -> usize { self.width - 1 }
    pub fn max_row(&self) -> usize { self.height - 1 }

    pub fn fits(&self, pos: PixelPos) -> bool {
        pos.col <= self.max_col() && pos.row <= self.max_row()
    }

    pub fn get_pixels_iter(&self) -> PixelsIterator {
        PixelsIterator::for_full_image(self)
    }

    pub fn get_pixels_area_iter(&self, tl: PixelPos, br_excluded: PixelPos) -> PixelsIterator {
        assert!(self.fits(tl));
        assert!(br_excluded.row > 0);
        assert!(br_excluded.col > 0);
        assert!(self.fits(br_excluded - PixelPos::one()));
        PixelsIterator::for_rect_area( tl, br_excluded)
    }

    pub fn scalar_transform_into<Tr: Fn(&Matrix2D, PixelPos) -> f64>(&self, area: PixelsArea, tr: Tr, dest_matrix: &mut Matrix2D) {
        for pos in self.get_pixels_area_iter(area.top_left(), area.bottom_right()) {
            dest_matrix[pos] = tr(dest_matrix, pos);
        }
    }

    pub fn scalar_transform<Tr: Fn(&Matrix2D, PixelPos) -> f64>(&self, area: PixelsArea, tr: Tr) -> Self {
        let mut transformed = Self::empty_size_of(self);
        self.scalar_transform_into(area, tr, &mut transformed);
        transformed
    }

    pub fn get_drawable_copy(&self) -> Result<image::RgbImage, MyError> { 
        let im_rgb = image::RgbImage::new(
            self.pixels.iter().map(|v| *v as u8).collect::<Vec<u8>>().as_slice(), 
            self.width as i32, self.height as i32,  ColorDepth::L8)?;
        Ok(im_rgb)
    }

    pub fn pixels<'own>(&'own self) -> &'own Vec<f64> {
        &self.pixels
    }

    pub fn extended_for_window_filter<F: WindowFilter>(&self, filter: &F) -> Matrix2D {
        let left = filter.w() / 2;
        let top = filter.h() / 2;
        let right = left;
        let bottom = top;
    
        self.extended( filter.get_extend_value(), left, top, right, bottom)
    }
    
    pub fn extended(
        &self, 
        with: ExtendValue, 
        left: usize, top: usize, right: usize, bottom: usize
    ) -> Matrix2D {
        let mut mat_ext = Matrix2D::empty_with_size(left + self.w() + right, top + self.h() + bottom);
    
        let origin = PixelPos::new(0, 0);
    
        let margin_left = PixelPos::new(0, left);
        let margin_top = PixelPos::new(top, 0);
        let margin_right = PixelPos::new(0, right);
        let margin_bottom = PixelPos::new(bottom, 0);
    
        let rect_left = PixelPos::new(self.h(), left);
        let rect_top = PixelPos::new(top, self.w());
        let rect_right = PixelPos::new(self.h(), right);
        let rect_bottom = PixelPos::new(bottom, self.w());
    
        let mat_size = PixelPos::new(self.h(), self.w());
    
        // ------------------------------------ top ------------------------------------
        if top > 0 {
            // top left
            if left > 0 {
                let tl = origin;
                let br_excluded = tl + margin_left + margin_top;
                match with {
                    ExtendValue::Closest => mat_ext.set_rect(tl, br_excluded, self[origin]),
                    ExtendValue::Given(val) => mat_ext.set_rect(tl, br_excluded, val),
                }
            }
            // top middle
            let tl = margin_left;
            let br_excluded = tl + rect_top;
            match with {
                ExtendValue::Closest => {
                    for pos in mat_ext.get_pixels_area_iter(tl, br_excluded) {
                        mat_ext[pos] = self[pos.with_row(0) - margin_left];
                    }  
                },
                ExtendValue::Given(val) => mat_ext.set_rect(tl, br_excluded, val),
            }    
            // top right
            if right > 0 { 
                let tl = margin_left + rect_top.col_vec();
                let br_excluded = tl + margin_right + margin_top;
                match with {
                    ExtendValue::Closest => mat_ext.set_rect(tl, br_excluded, self[PixelPos::new(0, self.w() - 1)]),
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
                    for pos in mat_ext.get_pixels_area_iter(tl, br_excluded) {
                        mat_ext[pos] = self[pos.with_col(0) - margin_top];
                    }
                },
                ExtendValue::Given(val) => mat_ext.set_rect(tl, br_excluded, val),
            }
        }
        // middle middle     
        let tl = margin_left + margin_top;
        let br_excluded = tl + mat_size;               
        for pos in mat_ext.get_pixels_area_iter(tl, br_excluded) {
            mat_ext[pos] = self[pos - tl];
        }    
        // middle right
        if right > 0 { 
            let tl = margin_left + rect_top;
            let br_excluded = tl + rect_right;
            match with {
                ExtendValue::Closest => {          
                    for pos in mat_ext.get_pixels_area_iter(tl, br_excluded) {
                        mat_ext[pos] = self[pos.with_col(self.w() - 1) - margin_top];
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
                    ExtendValue::Closest => mat_ext.set_rect(tl, br_excluded, self[PixelPos::new(self.h() - 1, 0)]),
                    ExtendValue::Given(val) => mat_ext.set_rect(tl, br_excluded, val),
                }
            }
            // bottom middle
            let tl = margin_top + rect_left;
            let br_excluded = tl + rect_bottom;
            match with {
                ExtendValue::Closest => {   
                    for pos in mat_ext.get_pixels_area_iter(tl, br_excluded) {
                        mat_ext[pos] = self[pos.with_row(self.h() - 1) - margin_left];
                    } 
                },
                ExtendValue::Given(val) => mat_ext.set_rect(tl, br_excluded, val),
            }       
            // bottom right
            if right > 0 {
                let tl = margin_left + margin_top + self.size_vec();
                let br_excluded = tl + margin_right + margin_bottom;
                match with {
                    ExtendValue::Closest => mat_ext.set_rect(tl, br_excluded, self[self.size_vec() - PixelPos::one()]),
                    ExtendValue::Given(val) => mat_ext.set_rect(tl, br_excluded, val),
                }
            }
        }
    
        mat_ext
    }

    pub fn set_rect(&mut self, tl: PixelPos, br_excluded: PixelPos, value: f64) -> () {
        for pos in self.get_pixels_area_iter(tl, br_excluded) {
            self[pos] = value;
        }
    }
}

impl Index<PixelPos> for Matrix2D {
    type Output = f64;

    fn index(&self, index: PixelPos) -> &Self::Output {
        if !self.fits(index) {
            panic!("pos is {:?} which is doesn't fit into {}, {}", index, self.max_col(), self.max_row());
        }
        &self.pixels[index.row * self.width + index.col]
    }
}

impl IndexMut<PixelPos> for Matrix2D {
    fn index_mut(&mut self, index: PixelPos) -> &mut Self::Output {
        if !self.fits(index) {
            panic!("pos is {:?} which is doesn't fit into {}, {}", index, self.max_col(), self.max_row());
        }
        &mut self.pixels[index.row * self.width + index.col]
    }
}


impl Index<usize> for Matrix2D {
    type Output = f64;

    fn index(&self, index: usize) -> &Self::Output {
        &self.pixels[index]
    }
}

impl IndexMut<usize> for Matrix2D {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.pixels[index]
    }
}
