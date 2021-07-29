use super::*;

#[derive(Clone)]
pub struct Img {
    width: usize,
    height: usize,
    layers: Vec<ImgLayer>,
    color_depth: ColorDepth,
}

impl Img {
    pub fn from_layers(layers: Vec<ImgLayer>, color_depth: ColorDepth) -> Self {
        let assert_layer_exists = |ch: ImgChannel| {
            layers
                .iter()
                .find(|l| l.channel() == ch)
                .expect(&format!("couldn't find layer {:?}", ch));
        };

        match color_depth {
            ColorDepth::L8 => {
                assert_eq!(layers.len(), 1, "there must be 1 layer: L");
                assert_layer_exists(ImgChannel::L);
            }
            ColorDepth::La8 => {
                assert_eq!(layers.len(), 2, "there must be 2 layers: L, A");

                assert_layer_exists(ImgChannel::L);
                assert_layer_exists(ImgChannel::A);
            }
            ColorDepth::Rgb8 => {
                assert_eq!(layers.len(), 3, "there must be 3 layers: R, G, B");

                assert_layer_exists(ImgChannel::R);
                assert_layer_exists(ImgChannel::G);
                assert_layer_exists(ImgChannel::B);
            }
            ColorDepth::Rgba8 => {
                assert_eq!(layers.len(), 4, "there must be 4 layers: R, G, B, A");

                assert_layer_exists(ImgChannel::R);
                assert_layer_exists(ImgChannel::G);
                assert_layer_exists(ImgChannel::B);
                assert_layer_exists(ImgChannel::A);
            }
        }

        let (width, height) = (layers[0].w(), layers[0].h());
        assert!(
            layers.iter().all(|l| l.w() == width && l.h() == height),
            "all layers must be the same size"
        );

        Img {
            width,
            height,
            layers,
            color_depth,
        }
    }

    pub fn empty_with_size(width: usize, height: usize, color_depth: ColorDepth) -> Self {
        let mut layers = Vec::<ImgLayer>::new();

        match color_depth {
            ColorDepth::L8 => {
                layers.push(ImgLayer::new(
                    Matrix2D::empty_with_size(width, height),
                    ImgChannel::L,
                ));
            }
            ColorDepth::La8 => {
                layers.push(ImgLayer::new(
                    Matrix2D::empty_with_size(width, height),
                    ImgChannel::L,
                ));
                layers.push(ImgLayer::new(
                    Matrix2D::empty_with_size(width, height),
                    ImgChannel::A,
                ));
            }
            ColorDepth::Rgb8 => {
                layers.push(ImgLayer::new(
                    Matrix2D::empty_with_size(width, height),
                    ImgChannel::R,
                ));
                layers.push(ImgLayer::new(
                    Matrix2D::empty_with_size(width, height),
                    ImgChannel::G,
                ));
                layers.push(ImgLayer::new(
                    Matrix2D::empty_with_size(width, height),
                    ImgChannel::B,
                ));
            }
            ColorDepth::Rgba8 => {
                layers.push(ImgLayer::new(
                    Matrix2D::empty_with_size(width, height),
                    ImgChannel::R,
                ));
                layers.push(ImgLayer::new(
                    Matrix2D::empty_with_size(width, height),
                    ImgChannel::G,
                ));
                layers.push(ImgLayer::new(
                    Matrix2D::empty_with_size(width, height),
                    ImgChannel::B,
                ));
                layers.push(ImgLayer::new(
                    Matrix2D::empty_with_size(width, height),
                    ImgChannel::A,
                ));
            }
        }

        Img {
            width,
            height,
            layers,
            color_depth,
        }
    }

    pub fn empty_size_of(other: &Img) -> Self {
        Self::empty_with_size(other.width, other.height, other.color_depth)
    }

    pub fn from_pixels(
        width: usize,
        height: usize,
        color_depth: ColorDepth,
        pixels: Vec<u8>,
    ) -> Self {
        let pixels_f: Vec<f64> = pixels.iter().map(|v| *v as f64).collect();

        let layers_count = color_depth as u8 as usize;
        assert_eq!(
            width * height * layers_count,
            pixels.len(),
            "values count doesn't satisfy color depth: {} pixels for {:?}x{}x{}",
            pixels_f.len(),
            color_depth,
            height,
            width
        );

        let mut img = Img::empty_with_size(width, height, color_depth);

        for pixel_num in 0..pixels_f.len() {
            let layer_num = pixel_num % layers_count;
            let layer_pixel_num = pixel_num / layers_count;
            img.layer_mut(layer_num).matrix_mut()[layer_pixel_num] = pixels_f[pixel_num];
        }

        img
    }

