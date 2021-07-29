use super::super::super::*;
use super::super::filter_trait::*;
use super::super::FilterBase;
use super::super::*;
use crate::my_err::MyError;
use crate::processing::TaskStop;
use fltk::enums::ColorDepth;

#[derive(Clone)]
pub struct CannyEdgeDetection {
    name: String,
    gaussian_filter: super::super::LinearGaussian,
    rgb2gray_filter: super::super::Rgb2Gray,
    dx_filter: super::super::LinearCustom,
    dy_filter: super::super::LinearCustom,
}

impl CannyEdgeDetection {
    pub fn new() -> Self {
        let g_sz = FilterWindowSize::new(5, 5);
        let g_ext_val = ExtendValue::Closest;

        let dx_filter_coeffs: Vec<f64> = vec![-1.0, 0.0, 1.0, -2.0, 0.0, 2.0, -1.0, 0.0, 1.0];
        let dy_filter_coeffs: Vec<f64> = vec![1.0, 2.0, 1.0, 0.0, 0.0, 0.0, -1.0, -2.0, -1.0];

        CannyEdgeDetection {
            name: "Детектор краев Канни".to_string(),
            gaussian_filter: super::super::LinearGaussian::new(g_sz, g_ext_val),
            rgb2gray_filter: super::super::Rgb2Gray::default(),
            dx_filter: super::super::LinearCustom::with_coeffs(
                dx_filter_coeffs,
                3,
                3,
                ExtendValue::Closest,
                NormalizeOption::NotNormalized,
            ),
            dy_filter: super::super::LinearCustom::with_coeffs(
                dy_filter_coeffs,
                3,
                3,
                ExtendValue::Closest,
                NormalizeOption::NotNormalized,
            ),
        }
    }
}

