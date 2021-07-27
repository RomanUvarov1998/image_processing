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
        &self.mat.pixels()[index]
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
