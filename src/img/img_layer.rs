use super::*;

#[derive(Clone)]
pub struct ImgLayer {
    mat: Matrix2D,
    channel: ImgChannel,
}

#[allow(unused)]
impl ImgLayer {
    pub fn new(mat: Matrix2D, channel: ImgChannel) -> Self {
        ImgLayer { mat, channel }
    }

    pub fn channel(&self) -> ImgChannel {
        self.channel
    }

    pub fn w(&self) -> usize {
        self.mat.w()
    }
    pub fn h(&self) -> usize {
        self.mat.h()
    }

    pub fn matrix(&self) -> &Matrix2D {
        &self.mat
    }

    pub fn matrix_mut(&mut self) -> &mut Matrix2D {
        &mut self.mat
    }

    pub fn get_area(&self) -> PixelsArea {
        PixelsArea::size_of(self.matrix())
    }
}

impl Index<usize> for ImgLayer {
    type Output = f64;

    fn index(&self, index: usize) -> &Self::Output {
        &self.mat[index]
    }
}

impl Index<PixelPos> for ImgLayer {
    type Output = f64;

    fn index(&self, index: PixelPos) -> &Self::Output {
        &self.mat[index]
    }
}

impl IndexMut<PixelPos> for ImgLayer {
    fn index_mut(&mut self, index: PixelPos) -> &mut Self::Output {
        &mut self.mat[index]
    }
}

#[cfg(test)]
mod tests {
    use super::ImgLayer;
    use crate::img::{filter::filter_option::ImgChannel, Matrix2D, PixelPos, PixelsArea};

    #[test]
    fn new_ctor() {
        const W: usize = 3;
        const H: usize = 4;

        let mat = Matrix2D::empty_with_size(W, H);
        let mat_copy = mat.clone();
        let mut layer = ImgLayer::new(mat, ImgChannel::A);

        assert_eq!(layer.channel(), ImgChannel::A);
        assert_eq!(layer.w(), W);
        assert_eq!(layer.h(), H);
        assert!(mat_copy.has_the_same_values_as(&layer.matrix()));
        assert!(mat_copy.has_the_same_values_as(&layer.matrix_mut()));

        let area: PixelsArea = layer.get_area();
        assert_eq!(area.top_left(), PixelPos::new(0, 0));
        assert_eq!(area.bottom_right(), PixelPos::new(H - 1, W - 1));
    }
}
