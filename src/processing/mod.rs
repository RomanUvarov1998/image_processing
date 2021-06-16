use fltk::app::App;

use crate::{filter::{channel::{ExtractChannel, NeutralizeChannel}, filter_trait::{ImgFilter, OneLayerFilter, StringFromTo}, linear::{LinearCustom, LinearGaussian, LinearMean}, non_linear::{CutBrightness, HistogramLocalContrast, MedianFilter}}, img::{Img, color_ops}, my_err::MyError};

use self::step_editor::StepEditor;

pub mod line;
pub mod progress_provider;
mod step;
mod step_editor;

const PADDING: i32 = 20;

#[derive(Clone)]
pub enum StepAction {
    LinearCustom(LinearCustom),
    LinearMean(LinearMean),
    LinearGaussian(LinearGaussian),
    MedianFilter(MedianFilter),
    HistogramLocalContrast(HistogramLocalContrast),
    CutBrightness(CutBrightness),
    HistogramEqualizer,
    Rgb2Gray,
    NeutralizeChannel(NeutralizeChannel),
    ExtractChannel(ExtractChannel),
}

impl StepAction {
    fn filter_description(&self) -> String {
        match self {
            StepAction::LinearCustom(filter) => filter.get_description(),
            StepAction::LinearMean(filter) => filter.get_description(),
            StepAction::LinearGaussian(filter) => filter.get_description(),
            StepAction::MedianFilter(filter) => filter.get_description(),
            StepAction::HistogramLocalContrast(filter) => filter.get_description(),
            StepAction::CutBrightness(filter) => filter.get_description(),
            StepAction::HistogramEqualizer => "Эквализация гистограммы".to_string(),
            StepAction::Rgb2Gray => "RGB => Gray".to_string(),
            StepAction::NeutralizeChannel(filter) => filter.get_description(),
            StepAction::ExtractChannel(filter) => filter.get_description(),
        }
    }

    fn edit_with_dlg(&self, app: App, step_editor: &mut StepEditor) -> StepAction {
        if let Some(edited_action) = step_editor.add_with_dlg(app, self.clone()) {
            edited_action
        } else {
            self.clone()
        }
    }

    pub fn act<Cbk: Fn(usize) + Clone>(&mut self, init_img: &Img, progress_cbk: Cbk) -> Img {
        match self {
            StepAction::LinearCustom(ref filter) => init_img.process_by_layer(filter, progress_cbk),
            StepAction::LinearMean(ref filter) => init_img.process_by_layer(filter, progress_cbk),
            StepAction::LinearGaussian(ref filter) => init_img.process_by_layer(filter, progress_cbk),
            StepAction::MedianFilter(ref filter) => init_img.process_by_layer(filter, progress_cbk),
            StepAction::HistogramLocalContrast(ref filter) => init_img.process_by_layer(filter, progress_cbk),
            StepAction::CutBrightness(ref filter) => init_img.process_by_layer(filter, progress_cbk),
            StepAction::HistogramEqualizer => color_ops::equalize_histogram(&init_img, progress_cbk),
            StepAction::Rgb2Gray => color_ops::rgb_to_gray(&init_img),
            StepAction::NeutralizeChannel(filter) => init_img.process_all_layers(filter, progress_cbk),
            StepAction::ExtractChannel(filter) => init_img.process_all_layers(filter, progress_cbk),
        }
    }

    pub fn content_to_string(&self) -> String {
        match self {            
            StepAction::LinearCustom(ref filter) => filter.content_to_string(),
            StepAction::LinearMean(ref filter) => filter.content_to_string(),
            StepAction::LinearGaussian(ref filter) => filter.content_to_string(),
            StepAction::MedianFilter(ref filter) => filter.content_to_string(),
            StepAction::HistogramLocalContrast(ref filter) => filter.content_to_string(),
            StepAction::CutBrightness(ref filter) => filter.content_to_string(),
            StepAction::HistogramEqualizer => "Эквализация гистограммы".to_string(),
            StepAction::Rgb2Gray => "RGB => Gray".to_string(),
            StepAction::NeutralizeChannel(filter) => filter.content_to_string(),
            StepAction::ExtractChannel(filter) => filter.content_to_string(),
        }
    }

    pub fn get_save_name(&self) -> String {
        match self {
            StepAction::LinearCustom(_) => "LinearCustom".to_string(),
            StepAction::LinearMean(_) => "LinearMean".to_string(),
            StepAction::LinearGaussian(_) => "LinearGaussian".to_string(),
            StepAction::MedianFilter(_) => "MedianFilter".to_string(),
            StepAction::HistogramLocalContrast(_) => "HistogramLocalContrast".to_string(),
            StepAction::CutBrightness(_) => "CutBrightness".to_string(),
            StepAction::HistogramEqualizer => "HistogramEqualizer".to_string(),
            StepAction::Rgb2Gray => "Rgb2Gray".to_string(),
            StepAction::NeutralizeChannel(_) => "NeutralizeChannel".to_string(),
            StepAction::ExtractChannel(_) => "ExtractChannel".to_string(),
        }
    }

    pub fn from_save_name_and_string(save_name: &str, content: &str) -> Result<Self, MyError> {
        match save_name {
            "LinearCustom" => Ok(LinearCustom::try_from_string(content)?.into()),
            "LinearMean" => Ok(LinearMean::try_from_string(content)?.into()),
            "LinearGaussian" => Ok(LinearGaussian::try_from_string(content)?.into()),
            "MedianFilter" => Ok(MedianFilter::try_from_string(content)?.into()),
            "HistogramLocalContrast" => Ok(HistogramLocalContrast::try_from_string(content)?.into()),
            "CutBrightness" => Ok(CutBrightness::try_from_string(content)?.into()),
            "HistogramEqualizer" => Ok(StepAction::HistogramEqualizer),
            "Rgb2Gray" => Ok(StepAction::Rgb2Gray),
            "NeutralizeChannel" => Ok(NeutralizeChannel::try_from_string(content)?.into()),
            "ExtractChannel" => Ok(ExtractChannel::try_from_string(content)?.into()),
            _ => Err(MyError::new(format!("Не удалось загрузить фильтр '{}'", save_name)))
        }
    }
}


struct ProcessingData {
    step_num: usize,
    step_action: StepAction,
    init_img: Img,
    result_img: Option<Img>,
    do_until_end: bool
}

impl ProcessingData {
    fn new(step_num: usize, step_action: StepAction, init_img: Img, do_until_end: bool) -> Self {
        ProcessingData {
            step_num,
            step_action,
            init_img,
            result_img: None,
            do_until_end
        }
    }

    fn take_result(&mut self) -> Option<Img> { self.result_img.take() }
}


