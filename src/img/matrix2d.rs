use super::*;
use crate::processing::{ExecutorHandle, TaskStop};

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
        Matrix2D {
            width,
            height,
            pixels,
        }
    }

    pub fn empty_size_of(other: &Matrix2D) -> Self {
        let mut pixels = Vec::<f64>::new();
        pixels.resize(other.w() * other.h(), 0_f64);
        Matrix2D {
            width: other.w(),
            height: other.h(),
            pixels,
        }
    }

    pub fn generate<'area, Tr: FnMut(PixelPos) -> f64>(
        iter: impl PixelsAreaIter<'area, Item = PixelPos>,
        mut tr: Tr,
    ) -> Result<Self, TaskStop> {
        let mut mat = Self::empty_with_size(iter.area().w(), iter.area().h());

        for pos in iter {
            mat[pos] = tr(pos);
        }

        Ok(mat)
    }

    pub fn w(&self) -> usize {
        self.width
    }
    pub fn h(&self) -> usize {
        self.height
    }

    pub fn size_vec(&self) -> PixelPos {
        PixelPos::new(self.h(), self.w())
    }

    pub fn max_col(&self) -> usize {
        self.width - 1
    }
    pub fn max_row(&self) -> usize {
        self.height - 1
    }

    pub fn fits(&self, pos: PixelPos) -> bool {
        pos.col <= self.max_col() && pos.row <= self.max_row()
    }

    pub fn area(&self) -> PixelsArea {
        PixelsArea::size_of(self)
    }

    pub fn scalar_transform_self_area<'area, Tr, Iter>(
        &mut self,
		iter: Iter,
        tr: Tr,
    ) -> Result<(), TaskStop>
	where
		Tr: Fn(&mut f64, PixelPos) -> (),
		Iter: PixelsAreaIter<'area>
	{
        for pos in iter {
			tr(&mut self[pos], pos);
        }

		Ok(())
    }

    pub fn pixels<'own>(&'own self) -> &'own Vec<f64> {
        &self.pixels
    }

    pub fn get_max(&self, executor_handle: &mut ExecutorHandle) -> Result<f64, TaskStop> {
        let mut max = self.pixels[0];

        for row in 0..self.h() {
            for col in 0..self.w() {
                let pos = PixelPos::new(row, col);
                let val = self[pos];
                if val > max {
                    max = val;
                }
            }
            executor_handle.complete_action()?
        }

        Ok(max)
    }

    pub fn extended_for_window_filter<F: WindowFilter>(&self, filter: &F) -> Matrix2D {
        let left = filter.w() / 2;
        let top = filter.h() / 2;
        let right = left;
        let bottom = top;

        self.extended(filter.get_extend_value(), left, top, right, bottom)
    }

    pub fn extended(
        &self,
        with: ExtendValue,
        left: usize,
        top: usize,
        right: usize,
        bottom: usize,
    ) -> Matrix2D {
        let mut mat_ext =
            Matrix2D::empty_with_size(left + self.w() + right, top + self.h() + bottom);

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
                let top_left_area = PixelsArea::with_size(top, left);
                match with {
                    ExtendValue::Closest => mat_ext.set_rect(top_left_area, self[origin]),
                    ExtendValue::Given(val) => mat_ext.set_rect(top_left_area, val),
                }
            }
            // top middle
            let top_moddle_area = PixelsArea::with_size(top, self.w()).with_pos(0, left);
            match with {
                ExtendValue::Closest => {
                    for pos in top_moddle_area.iter_pixels() {
                        mat_ext[pos] = self[pos.with_row(0) - margin_left];
                    }
                }
                ExtendValue::Given(val) => mat_ext.set_rect(top_moddle_area, val),
            }
            // top right
            if right > 0 {
                let top_right_area = PixelsArea::with_size(top, right).with_pos(0, left + self.w());
                match with {
                    ExtendValue::Closest => {
                        mat_ext.set_rect(top_right_area, self[PixelPos::new(0, self.w() - 1)])
                    }
                    ExtendValue::Given(val) => mat_ext.set_rect(top_right_area, val),
                }
            }
        }
        // ------------------------------------ middle ------------------------------------
        // middle left
        if left > 0 {
            let middle_left_area = PixelsArea::with_size(self.h(), left).with_pos(top, 0);
            match with {
                ExtendValue::Closest => {
                    for pos in middle_left_area.iter_pixels() {
                        mat_ext[pos] = self[pos.with_col(0) - margin_top];
                    }
                }
                ExtendValue::Given(val) => mat_ext.set_rect(middle_left_area, val),
            }
        }
        // middle middle
        let middle_middle_area = PixelsArea::size_of(self).with_pos(top, left);
        for pos in middle_middle_area.iter_pixels() {
            mat_ext[pos] = self[pos - PixelPos::new(top, left)];
        }
        // middle right
        if right > 0 {
            let middle_right_area =
                PixelsArea::with_size(self.h(), right).with_pos(top, left + self.w());
            match with {
                ExtendValue::Closest => {
                    for pos in middle_right_area.iter_pixels() {
                        mat_ext[pos] = self[pos.with_col(self.w() - 1) - margin_top];
                    }
                }
                ExtendValue::Given(val) => mat_ext.set_rect(middle_right_area, val),
            }
        }

        // ------------------------------------ bottom ------------------------------------
        if bottom > 0 {
            // bottom left
            if left > 0 {
                let bottom_left_area =
                    PixelsArea::with_size(bottom, left).with_pos(top + self.h(), 0);
                match with {
                    ExtendValue::Closest => {
                        mat_ext.set_rect(bottom_left_area, self[PixelPos::new(self.h() - 1, 0)])
                    }
                    ExtendValue::Given(val) => mat_ext.set_rect(bottom_left_area, val),
                }
            }
            // bottom middle
            let bottom_middle_area =
                PixelsArea::with_size(bottom, self.w()).with_pos(top + self.h(), left);
            match with {
                ExtendValue::Closest => {
                    for pos in bottom_middle_area.iter_pixels() {
                        mat_ext[pos] = self[pos.with_row(self.h() - 1) - margin_left];
                    }
                }
                ExtendValue::Given(val) => mat_ext.set_rect(bottom_middle_area, val),
            }
            // bottom right
            if right > 0 {
                let bottom_right_area =
                    PixelsArea::with_size(bottom, right).with_pos(top + self.h(), left + self.w());
                match with {
                    ExtendValue::Closest => {
                        mat_ext.set_rect(bottom_right_area, self[self.size_vec() - PixelPos::one()])
                    }
                    ExtendValue::Given(val) => mat_ext.set_rect(bottom_right_area, val),
                }
            }
        }

        mat_ext
    }

    pub fn set_rect(&mut self, area: PixelsArea, value: f64) {
        for pos in area.iter_pixels() {
            self[pos] = value;
        }
    }

    pub fn has_the_same_values_as(&self, other: &Matrix2D) -> bool {
        if self.w() != other.w() {
            return false;
        }
        if self.h() != other.h() {
            return false;
        }
        self.pixels()
            .iter()
            .enumerate()
            .all(|(ind, p)| (other[ind] - *p).abs() <= std::f64::EPSILON)
    }
}

