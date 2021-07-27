pub mod color_channel;
pub mod filter_option;
pub mod filter_trait;
pub mod linear;
pub mod non_linear;
pub mod utils;

use self::filter_trait::WindowFilter;
use crate::{
    img::{Img, ImgLayer, Matrix2D},
    my_err::MyError,
    processing::{ExecutorHandle, TaskStop},
};

pub type FilterBase = Box<dyn self::filter_trait::Filter>;

use crate::my_ui::message::AddStep;
impl From<AddStep> for FilterBase {
    fn from(msg: AddStep) -> Self {
        match msg {
            AddStep::LinCustom => Box::new(LinearCustom::default()) as FilterBase,
            AddStep::LinMean => Box::new(LinearMean::default()) as FilterBase,
            AddStep::LinGauss => Box::new(LinearGaussian::default()) as FilterBase,
            AddStep::Median => Box::new(MedianFilter::default()) as FilterBase,
            AddStep::HistogramLocalContrast => {
                Box::new(HistogramLocalContrast::default()) as FilterBase
            }
            AddStep::CutBrightness => Box::new(CutBrightness::default()) as FilterBase,
            AddStep::HistogramEqualizer => Box::new(EqualizeHist::default()) as FilterBase,
            AddStep::Rgb2Gray => Box::new(Rgb2Gray::default()) as FilterBase,
            AddStep::NeutralizeChannel => Box::new(NeutralizeChannel::default()) as FilterBase,
            AddStep::ExtractChannel => Box::new(ExtractChannel::default()) as FilterBase,
            AddStep::CannyEdgeDetection => Box::new(CannyEdgeDetection::default()) as FilterBase,
        }
    }
}

pub struct FilterIterator {
    width: usize,
    height: usize,
    cur_pos: PixelPos,
}

impl FilterIterator {
    pub fn fits(&self, pos: PixelPos) -> bool {
        pos.col < self.width && pos.row < self.height
    }
}

impl Iterator for FilterIterator {
    type Item = PixelPos;

    fn next(&mut self) -> Option<PixelPos> {
        let curr = self.cur_pos;

        self.cur_pos.col += 1;

        if self.cur_pos.col >= self.width {
            self.cur_pos.col = 0;
            self.cur_pos.row += 1;
        }

        if self.fits(curr) {
            Some(curr)
        } else {
            None
        }
    }
}

fn process_with_window<T: WindowFilter>(
    init: &Matrix2D,
    filter: &T,
    executor_handle: &mut ExecutorHandle,
) -> Result<Matrix2D, TaskStop> {
    assert!(filter.w() > 1);
    assert!(filter.h() > 1);

    let mut res = Matrix2D::empty_size_of(init);

    let mut pixel_buf = Vec::<f64>::new();
    pixel_buf.resize(filter.w() * filter.h(), 0_f64);

    let fil_half_size = PixelPos::new(filter.h() / 2, filter.w() / 2);

    let layer_ext = init.extended_for_window_filter(filter);

    // for row in fil_half_size,
    //     PixelPos::new(init.h(), init.w()) + fil_half_size)
    for row in fil_half_size.row..init.h() + fil_half_size.row {
        for col in fil_half_size.col..init.w() + fil_half_size.col {
            let pos_im = PixelPos::new(row, col);

            for pos_w in filter.get_iter() {
                let buf_ind: usize = pos_w.row * filter.w() + pos_w.col;
                let pix_pos: PixelPos = pos_im + pos_w - fil_half_size;
                pixel_buf[buf_ind] = layer_ext[pix_pos];
            }

            let filter_result: f64 = filter.process_window(&mut pixel_buf[..]);

            res[pos_im - fil_half_size] = filter_result;
        }

        executor_handle.complete_action()?;
    }

    Ok(res)
}

// AnyFilter : ByLayer -> filter<>() -> filter_layers<>() -> process_layer<>()

trait ByLayer {
    fn process_layer(
        &self,
        layer: &ImgLayer,
        executor_handle: &mut ExecutorHandle,
    ) -> Result<ImgLayer, TaskStop>;
}

fn process_each_layer<F: ByLayer>(
    img: &Img,
    filter: &F,
    executor_handle: &mut ExecutorHandle,
) -> Result<Img, TaskStop> {
    let mut res_layers = Vec::<ImgLayer>::with_capacity(img.d());

    for layer in img.layers().iter() {
        let res_layer = filter.process_layer(layer, executor_handle)?;
        res_layers.push(res_layer);
    }

    Ok(Img::new(img.w(), img.h(), res_layers, img.color_depth()))
}

use self::{color_channel::*, linear::*, non_linear::*};

use super::PixelPos;
pub fn try_parce_filter(save_name: &str, content: &str) -> Result<FilterBase, MyError> {
    let mut filter = match save_name {
        "LinearCustom" => Box::new(LinearCustom::default()) as FilterBase,
        "LinearMean" => Box::new(LinearMean::default()) as FilterBase,
        "LinearGaussian" => Box::new(LinearGaussian::default()) as FilterBase,
        "MedianFilter" => Box::new(MedianFilter::default()) as FilterBase,
        "HistogramLocalContrast" => Box::new(HistogramLocalContrast::default()) as FilterBase,
        "CutBrightness" => Box::new(CutBrightness::default()) as FilterBase,
        "EqualizeHist" => Box::new(EqualizeHist::default()) as FilterBase,
        "Rgb2Gray" => Box::new(Rgb2Gray::default()) as FilterBase,
        "NeutralizeChannel" => Box::new(NeutralizeChannel::default()) as FilterBase,
        "ExtractChannel" => Box::new(ExtractChannel::default()) as FilterBase,
        "CannyEdgeDetection" => Box::new(CannyEdgeDetection::default()) as FilterBase,
        _ => {
            return Err(MyError::new(format!(
                "Не удалось загрузить фильтр '{}'",
                save_name
            )));
        }
    };
    filter.try_set_from_string(content)?;
    Ok(filter)
}

#[cfg(test)]
mod tests {
    use crate::{
        img::{
            filter::{color_channel::*, linear::*, non_linear::*, FilterBase},
            Img,
        },
        processing::create_task_info_channel,
    };

    #[test]
    fn all_actions_are_completed() {
        let filters: Vec<FilterBase> = vec![
            Box::new(LinearCustom::default()) as FilterBase,
            Box::new(LinearGaussian::default()) as FilterBase,
            Box::new(LinearMean::default()) as FilterBase,
            Box::new(CannyEdgeDetection::default()) as FilterBase,
            Box::new(HistogramLocalContrast::default()) as FilterBase,
            Box::new(MedianFilter::default()) as FilterBase,
            Box::new(CutBrightness::default()) as FilterBase,
            Box::new(EqualizeHist::default()) as FilterBase,
            Box::new(ExtractChannel::default()) as FilterBase,
            Box::new(NeutralizeChannel::default()) as FilterBase,
            Box::new(Rgb2Gray::default()) as FilterBase,
        ];

        let img = Img::empty_with_size(100, 100, fltk::enums::ColorDepth::Rgba8);

        for filter in filters.iter() {
            let (mut executor_handle, _delegator_handle) = create_task_info_channel();
            executor_handle.reset(filter.get_steps_num(&img));
            let _res = filter.process(&img, &mut executor_handle);
            executor_handle.assert_all_actions_completed();
            println!("{} is ok", filter.get_description());
        }
    }
}
