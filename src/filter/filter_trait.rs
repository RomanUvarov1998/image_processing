use crate::{filter::FilterIterator, img::{Img}, my_err::MyError, processing::{FilterBase, progress_provider::ProgressProvider}};
use super::filter_option::ExtendValue;

pub trait StringFromTo {
    fn content_to_string(&self) -> String;
    fn try_set_from_string(&mut self, string: &str) -> Result<(), MyError>;
}

pub trait Filter : StringFromTo + Send {
    fn filter(&self, img: &Img, prog_prov: &mut ProgressProvider) -> Img;
    fn get_description(&self) -> String;
    fn get_save_name(&self) -> String;
    fn get_copy(&self) -> FilterBase;
}

pub trait WindowFilter : Filter {
    fn process_window(&self, window_buffer: &mut [f64]) -> f64;
    fn w(&self) -> usize;
    fn h(&self) -> usize;
    fn get_extend_value(&self) -> ExtendValue;
    fn get_iter(&self) -> FilterIterator;
}

