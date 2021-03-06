use super::super::super::Img;
use super::super::FilterBase;
use super::super::{process_each_layer, *};
use super::options::*;
use super::traits::*;
use crate::my_err::MyError;
use crate::processing::{ExecutorHandle, TaskStop};
use crate::utils::LinesIter;
use fltk::enums::ColorDepth;

#[derive(Clone)]
pub struct CutBrightness {
    cut_range: CutBrightnessRange,
    replace_with: ValueRepaceWith,
    name: String,
}

impl CutBrightness {
    pub fn new(cut_range: CutBrightnessRange, replace_with: ValueRepaceWith) -> Self {
        CutBrightness {
            cut_range,
            replace_with,
            name: "Вырезание яркости".to_string(),
        }
    }
}

impl Filter for CutBrightness {
    fn process(&self, img: &Img, executor_handle: &mut ExecutorHandle) -> Result<Img, TaskStop> {
        process_each_layer(img, self, executor_handle)
    }

    fn get_steps_num(&self, img: &Img) -> usize {
        let pixels_per_layer = img.h() * img.w();
        let layers_count = match img.color_depth() {
            ColorDepth::L8 => img.d(),
            ColorDepth::La8 => img.d() - 1,
            ColorDepth::Rgb8 => img.d(),
            ColorDepth::Rgba8 => img.d() - 1,
        };

        layers_count * pixels_per_layer
    }

    fn get_description(&self) -> String {
        format!(
            "{} ({} - {})",
            &self.name, self.cut_range.min, self.cut_range.max
        )
    }

    fn get_save_name(&self) -> String {
        "CutBrightness".to_string()
    }

    fn get_copy(&self) -> FilterBase {
        let copy = self.clone();
        Box::new(copy) as FilterBase
    }
}

impl Default for CutBrightness {
    fn default() -> Self {
        Self::new(CutBrightnessRange::new(100, 200), ValueRepaceWith::new(0))
    }
}

impl StringFromTo for CutBrightness {
    fn try_set_from_string(&mut self, string: &str) -> Result<(), MyError> {
        let mut lines_iter = LinesIter::new(string);
        if lines_iter.len() != 2 {
            return Err(MyError::new("Должно быть 2 строки".to_string()));
        }

        let cut_range = CutBrightnessRange::try_from_string(lines_iter.next_or_empty())?;

        let replace_with = ValueRepaceWith::try_from_string(lines_iter.next_or_empty())?;

        self.cut_range = cut_range;
        self.replace_with = replace_with;

        Ok(())
    }

    fn params_to_string(&self) -> Option<String> {
        let params_str = format!(
            "{}\n{}",
            self.cut_range.content_to_string(),
            self.replace_with.content_to_string()
        );
        Some(params_str)
    }
}

impl ByLayer for CutBrightness {
    fn process_layer(
        &self,
        layer: &ImgLayer,
        executor_handle: &mut ExecutorHandle,
    ) -> Result<ImgLayer, TaskStop> {
        let mut mat_res = {
            match layer.channel() {
                ImgChannel::A => {
                    return Ok(layer.clone());
                }
                _ => Matrix2D::empty_size_of(layer.matrix()),
            }
        };

        let mat = layer.matrix();

        for pos in mat.area().iter_pixels() {
            let pix_val = mat[pos] as u8;
            let before_min = pix_val < self.cut_range.min;
            let after_max = pix_val > self.cut_range.max;

            let result = pix_val * (!before_min) as u8 * (!after_max) as u8
                + self.replace_with.value * before_min as u8 * after_max as u8;

            mat_res[pos] = result as f64;

            executor_handle.complete_action()?;
        }

        Ok(ImgLayer::new(mat_res, layer.channel()))
    }
}
