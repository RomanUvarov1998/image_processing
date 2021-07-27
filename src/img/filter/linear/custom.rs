use super::super::super::*;
use super::super::filter_trait::*;
use super::super::FilterBase;
use super::super::*;
use crate::my_err::MyError;
use crate::processing::TaskStop;
use crate::utils::{LinesIter, WordsIter};
use fltk::enums::ColorDepth;

#[derive(Clone)]
pub struct LinearCustom {
    width: usize,
    height: usize,
    extend_value: ExtendValue,
    coeffs: Vec<f64>,
    normalized: NormalizeOption,
    name: String,
}

impl LinearCustom {
    pub fn with_coeffs(
        mut coeffs: Vec<f64>,
        width: usize,
        height: usize,
        extend_value: ExtendValue,
        normalized: NormalizeOption,
    ) -> Self {
        assert!(width > 0);
        assert!(height > 0);
        assert!(coeffs.len() > 0);

        normalized.normalize(&mut coeffs[..]);

        LinearCustom {
            width,
            height,
            coeffs,
            extend_value,
            normalized,
            name: "Линейный фильтр".to_string(),
        }
    }
}

impl WindowFilter for LinearCustom {
    fn process_window(&self, window_buffer: &mut [f64]) -> f64 {
        let mut sum: f64 = 0_f64;

        for pos in self.get_iter() {
            let ind = pos.row * self.width + pos.col;
            sum += window_buffer[ind] * self.coeffs[ind];
        }

        sum
    }

    fn w(&self) -> usize {
        self.width
    }

    fn h(&self) -> usize {
        self.height
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

impl Filter for LinearCustom {
    fn process(&self, img: &Img, executor_handle: &mut ExecutorHandle) -> Result<Img, TaskStop> {
        super::super::process_each_layer(img, self, executor_handle)
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
        "LinearCustom".to_string()
    }

    fn get_copy(&self) -> FilterBase {
        let copy = self.clone();
        Box::new(copy) as FilterBase
    }
}

impl StringFromTo for LinearCustom {
    fn try_set_from_string(&mut self, string: &str) -> Result<(), MyError> {
        let mut rows = Vec::<Vec<f64>>::new();

        let mut lines_iter = LinesIter::new(string);

        if lines_iter.len() < 3 {
            return Err(MyError::new(
                "Нужно ввести матрицу и параметры на следующей строке".to_string(),
            ));
        }

        for _ in 0..lines_iter.len() - 2 {
            let mut row = Vec::<f64>::new();
            let mut words_iter = WordsIter::new(lines_iter.next_or_empty(), ",");
            loop {
                match words_iter.next_or_empty() {
                    "" => break,
                    word => match word.parse::<f64>() {
                        Ok(value) => row.push(value),
                        Err(_) => {
                            return Err(MyError::new("Некорректный формат чисел".to_string()));
                        }
                    },
                }
            }
            match rows.last() {
                Some(last_row) => {
                    if row.len() != last_row.len() {
                        return Err(MyError::new("Некорректная разменость матрицы".to_string()));
                    }
                }
                None => {}
            }
            if row.len() < 2 {
                return Err(MyError::new("Матрица должна иметь размеры > 1".to_string()));
            }
            rows.push(row);
        }

        if rows.len() < 2 {
            return Err(MyError::new("Матрица должна иметь размеры > 1".to_string()));
        }

        let extend_value = ExtendValue::try_from_string(lines_iter.next_or_empty())?;

        let normalized_value = NormalizeOption::try_from_string(lines_iter.next_or_empty())?;

        let mut coeffs = Vec::<f64>::new();
        for mut row in rows.clone() {
            coeffs.append(&mut row);
        }
        let width = rows.last().expect("rows count appeared to be 0").len();
        let height = rows.len();

        self.width = width;
        self.height = height;
        self.coeffs = coeffs;
        self.extend_value = extend_value;
        self.normalized = normalized_value;

        Ok(())
    }

    fn params_to_string(&self) -> Option<String> {
        let mut params_str = String::new();

        for row in 0..self.height {
            for col in 0..self.width {
                params_str.push_str(&self.coeffs[row * self.width + col].to_string());
                if col < self.width - 1 {
                    params_str.push_str(", ");
                }
            }
            if row < self.height - 1 {
                params_str.push_str("\n");
            }
        }

        params_str.push_str(&format!("\n{}", self.extend_value.content_to_string()));

        params_str.push_str(&format!("\n{}", self.normalized.content_to_string()));

        Some(params_str)
    }
}

impl Default for LinearCustom {
    fn default() -> Self {
        let coeffs: Vec<f64> = vec![1.0, 2.0, 1.0, 0.0, 0.0, 0.0, -1.0, -2.0, -1.0];
        LinearCustom::with_coeffs(
            coeffs,
            3,
            3,
            ExtendValue::Closest,
            NormalizeOption::Normalized,
        )
    }
}

impl ByLayer for LinearCustom {
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
