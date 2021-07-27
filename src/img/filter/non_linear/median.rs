use super::super::super::*;
use super::super::filter_trait::*;
use super::super::FilterBase;
use super::super::*;
use crate::my_err::MyError;
use crate::processing::TaskStop;
use crate::utils::LinesIter;
use fltk::enums::ColorDepth;

#[derive(Clone)]
pub struct MedianFilter {
    size: FilterWindowSize,
    extend_value: ExtendValue,
    name: String,
}

impl MedianFilter {
    pub fn new(size: FilterWindowSize, extend_value: ExtendValue) -> Self {
        MedianFilter {
            size,
            extend_value,
            name: "Медианный фильтр".to_string(),
        }
    }
}

impl WindowFilter for MedianFilter {
    /*fn process_window(&self, window_buffer: &mut [f64]) -> f64 {
        /*
            * Algorithm from N. Wirth's book, implementation by N. Devillard.
            * This code in public domain.
            */
        let mut outer_beg: usize = 0;
        let mut outer_end: usize = window_buffer.len() - 1;
        let mut inner_beg: usize;
        let mut inner_end: usize;
        let med_ind: usize = window_buffer.len() / 2;
        let mut median: f64;

        while outer_beg < outer_end {
            median = window_buffer[med_ind];
            inner_beg = outer_beg;
            inner_end = outer_end;

            loop {
                while window_buffer[inner_beg] < median { inner_beg += 1; }
                while median < window_buffer[inner_end] { inner_end -= 1; }

                if inner_beg <= inner_end {
                    window_buffer.swap(inner_beg, inner_end);
                    inner_beg += 1; inner_end -= 1;
                }

                if inner_beg > inner_end { break; }
            }

            if inner_end < med_ind { outer_beg = inner_beg; }
            if med_ind < inner_beg { outer_end = inner_end; }
        }

        window_buffer[med_ind]
    }*/

    fn process_window(&self, window_buffer: &mut [f64]) -> f64 {
        let mut hist_buffer = [0_usize; 256];

        for val in window_buffer.iter() {
            let ind = (*val as u8) as usize;
            hist_buffer[ind] += 1_usize;
        }

        let mut values_until_median = window_buffer.len() / 2;
        let mut bin_ind = 0_usize;
        while bin_ind < hist_buffer.len() && values_until_median > hist_buffer[bin_ind] {
            values_until_median -= hist_buffer[bin_ind];
            bin_ind += 1;
        }

        bin_ind as f64
    }

    fn w(&self) -> usize {
        self.size.width
    }

    fn h(&self) -> usize {
        self.size.height
    }

    fn get_extend_value(&self) -> ExtendValue {
        self.extend_value
    }

    fn get_iter(&self) -> FilterIterator {
        FilterIterator {
            width: self.w(),
            height: self.h(),
            cur_pos: PixelPos::default(),
        }
    }
}

impl Filter for MedianFilter {
    fn process(&self, img: &Img, executor_handle: &mut ExecutorHandle) -> Result<Img, TaskStop> {
        process_each_layer(img, self, executor_handle)
    }

    fn get_steps_num(&self, img: &Img) -> usize {
        let rows_per_layer = img.h();
        let layers_count = match img.color_depth() {
            ColorDepth::L8 => img.d(),
            ColorDepth::La8 => img.d() - 1,
            ColorDepth::Rgb8 => img.d(),
            ColorDepth::Rgba8 => img.d() - 1,
        };

        layers_count * rows_per_layer
    }

    fn get_description(&self) -> String {
        format!("{} {}x{}", &self.name, self.h(), self.w())
    }

    fn get_save_name(&self) -> String {
        "MedianFilter".to_string()
    }

    fn get_copy(&self) -> FilterBase {
        let copy = self.clone();
        Box::new(copy) as FilterBase
    }
}

impl StringFromTo for MedianFilter {
    fn try_set_from_string(&mut self, string: &str) -> Result<(), MyError> {
        let mut lines_iter = LinesIter::new(string);
        if lines_iter.len() != 2 {
            return Err(MyError::new("Должно быть 2 строки".to_string()));
        }

        let size = FilterWindowSize::try_from_string(lines_iter.next_or_empty())?
            .check_size_be_3()?
            .check_w_equals_h()?
            .check_w_h_odd()?;

        let extend_value = ExtendValue::try_from_string(lines_iter.next_or_empty())?;

        self.size = size;
        self.extend_value = extend_value;

        Ok(())
    }

    fn params_to_string(&self) -> Option<String> {
        let params_str = format!(
            "{}\n{}",
            self.size.content_to_string(),
            self.extend_value.content_to_string()
        );
        Some(params_str)
    }
}

impl Default for MedianFilter {
    fn default() -> Self {
        MedianFilter::new(FilterWindowSize::new(3, 3), ExtendValue::Closest)
    }
}

impl ByLayer for MedianFilter {
    fn process_layer(
        &self,
        layer: &ImgLayer,
        executor_handle: &mut ExecutorHandle,
    ) -> Result<ImgLayer, TaskStop> {
        let result_mat = {
            match layer.channel() {
                ImgChannel::A => layer.matrix().clone(),
                _ => process_with_window(layer.matrix(), self, executor_handle)?,
            }
        };

        Ok(ImgLayer::new(result_mat, layer.channel()))
    }
}
