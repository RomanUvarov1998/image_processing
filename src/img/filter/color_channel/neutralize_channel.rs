use crate::my_err::MyError;
use crate::processing::{ExecutorHandle, Halted};
use super::super::super::ImgChannel;
use super::super::super::Img;
use super::super::filter_trait::*;
use super::super::filter_option::*;
use super::super::FilterBase;


#[derive(Clone)]
pub struct NeutralizeChannel {
    channel: ImgChannel
}

impl NeutralizeChannel {
    pub fn new(channel: ImgChannel) -> Self {
        NeutralizeChannel { channel }
    }
}

impl Filter for NeutralizeChannel {
    fn process(&self, img: &Img, executor_handle: &ExecutorHandle) -> Result<Img, Halted> {
        let mut img_res = img.clone();

        if let Some(layer) = img_res.layers_mut().into_iter().find(|layer| layer.channel() == self.channel) {
            for pos in layer.get_iter() {
                layer[pos] = 0_f64;
                executor_handle.complete_action()?;
            }
        }
        
        Ok(img_res)
    }

    fn get_steps_num(&self, img: &Img) -> usize {
        if let Some(_) = img.layers().into_iter().find(|layer| layer.channel() == self.channel) {
            img.w() * img.h()
        } else {
            0
        }
    }

    fn get_description(&self) -> String {
        format!("Подавление канала {}", self.channel)
    }
    
    fn get_save_name(&self) -> String {
        "NeutralizeChannel".to_string()
    }

    fn get_copy(&self) -> FilterBase {
        let copy = self.clone();
        Box::new(copy) as FilterBase
    }
}

impl StringFromTo for NeutralizeChannel {
    fn params_to_string(&self) -> Option<String> {
        Some(self.channel.content_to_string())
    }

    fn try_set_from_string(&mut self, string: &str) -> Result<(), MyError> {
        let channel = ImgChannel::try_from_string(string)?;

        self.channel = channel;

        Ok(())
    }
}

impl Default for NeutralizeChannel {
    fn default() -> Self {
        NeutralizeChannel::new(ImgChannel::R)
    }
}