impl Filter for CannyEdgeDetection {
    fn process(&self, img: &Img, executor_handle: &mut ExecutorHandle) -> Result<Img, TaskStop> {
        let grayed: Img;
        let img = match img.color_depth() {
            ColorDepth::L8 | ColorDepth::La8 => img,
            ColorDepth::Rgb8 | ColorDepth::Rgba8 => {
                grayed = self.rgb2gray_filter.process(img, executor_handle)?;
                &grayed
            }
        };

        // bluring
        let layer_blured = {
            let l_layer: &ImgLayer = img
                .layers()
                .iter()
                .find(|l| l.channel() == ImgChannel::L)
                .as_ref()
                .unwrap();

            self.gaussian_filter
                .process_layer(l_layer, executor_handle)?
        };

        // derivatives by X and Y
        let dx = self
            .dx_filter
            .process_layer(&layer_blured, executor_handle)?;
        let dy = self
            .dy_filter
            .process_layer(&layer_blured, executor_handle)?;

        // gradient
        let grad: Matrix2D = {
            let mut grad = Matrix2D::generate(
                layer_blured
                    .get_area()
                    .iter_pixels()
                    .track_progress(executor_handle),
                |pos| (dx[pos].powi(2) + dy[pos].powi(2)).sqrt(),
            )?;

            let g_max: f64 = grad.get_max(executor_handle)?;

            grad.scalar_transform_self(
                |val, _| {
                    *val = *val / g_max * 255.0;
                },
                executor_handle,
            )?;

            grad
        };

        // angles
        let angles = Matrix2D::generate(
            layer_blured
                .get_area()
                .iter_pixels()
                .track_progress(executor_handle),
            |pos| {
                // top left: -3pi/4
                // top right: -1pi/4
                // bottom left: 3pi/4
                // bottom right: 1pi/4
                dy[pos].atan2(dx[pos])
            },
        )?;

        // non-max supression
        let mat_non_max_supressed: Matrix2D = {
            let generate_fcn = |pos: PixelPos| -> f64 {
                if pos.row == 0
                    || pos.col == 0
                    || pos.row == grad.max_row()
                    || pos.col == grad.max_col()
                {
                    return 0.0;
                }

                const PI_OVER_8: f64 = std::f64::consts::PI / 8.0;

                let angle_ref = &angles[pos];
                let angle_between =
                    |min: f64, max_ex: f64| -> bool { min <= *angle_ref && *angle_ref < max_ex };

                let (pos1, pos2) = if angle_between(-7.0 * PI_OVER_8, -5.0 * PI_OVER_8) {
                    // top left
                    (pos.upper_lefter(), pos.downer_righter())
                } else if angle_between(-5.0 * PI_OVER_8, -3.0 * PI_OVER_8) {
                    // top
                    (pos.upper(), pos.downer())
                } else if angle_between(-3.0 * PI_OVER_8, -1.0 * PI_OVER_8) {
                    // top right
                    (pos.upper_righter(), pos.downer_lefter())
                } else if angle_between(-1.0 * PI_OVER_8, 1.0 * PI_OVER_8) {
                    // right
                    (pos.lefter(), pos.righter())
                } else if angle_between(1.0 * PI_OVER_8, 3.0 * PI_OVER_8) {
                    // down right
                    (pos.downer_righter(), pos.upper_lefter())
                } else if angle_between(3.0 * PI_OVER_8, 5.0 * PI_OVER_8) {
                    // down
                    (pos.downer(), pos.upper())
                } else if angle_between(5.0 * PI_OVER_8, 7.0 * PI_OVER_8) {
                    // down left
                    (pos.downer_lefter(), pos.upper_righter())
                } else {
                    // left
                    (pos.lefter(), pos.righter())
                };

                if grad[pos1] > grad[pos] || grad[pos2] > grad[pos] {
                    0.0
                } else {
                    grad[pos]
                }
            };

            Matrix2D::generate(
                layer_blured
                    .get_area()
                    .iter_pixels()
                    .track_progress(executor_handle),
                generate_fcn,
            )?
        };

        // double thesholding and hysteresis
        let mat_hysteresis: Matrix2D = {
            let max_pix_value: f64 = mat_non_max_supressed.get_max(executor_handle)?;
            let high_tr = 0.09 * max_pix_value;
            let low_tr = 0.05 * high_tr;

            const WHITE: f64 = 255.0;
            const BLACK: f64 = 0.0;

            let generate_fcn = |pos: PixelPos| -> f64 {
                if pos.row == 0
                    || pos.col == 0
                    || pos.row == grad.max_row()
                    || pos.col == grad.max_col()
                {
                    return BLACK;
                }

                let is_non_relevant =
                    |pos: PixelPos| -> bool { mat_non_max_supressed[pos] <= low_tr };
                let is_weak = |pos: PixelPos| -> bool {
                    low_tr <= mat_non_max_supressed[pos] && mat_non_max_supressed[pos] <= high_tr
                };
                let has_strong_neigbour = |pos: PixelPos| -> bool {
                    let pixels_around: [f64; 8] = [
                        mat_non_max_supressed[pos.upper()],
                        mat_non_max_supressed[pos.upper_righter()],
                        mat_non_max_supressed[pos.righter()],
                        mat_non_max_supressed[pos.downer_righter()],
                        mat_non_max_supressed[pos.downer()],
                        mat_non_max_supressed[pos.downer_lefter()],
                        mat_non_max_supressed[pos.lefter()],
                        mat_non_max_supressed[pos.upper_lefter()],
                    ];
                    pixels_around.iter().any(|p| *p >= high_tr)
                };

                if is_non_relevant(pos) {
                    BLACK
                } else if is_weak(pos) {
                    if has_strong_neigbour(pos) {
                        WHITE
                    } else {
                        BLACK
                    }
                } else {
                    // is_strong
                    WHITE
                }
            };

            Matrix2D::generate(
                layer_blured
                    .get_area()
                    .iter_pixels()
                    .track_progress(executor_handle),
                generate_fcn,
            )?
        };

        // creating result
        let layer_l = ImgLayer::new(mat_hysteresis, ImgChannel::L);
        let layer_a: ImgLayer = {
            let mut layer_a = Matrix2D::empty_size_of(layer_l.matrix());
            for pos in layer_a.area().iter_pixels() {
                layer_a[pos] = 255.0;
            }
            ImgLayer::new(layer_a, ImgChannel::A)
        };
        let img_res = Img::from_layers(vec![layer_l, layer_a], ColorDepth::La8);

        Ok(img_res)
    }

    fn get_steps_num(&self, img: &Img) -> usize {
        let rows_count: usize = img.h();

        let count =
            // to make grayed
            match img.color_depth() {
                ColorDepth::L8 | ColorDepth::La8 => 0,
                ColorDepth::Rgb8 | ColorDepth::Rgba8 => self.rgb2gray_filter.get_steps_num(img),
            }

             // to make blured
            + rows_count

            // for dx
            + rows_count

            // for dy
            + rows_count

            // for grad
            + 3 * rows_count

            // for angles
            + rows_count

            // for non-max supression
            + rows_count

            // for hysteresis
            + 2 * rows_count;

        count
    }

    fn get_description(&self) -> String {
        format!("{} {}", self.name, self.gaussian_filter.get_description())
    }

    fn get_save_name(&self) -> String {
        "CannyEdgeDetection".to_string()
    }

    fn get_copy(&self) -> FilterBase {
        Box::new(self.clone()) as FilterBase
    }
}

impl StringFromTo for CannyEdgeDetection {
    fn params_to_string(&self) -> Option<String> {
        None
    }

    fn try_set_from_string(&mut self, string: &str) -> Result<(), MyError> {
        if string.trim().is_empty() {
            Ok(())
        } else {
            Err(MyError::new("У данного фильтра нет настроек".to_string()))
        }
    }
}

impl Default for CannyEdgeDetection {
    fn default() -> Self {
        CannyEdgeDetection::new()
    }
}
