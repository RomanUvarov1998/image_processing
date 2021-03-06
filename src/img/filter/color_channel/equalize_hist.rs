use super::img::*;
use super::traits::*;
use super::utils::*;
use crate::img::filter::FilterBase;
use crate::processing::{ExecutorHandle, TaskStop};
use fltk::enums::ColorDepth;

#[derive(Clone)]
pub struct EqualizeHist {}

impl Filter for EqualizeHist {
    fn process(&self, img: &Img, executor_handle: &mut ExecutorHandle) -> Result<Img, TaskStop> {
        let mut buffer: HistBuf = [0_f64; PIXEL_VALUES_COUNT];

        let mut img_res = img.clone();

        'out: for layer in img_res.layers_mut() {
            if layer.channel() == ImgChannel::A {
                continue 'out;
            }

            // count histogram
            count_histogram(layer.matrix(), &mut buffer);

            // cumulate histogram
            let mut sum = 0_f64;
            for bin in buffer.iter_mut() {
                sum += *bin;
                *bin = sum;

                executor_handle.complete_action()?;
            }

            // equalize
            let max_color_over_max_value = 255_f64 / buffer.last().unwrap();
            for bin in buffer.iter_mut() {
                *bin *= max_color_over_max_value;

                executor_handle.complete_action()?;
            }

            // apply coeff
            for pos in layer.get_area().iter_pixels() {
                let pix_value = layer[pos] as u8 as usize;
                layer[pos] = buffer[pix_value];

                executor_handle.complete_action()?;
            }
        }

        Ok(img_res)
    }

    fn get_steps_num(&self, img: &Img) -> usize {
        let pixels_per_layer = img.h() * img.w();
        let layers_count = match img.color_depth() {
            ColorDepth::L8 => img.d(),
            ColorDepth::La8 => img.d() - 1,
            ColorDepth::Rgb8 => img.d(),
            ColorDepth::Rgba8 => img.d() - 1,
        };

        layers_count * (PIXEL_VALUES_COUNT * 2 + pixels_per_layer)
    }

    fn get_description(&self) -> String {
        "Эквализация гистограммы".to_string()
    }

    fn get_save_name(&self) -> String {
        "EqualizeHist".to_string()
    }

    fn get_copy(&self) -> FilterBase {
        let copy = self.clone();
        Box::new(copy) as FilterBase
    }
}

impl StringFromTo for EqualizeHist {
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

impl Default for EqualizeHist {
    fn default() -> Self {
        EqualizeHist {}
    }
}
