use crate::{filter::FilterIterator, filter_option::ExtendValue, my_err::MyError};

pub trait StringFromTo {
    fn try_from_string(string: &str) -> Result<Self, MyError> where Self: Sized;
    fn content_to_string(&self) -> String;
}

pub trait Filter : Default + Clone + StringFromTo {
    fn filter(&self, img: crate::img::Img) -> crate::img::Img;
}

pub trait WindowFilter : Filter {
    fn process_window(&self, window_buffer: &mut [f64]) -> f64;
    fn w(&self) -> usize;
    fn h(&self) -> usize;
    fn get_extend_value(&self) -> ExtendValue;
    fn get_iterator(&self) -> FilterIterator;
}

