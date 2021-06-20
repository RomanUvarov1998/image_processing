use crate::{filter::{filter_trait::{Filter}}, img::{Img}};

pub mod line;
pub mod progress_provider;
mod step;
mod step_editor;

const PADDING: i32 = 20;

pub type FilterBase = Box<dyn Filter>;

struct ProcessingData {
    step_num: usize,
    filter_copy: FilterBase,
    init_img: Img,
    result_img: Option<Img>,
    do_until_end: bool
}

impl ProcessingData {
    fn new(step_num: usize, filter_copy: FilterBase, init_img: Img, do_until_end: bool) -> Self {
        ProcessingData {
            step_num,
            filter_copy,
            init_img,
            result_img: None,
            do_until_end
        }
    }

    fn take_result(&mut self) -> Option<Img> { self.result_img.take() }
}


