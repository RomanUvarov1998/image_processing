use crate::my_err::MyError;
use crate::processing::{ExecutorHandle, TaskStop};

use super::super::Img;
use super::{filter_option::ExtendValue, FilterBase, FilterIterator};

pub trait StringFromTo {
    fn params_to_string(&self) -> Option<String>;
    fn try_set_from_string(&mut self, string: &str) -> Result<(), MyError>;
}

pub trait Filter: StringFromTo + Send {
    fn process(&self, img: &Img, executor_handle: &mut ExecutorHandle) -> Result<Img, TaskStop>;
    fn get_steps_num(&self, img: &Img) -> usize;
    fn get_description(&self) -> String;
    fn get_save_name(&self) -> String;
    fn get_copy(&self) -> FilterBase;
}

pub trait WindowFilter: Filter {
    fn process_window(&self, window_buffer: &mut [f64]) -> f64;
    fn w(&self) -> usize;
    fn h(&self) -> usize;
    fn get_extend_value(&self) -> ExtendValue;
    fn get_iter(&self) -> FilterIterator;
}
