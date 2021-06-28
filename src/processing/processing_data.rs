use crate::{img::Img};
use super::{FilterBase};

pub struct ProcessingData {
    pub step_num: usize,
    pub filter_copy: FilterBase,
    pub init_img: Img,
    pub result_img: Option<Img>,
    pub do_until_end: bool
}

impl ProcessingData {
    pub fn new(step_num: usize, filter_copy: FilterBase, init_img: Img, do_until_end: bool) -> Self {
        ProcessingData {
            step_num,
            filter_copy,
            init_img,
            result_img: None,
            do_until_end
        }
    }

    pub fn take_result(&mut self) -> Option<Img> { self.result_img.take() }
}