impl Index<PixelPos> for Matrix2D {
    type Output = f64;

    fn index(&self, index: PixelPos) -> &Self::Output {
        if !self.fits(index) {
            panic!(
                "pos is {:?} which is doesn't fit into {}, {}",
                index,
                self.max_col(),
                self.max_row()
            );
        }
        &self.pixels[index.row * self.width + index.col]
    }
}

impl IndexMut<PixelPos> for Matrix2D {
    fn index_mut(&mut self, index: PixelPos) -> &mut Self::Output {
        if !self.fits(index) {
            panic!(
                "pos is {:?} which is doesn't fit into {}, {}",
                index,
                self.max_col(),
                self.max_row()
            );
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

#[cfg(test)]
mod tests {
    use super::Matrix2D;
    use crate::img::{PixelPos, PixelsArea};

    #[test]
    fn ctor_empty_with_size() {
        let m = Matrix2D::empty_with_size(2, 3);

        assert_eq!(m.w(), 2);
        assert_eq!(m.h(), 3);

        for v in m.pixels {
            assert!(v.abs() < 1e-14);
        }
    }

    #[test]
    fn ctor_empty_size_of() {
        let m = Matrix2D::empty_with_size(2, 3);
        let m2 = Matrix2D::empty_size_of(&m);

        assert_eq!(m2.w(), m.w());
        assert_eq!(m2.h(), m.h());

        for v in m2.pixels {
            assert!(v.abs() < 1e-14);
        }
    }

    #[test]
    fn ctor_generate() {
        let mut val: f64 = -1.0;
        let gen = move |_pos: PixelPos| -> f64 {
            val += 1.0;
            val
        };

        let area = PixelsArea::with_size(3, 2);

        let m = Matrix2D::generate(area.iter_pixels(), gen).unwrap();

        let positions: [PixelPos; 6] = [
            PixelPos::new(0, 0),
            PixelPos::new(0, 1),
            PixelPos::new(1, 0),
            PixelPos::new(1, 1),
            PixelPos::new(2, 0),
            PixelPos::new(2, 1),
        ];

        for i in 0..positions.len() {
            let pos = positions[i];

            assert_eq!(m[pos], m[i]);
            assert_eq!(m[i], i as f64);
        }
    }

	#[test]
	fn w_h_size_vec_max_col_max_row() {
		const W: usize = 2;
		const H: usize = 3;
        let m = Matrix2D::empty_with_size(W, H);
		assert_eq!(m.w(), W);
		assert_eq!(m.h(), H);
		assert_eq!(m.size_vec(), PixelPos::new(H, W));
		assert_eq!(m.max_col(), W - 1);
		assert_eq!(m.max_row(), H - 1);
	}

	#[test]
	fn fits() {
		const W: usize = 2;
		const H: usize = 3;
        let m = Matrix2D::empty_with_size(W, H);

		for pos in PixelsArea::with_size(4, 5).iter_pixels() {
			let fits: bool = pos.col < W && pos.row < H;
			assert_eq!(m.fits(pos), fits);
		}
	}

	#[test]
	fn area() {
		const W: usize = 2;
		const H: usize = 3;
        let m = Matrix2D::empty_with_size(W, H);
		assert_eq!(
			m.area(), 
			PixelsArea::new(
				PixelPos::new(0, 0), 
				PixelPos::new(H - 1, W - 1)));
	}

	#[test]
	fn scalar_transform_self_area() {
		const W: usize = 2;
		const H: usize = 3;
        let mut m = Matrix2D::empty_with_size(W, H);
		let mut val = 0_f64;
		for pos in m.area().iter_pixels() {
			m[pos] = val;
			val += 1.0;
		}

		let mut m2 = m.clone();
		m2.scalar_transform_self_area(
			m.area().iter_pixels(), 
			|val, _pos| {
				*val *= 2.0
			}).unwrap();
		
		assert!(m.pixels().iter().enumerate()
			.all(|(ind, p)| (m2[ind] - *p * 2.0).abs() <= std::f64::EPSILON));
	}

	#[test]
	fn get_max() {
		const W: usize = 2;
		const H: usize = 3;
		let pixels: Vec<f64> = (1..=W * H).map(|v| v as f64).collect();
		let _mat = Matrix2D {
			width: W,
			height: H,
			pixels,
		};
		unimplemented!()
		// assert_eq!(mat.get_max(executor_handle))
	}

	#[test]
	fn extended_for_window_filter() {
		unimplemented!()
	}

	#[test]
	fn extended() {
		unimplemented!()
	}

	#[test]
	fn set_rect() {
		unimplemented!()
	}

	#[test]
	fn has_the_same_values_as() {
		unimplemented!()
	}
}
