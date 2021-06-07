use crate::{filter::FilterIterator, my_err::MyError};
use super::filter_option::ExtendValue;

pub trait StringFromTo {
    fn try_from_string(string: &str) -> Result<Self, MyError> where Self: Sized;
    fn content_to_string(&self) -> String;
}

pub trait Filter : Default + Clone + StringFromTo {
    fn filter<Cbk: Fn(usize)>(&self, img: crate::img::Matrix2D, progress_cbk: Cbk) -> crate::img::Matrix2D;
}

pub trait WindowFilter : Filter {
    fn process_window(&self, window_buffer: &mut [f64]) -> f64;
    fn w(&self) -> usize;
    fn h(&self) -> usize;
    fn get_extend_value(&self) -> ExtendValue;
    fn get_iterator(&self) -> FilterIterator;
}