    pub fn w(&self) -> usize {
        self.width
    }
    pub fn h(&self) -> usize {
        self.height
    }
    pub fn d(&self) -> usize {
        self.color_depth as u8 as usize
    }
    pub fn color_depth(&self) -> ColorDepth {
        self.color_depth
    }

    pub fn get_description(&self) -> String {
        format!(
            "Изображение {} (строк) x {} (столбцов) x {} (каналов)",
            self.h(),
            self.w(),
            self.d()
        )
    }

    pub fn layers<'own>(&'own self) -> &'own Vec<ImgLayer> {
        &self.layers
    }
    pub fn layers_mut<'own>(&'own mut self) -> &'own mut Vec<ImgLayer> {
        &mut self.layers
    }
    pub fn layer_mut<'own>(&'own mut self, ind: usize) -> &'own mut ImgLayer {
        &mut self.layers[ind]
    }
    pub fn layer_by_channel<'own>(&'own self, ch: ImgChannel) -> Option<&'own ImgLayer> {
        self.layers.iter().find(|l| l.channel() == ch)
    }
    pub fn layer<'own>(&'own self, ind: usize) -> &'own ImgLayer {
        &self.layers[ind]
    }

    pub fn get_cropped_copy(&self, area: PixelsArea) -> Img {
        assert!(
            area.is_inside_of(&self.get_area()),
            "crop area must be inside of the img area"
        );

        let mut img = Img::empty_with_size(area.w(), area.h(), self.color_depth());

        for pos in area.iter_pixels() {
            for ch_num in 0..self.d() {
                img.layer_mut(ch_num)[pos - area.top_left()] = self.layer(ch_num)[pos];
            }
        }

        img
    }

    pub fn get_area(&self) -> PixelsArea {
        PixelsArea::with_size(self.h(), self.w())
    }

    pub fn get_drawable_copy(&self) -> image::RgbImage {
        let mut all_pixels = Vec::<u8>::with_capacity(self.w() * self.h() * self.d());

        let layer_length = self.w() * self.h();
        for pix_num in 0..layer_length {
            for layer in self.layers().iter() {
                all_pixels.push(layer[pix_num] as u8);
            }
        }

        let im_rgb = image::RgbImage::new(
            all_pixels.as_slice(),
            self.width as i32,
            self.height as i32,
            self.color_depth,
        )
        .unwrap();

        im_rgb
    }

    pub fn extended(
        &self,
        with: ExtendValue,
        left: usize,
        top: usize,
        right: usize,
        bottom: usize,
    ) -> Img {
        let mut ext_layers = Vec::<ImgLayer>::with_capacity(self.d());

        for layer in self.layers() {
            let ext_layer = match layer.channel() {
                ImgChannel::A => {
                    let mut ext_mat = Matrix2D::empty_with_size(
                        left + layer.w() + right,
                        top + layer.h() + bottom,
                    );
                    ext_mat.set_rect(ext_mat.area(), 255_f64);
                    ImgLayer::new(ext_mat, layer.channel())
                }
                _ => {
                    let ext_mat = layer.matrix().extended(with, left, top, right, bottom);
                    ImgLayer::new(ext_mat, layer.channel())
                }
            };

            ext_layers.push(ext_layer);
        }

        Img::from_layers(ext_layers, self.color_depth())
    }

    pub fn try_save(&self, path: &str) -> Result<(), MyError> {
        use jpeg_encoder::{ColorType, Encoder};

        let (pixels, color_type): (Vec<u8>, ColorType) = match self.color_depth() {
            ColorDepth::L8 | ColorDepth::La8 => {
                let vals: Vec<u8> = self
                    .layer(0)
                    .matrix()
                    .pixels()
                    .iter()
                    .map(|p| *p as u8)
                    .collect();

                (vals, ColorType::Luma)
            }
            ColorDepth::Rgb8 | ColorDepth::Rgba8 => {
                let mut vals = Vec::<u8>::with_capacity(self.w() * self.h() * 3);

                let r = self.layer(0).matrix().pixels();
                let g = self.layer(1).matrix().pixels();
                let b = self.layer(2).matrix().pixels();

                for pix_num in 0..self.w() * self.h() {
                    vals.push(r[pix_num] as u8);
                    vals.push(g[pix_num] as u8);
                    vals.push(b[pix_num] as u8);
                }

                assert_eq!(vals.len(), self.w() * self.h() * 3);

                (vals, ColorType::Rgb)
            }
        };

        let encoder = Encoder::new_file(path, 100)?;
        encoder.encode(&pixels, self.w() as u16, self.h() as u16, color_type)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Img;
    use crate::img::{filter::filter_option::ImgChannel, ImgLayer, Matrix2D, PixelPos, PixelsArea};
    use fltk::{enums::ColorDepth, prelude::ImageExt};

    #[allow(non_snake_case)]
    #[test]
    #[should_panic(expected = "there must be 1 layer: L")]
    fn from_layers_ctor_panics_if_0_layers_for_L8() {
        let _img = Img::from_layers(Vec::new(), ColorDepth::L8);
    }

    #[allow(non_snake_case)]
    #[test]
    #[should_panic(expected = "there must be 1 layer: L")]
    fn from_layers_ctor_panics_if_2_layers_for_L8() {
        let _img = Img::from_layers(
            vec![
                create_layer(3, 3, ImgChannel::A),
                create_layer(3, 3, ImgChannel::A),
            ],
            ColorDepth::L8,
        );
    }

    #[allow(non_snake_case)]
    #[test]
    #[should_panic(expected = "there must be 2 layers: L, A")]
    fn from_layers_ctor_panics_if_0_layers_for_La8() {
        let _img = Img::from_layers(Vec::new(), ColorDepth::La8);
    }

    #[allow(non_snake_case)]
    #[test]
    #[should_panic(expected = "there must be 2 layers: L, A")]
    fn from_layers_ctor_panics_if_3_layers_for_La8() {
        let _img = Img::from_layers(
            vec![
                create_layer(3, 3, ImgChannel::A),
                create_layer(3, 3, ImgChannel::A),
                create_layer(3, 3, ImgChannel::A),
            ],
            ColorDepth::La8,
        );
    }

    #[allow(non_snake_case)]
    #[test]
    #[should_panic(expected = "there must be 3 layers: R, G, B")]
    fn from_layers_ctor_panics_if_0_layers_for_Rgb8() {
        let _img = Img::from_layers(Vec::new(), ColorDepth::Rgb8);
    }

    #[allow(non_snake_case)]
    #[test]
    #[should_panic(expected = "there must be 3 layers: R, G, B")]
    fn from_layers_ctor_panics_if_4_layers_for_Rgb8() {
        let _img = Img::from_layers(
            vec![
                create_layer(3, 3, ImgChannel::A),
                create_layer(3, 3, ImgChannel::A),
                create_layer(3, 3, ImgChannel::A),
                create_layer(3, 3, ImgChannel::A),
            ],
            ColorDepth::Rgb8,
        );
    }

    #[allow(non_snake_case)]
    #[test]
    #[should_panic(expected = "there must be 4 layers: R, G, B, A")]
    fn from_layers_ctor_panics_if_0_layers_for_Rgba8() {
        let _img = Img::from_layers(Vec::new(), ColorDepth::Rgba8);
    }

    #[allow(non_snake_case)]
    #[test]
    #[should_panic(expected = "there must be 4 layers: R, G, B, A")]
    fn from_layers_ctor_panics_if_5_layers_for_Rgba8() {
        let _img = Img::from_layers(
            vec![
                create_layer(3, 3, ImgChannel::A),
                create_layer(3, 3, ImgChannel::A),
                create_layer(3, 3, ImgChannel::A),
                create_layer(3, 3, ImgChannel::A),
                create_layer(3, 3, ImgChannel::A),
            ],
            ColorDepth::Rgba8,
        );
    }

    #[allow(non_snake_case)]
    #[test]
    #[should_panic(expected = "couldn't find layer")]
    fn from_layers_ctor_panics_if_layers_set_is_wrong_L8() {
        let _img = Img::from_layers(vec![create_layer(3, 3, ImgChannel::R)], ColorDepth::L8);
    }

    #[allow(non_snake_case)]
    #[test]
    #[should_panic(expected = "couldn't find layer")]
    fn from_layers_ctor_panics_if_layers_set_is_wrong_La8() {
        let _img = Img::from_layers(
            vec![
                create_layer(3, 3, ImgChannel::R),
                create_layer(3, 3, ImgChannel::G),
            ],
            ColorDepth::La8,
        );
    }

    #[allow(non_snake_case)]
    #[test]
    #[should_panic(expected = "couldn't find layer")]
    fn from_layers_ctor_panics_if_layers_set_is_wrong_Rgb8() {
        let _img = Img::from_layers(
            vec![
                create_layer(3, 3, ImgChannel::R),
                create_layer(3, 3, ImgChannel::G),
                create_layer(3, 3, ImgChannel::A),
            ],
            ColorDepth::Rgb8,
        );
    }

    #[allow(non_snake_case)]
    #[test]
    #[should_panic(expected = "couldn't find layer")]
    fn from_layers_ctor_panics_if_layers_set_is_wrong_Rgba8() {
        let _img = Img::from_layers(
            vec![
                create_layer(3, 3, ImgChannel::R),
                create_layer(3, 3, ImgChannel::L),
                create_layer(3, 3, ImgChannel::G),
                create_layer(3, 3, ImgChannel::A),
            ],
            ColorDepth::Rgba8,
        );
    }

    #[allow(non_snake_case)]
    #[test]
    #[should_panic(expected = "all layers must be the same size")]
    fn from_layers_ctor_panics_if_layers_not_of_one_size_La8() {
        let _img = Img::from_layers(
            vec![
                create_layer(3, 3, ImgChannel::L),
                create_layer(3, 2, ImgChannel::A),
            ],
            ColorDepth::La8,
        );
    }

    #[allow(non_snake_case)]
    #[test]
    #[should_panic(expected = "all layers must be the same size")]
    fn from_layers_ctor_panics_if_layers_not_of_one_size_Rgb8() {
        let _img = Img::from_layers(
            vec![
                create_layer(1, 3, ImgChannel::R),
                create_layer(3, 3, ImgChannel::G),
                create_layer(3, 3, ImgChannel::B),
            ],
            ColorDepth::Rgb8,
        );
    }

    #[allow(non_snake_case)]
    #[test]
    #[should_panic(expected = "all layers must be the same size")]
    fn from_layers_ctor_panics_if_layers_not_of_one_size_Rgba8() {
        let _img = Img::from_layers(
            vec![
                create_layer(1, 3, ImgChannel::R),
                create_layer(3, 3, ImgChannel::G),
                create_layer(3, 3, ImgChannel::B),
                create_layer(3, 32, ImgChannel::A),
            ],
            ColorDepth::Rgba8,
        );
    }

    fn create_layer(w: usize, h: usize, ch: ImgChannel) -> ImgLayer {
        ImgLayer::new(Matrix2D::empty_with_size(w, h), ch)
    }

    #[test]
    fn empty_with_size_ctor() {
        let img = Img::empty_with_size(3, 3, ColorDepth::L8);
        assert_eq!(img.color_depth(), ColorDepth::L8);
        assert_eq!(img.w(), 3);
        assert_eq!(img.h(), 3);
        assert_eq!(img.layers().len(), 1);
        assert_all_pixels_are_0(img.layer(0).matrix());
        assert_eq!(img.layer(0).channel(), ImgChannel::L);

        let img = Img::empty_with_size(3, 3, ColorDepth::La8);
        assert_eq!(img.color_depth(), ColorDepth::La8);
        assert_eq!(img.w(), 3);
        assert_eq!(img.h(), 3);
        assert_eq!(img.layers().len(), 2);
        assert_all_pixels_are_0(img.layer(0).matrix());
        // assert_all_pixels_are_0(img.layer(1).matrix());
        assert_eq!(img.layer(0).channel(), ImgChannel::L);
        assert_eq!(img.layer(1).channel(), ImgChannel::A);

        let img = Img::empty_with_size(3, 3, ColorDepth::Rgb8);
        assert_eq!(img.color_depth(), ColorDepth::Rgb8);
        assert_eq!(img.w(), 3);
        assert_eq!(img.h(), 3);
        assert_eq!(img.layers().len(), 3);
        assert_all_pixels_are_0(img.layer(0).matrix());
        assert_all_pixels_are_0(img.layer(1).matrix());
        assert_all_pixels_are_0(img.layer(2).matrix());
        assert_eq!(img.layer(0).channel(), ImgChannel::R);
        assert_eq!(img.layer(1).channel(), ImgChannel::G);
        assert_eq!(img.layer(2).channel(), ImgChannel::B);

        let img = Img::empty_with_size(3, 3, ColorDepth::Rgba8);
        assert_eq!(img.color_depth(), ColorDepth::Rgba8);
        assert_eq!(img.w(), 3);
        assert_eq!(img.h(), 3);
        assert_eq!(img.layers().len(), 4);
        assert_all_pixels_are_0(img.layer(0).matrix());
        assert_all_pixels_are_0(img.layer(1).matrix());
        assert_all_pixels_are_0(img.layer(2).matrix());
        // assert_all_pixels_are_0(img.layer(4).matrix());
        assert_eq!(img.layer(0).channel(), ImgChannel::R);
        assert_eq!(img.layer(1).channel(), ImgChannel::G);
        assert_eq!(img.layer(2).channel(), ImgChannel::B);
        assert_eq!(img.layer(3).channel(), ImgChannel::A);
    }

    #[test]
    fn from_pixels_ctor() {
        const W: usize = 4;
        const H: usize = 5;
        let depths = [
            ColorDepth::L8,
            ColorDepth::La8,
            ColorDepth::Rgb8,
            ColorDepth::Rgba8,
        ];

        for depth in depths.iter() {
            let pixels: Vec<u8> = (1..=W * H * (*depth as usize)).map(|v| v as u8).collect();

            let img = Img::from_pixels(W, H, *depth, pixels.clone());

            assert_eq!(img.w(), W);
            assert_eq!(img.h(), H);
            assert_eq!(img.color_depth(), *depth);
            assert_eq!(img.layers().len(), *depth as usize);

            for (ind, p) in pixels.iter().enumerate() {
                let d = *depth as usize;
                let layer_ind: usize = ind % d;
                let row: usize = ind / d / W;
                let col = ind / d % W;
                let p_img: f64 = img.layer(layer_ind)[PixelPos::new(row, col)];

                assert_eq!(p_img as u8, *p);
            }
            assert_eq!(img.layers().len(), *depth as usize);
        }
    }

    #[allow(non_snake_case)]
    #[test]
    #[should_panic(expected = "values count doesn't satisfy color depth")]
    fn from_pixels_ctor_panics_if_values_count_not_divisible_by_color_depth_L8() {
        test_img_creation(ColorDepth::L8);
    }

    #[allow(non_snake_case)]
    #[test]
    #[should_panic(expected = "values count doesn't satisfy color depth")]
    fn from_pixels_ctor_panics_if_values_count_not_divisible_by_color_depth_La8() {
        test_img_creation(ColorDepth::La8);
    }

    #[allow(non_snake_case)]
    #[test]
    #[should_panic(expected = "values count doesn't satisfy color depth")]
    fn from_pixels_ctor_panics_if_values_count_not_divisible_by_color_depth_Rgb8() {
        test_img_creation(ColorDepth::Rgb8);
    }

    #[allow(non_snake_case)]
    #[test]
    #[should_panic(expected = "values count doesn't satisfy color depth")]
    fn from_pixels_ctor_panics_if_values_count_not_divisible_by_color_depth_Rgba8() {
        test_img_creation(ColorDepth::Rgba8);
    }

    fn test_img_creation(color_depth: ColorDepth) {
        const W: usize = 5;
        const H: usize = 6;

        let pixels: Vec<u8> = (1..=W * H * (color_depth as usize) + 1)
            .map(|v| v as u8)
            .collect();

        let _img = Img::from_pixels(W, H, color_depth, pixels.clone());
    }

    #[test]
    fn w_h_d_color_depth() {
        const W: usize = 4;
        const H: usize = 5;
        let depths = [
            ColorDepth::L8,
            ColorDepth::La8,
            ColorDepth::Rgb8,
            ColorDepth::Rgba8,
        ];

        for depth in depths.iter() {
            let pixels: Vec<u8> = (1..=W * H * (*depth as usize)).map(|v| v as u8).collect();

            let img = Img::from_pixels(W, H, *depth, pixels.clone());

            assert_eq!(img.w(), W);
            assert_eq!(img.h(), H);
            assert_eq!(img.color_depth(), *depth);
            assert_eq!(img.d(), *depth as usize);
        }
    }

    #[test]
    fn get_descrription() {
        const W: usize = 4;
        const H: usize = 5;
        let depths = [
            ColorDepth::L8,
            ColorDepth::La8,
            ColorDepth::Rgb8,
            ColorDepth::Rgba8,
        ];

        for depth in depths.iter() {
            let pixels: Vec<u8> = (1..=W * H * (*depth as usize)).map(|v| v as u8).collect();

            let img = Img::from_pixels(W, H, *depth, pixels.clone());

            assert_eq!(
                img.get_description(),
                format!(
                    "Изображение {} (строк) x {} (столбцов) x {} (каналов)",
                    img.h(),
                    img.w(),
                    img.d()
                )
            );
        }
    }

    #[test]
    fn layer_by_channel() {
        const W: usize = 4;
        const H: usize = 5;
        let depths = [
            ColorDepth::L8,
            ColorDepth::La8,
            ColorDepth::Rgb8,
            ColorDepth::Rgba8,
        ];

        for depth in depths.iter() {
            let pixels: Vec<u8> = (1..=W * H * (*depth as usize)).map(|v| v as u8).collect();

            let img = Img::from_pixels(W, H, *depth, pixels.clone());

            match *depth {
                ColorDepth::L8 => {
                    let l = img.layer_by_channel(ImgChannel::L).unwrap();
                    assert_eq!(l.channel(), ImgChannel::L);
                }
                ColorDepth::La8 => {
                    let l = img.layer_by_channel(ImgChannel::L).unwrap();
                    assert_eq!(l.channel(), ImgChannel::L);
                    let l = img.layer_by_channel(ImgChannel::A).unwrap();
                    assert_eq!(l.channel(), ImgChannel::A);
                }
                ColorDepth::Rgb8 => {
                    let l = img.layer_by_channel(ImgChannel::R).unwrap();
                    assert_eq!(l.channel(), ImgChannel::R);
                    let l = img.layer_by_channel(ImgChannel::G).unwrap();
                    assert_eq!(l.channel(), ImgChannel::G);
                    let l = img.layer_by_channel(ImgChannel::B).unwrap();
                    assert_eq!(l.channel(), ImgChannel::B);
                }
                ColorDepth::Rgba8 => {
                    let l = img.layer_by_channel(ImgChannel::R).unwrap();
                    assert_eq!(l.channel(), ImgChannel::R);
                    let l = img.layer_by_channel(ImgChannel::G).unwrap();
                    assert_eq!(l.channel(), ImgChannel::G);
                    let l = img.layer_by_channel(ImgChannel::B).unwrap();
                    assert_eq!(l.channel(), ImgChannel::B);
                    let l = img.layer_by_channel(ImgChannel::A).unwrap();
                    assert_eq!(l.channel(), ImgChannel::A);
                }
            }
        }
    }

    #[test]
    fn get_cropped_copy() {
        const W: usize = 9;
        const H: usize = 10;
        let depths = [
            ColorDepth::L8,
            ColorDepth::La8,
            ColorDepth::Rgb8,
            ColorDepth::Rgba8,
        ];

        for depth in depths.iter() {
            let pixels: Vec<u8> = (1..=W * H * (*depth as usize)).map(|v| v as u8).collect();

            let img = Img::from_pixels(W, H, *depth, pixels.clone());

            let crop_tl = PixelPos::new(3, 4);

            let crop_area = PixelsArea::new(crop_tl, PixelPos::new(7, 8));

            let cropped = img.get_cropped_copy(crop_area);

            for pos in crop_area.iter_pixels() {
                for d in 0..(*depth as usize) {
                    let pix1: f64 = img.layer(d)[pos];
                    let pix2: f64 = cropped.layer(d)[pos - crop_tl];
                    assert!((pix1 - pix2).abs() <= std::f64::EPSILON);
                }
            }
        }
    }

    #[test]
    fn get_area() {
        const W: usize = 4;
        const H: usize = 5;
        let depths = [
            ColorDepth::L8,
            ColorDepth::La8,
            ColorDepth::Rgb8,
            ColorDepth::Rgba8,
        ];

        for depth in depths.iter() {
            let pixels: Vec<u8> = (1..=W * H * (*depth as usize)).map(|v| v as u8).collect();

            let img = Img::from_pixels(W, H, *depth, pixels.clone());

            let area: PixelsArea = img.get_area();

            assert_eq!(area.top_left(), PixelPos::new(0, 0));
            assert_eq!(area.bottom_right(), PixelPos::new(H - 1, W - 1));
            assert_eq!(area.w(), W);
            assert_eq!(area.h(), H);
        }
    }

    #[test]
    fn get_drawable_copy() {
        const W: usize = 4;
        const H: usize = 5;
        let depths = [
            ColorDepth::L8,
            ColorDepth::La8,
            ColorDepth::Rgb8,
            ColorDepth::Rgba8,
        ];

        for depth in depths.iter() {
            let pixels: Vec<u8> = (1..=W * H * (*depth as usize)).map(|v| v as u8).collect();

            let img = Img::from_pixels(W, H, *depth, pixels.clone());

            let drawable: fltk::image::RgbImage = img.get_drawable_copy();

            let img_from_drawable = Img::from_pixels(W, H, *depth, drawable.to_rgb_data());

            assert_eq!(img.get_area(), img_from_drawable.get_area());

            for pos in img.get_area().iter_pixels() {
                for d in 0..(*depth as usize) {
                    let pix1: f64 = img.layer(d)[pos];
                    let pix2: f64 = img_from_drawable.layer(d)[pos];
                    assert!((pix1 - pix2).abs() <= std::f64::EPSILON);
                }
            }
        }
    }

    #[test]
    fn extended() {
        const W: usize = 4;
        const H: usize = 5;
        let depths = [
            ColorDepth::L8,
            ColorDepth::La8,
            ColorDepth::Rgb8,
            ColorDepth::Rgba8,
        ];

        use crate::img::filter::filter_option::ExtendValue;
        let (left, top, right, bottom) = (1, 2, 3, 4);
        for depth in depths.iter() {
            let pixels: Vec<u8> = (1..=W * H * (*depth as usize)).map(|v| v as u8).collect();

            let img = Img::from_pixels(W, H, *depth, pixels.clone());

            let ext = img.extended(ExtendValue::Closest, left, top, right, bottom);

            // extend matrix fn is already tested in matrix2d mod

            match *depth {
                ColorDepth::L8 => {
                    assert_eq!(ext.layers().len(), 1);
                    ext.layer_by_channel(ImgChannel::L)
                        .expect(&format!("Couldn't find L channel"));
                }
                ColorDepth::La8 => {
                    assert_eq!(ext.layers().len(), 2);
                    ext.layer_by_channel(ImgChannel::L)
                        .expect(&format!("Couldn't find L channel"));
                    let layer_a = ext
                        .layer_by_channel(ImgChannel::A)
                        .expect(&format!("Couldn't find A channel"));
                    assert!(layer_a
                        .matrix()
                        .pixels()
                        .iter()
                        .all(|p| (p - 255.0).abs() <= std::f64::EPSILON));
                }
                ColorDepth::Rgb8 => {
                    assert_eq!(ext.layers().len(), 3);
                    ext.layer_by_channel(ImgChannel::R)
                        .expect(&format!("Couldn't find R channel"));
                    ext.layer_by_channel(ImgChannel::G)
                        .expect(&format!("Couldn't find G channel"));
                    ext.layer_by_channel(ImgChannel::B)
                        .expect(&format!("Couldn't find B channel"));
                }
                ColorDepth::Rgba8 => {
                    assert_eq!(ext.layers().len(), 4);
                    ext.layer_by_channel(ImgChannel::R)
                        .expect(&format!("Couldn't find R channel"));
                    ext.layer_by_channel(ImgChannel::G)
                        .expect(&format!("Couldn't find G channel"));
                    ext.layer_by_channel(ImgChannel::B)
                        .expect(&format!("Couldn't find B channel"));
                    let layer_a = ext
                        .layer_by_channel(ImgChannel::A)
                        .expect(&format!("Couldn't find A channel"));
                    assert!(layer_a
                        .matrix()
                        .pixels()
                        .iter()
                        .all(|p| (p - 255.0).abs() <= std::f64::EPSILON));
                }
            }
        }
    }

    fn assert_all_pixels_are_0(matrix: &Matrix2D) {
        assert!(matrix.pixels().iter().all(|p| p.abs() <= std::f64::EPSILON));
    }
}
