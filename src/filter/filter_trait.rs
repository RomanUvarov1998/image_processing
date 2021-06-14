use crate::{filter::FilterIterator, img::{Img, Matrix2D}, my_err::MyError, proc_steps::StepAction, progress_provider::ProgressProvider};
use super::filter_option::ExtendValue;

pub trait StringFromTo {
    fn try_from_string(string: &str) -> Result<Self, MyError> where Self: Sized;
    fn content_to_string(&self) -> String;
}

pub trait OneLayerFilter : Default + Clone + StringFromTo + Into<StepAction> {
    fn filter<Cbk: Fn(usize)>(&self, mat: &Matrix2D, prog_prov: &mut ProgressProvider<Cbk>) -> Matrix2D;
    fn get_description(&self) -> String;
    fn create_progress_provider<Cbk: Fn(usize)>(&self, img: &Img, progress_cbk: Cbk) -> ProgressProvider<Cbk>;
}

pub trait WindowFilter : OneLayerFilter {
    fn process_window(&self, window_buffer: &mut [f64]) -> f64;
    fn w(&self) -> usize;
    fn h(&self) -> usize;
    fn get_extend_value(&self) -> ExtendValue;
    fn get_iter(&self) -> FilterIterator;
}

