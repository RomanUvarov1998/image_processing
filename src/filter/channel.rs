use crate::{img::{Img, ImgChannel}, proc_steps::StepAction, progress_provider::ProgressProvider};
use super::filter_trait::{ImgFilter, StringFromTo};


#[derive(Clone)]
pub struct ExtractChannel {
    channel: ImgChannel
}

impl ExtractChannel {
    pub fn new(channel: ImgChannel) -> Self {
        ExtractChannel { channel }
    }
}

impl ImgFilter for ExtractChannel {
    fn filter<Cbk: Fn(usize)>(&self, img: &Img, prog_prov: &mut ProgressProvider<Cbk>) -> Img {
        let mut img_res = img.clone();

        for layer in img_res.layers_mut() {
            if layer.channel() != self.channel && layer.channel() != ImgChannel::A { 
                for pos in layer.get_iter() {
                    layer[pos] = 0_f64;
                }
            }

            prog_prov.complete_action();
        }
        
        img_res
    }

    fn get_description(&self) -> String {
        format!("Выделение канала {}", self.channel)
    }

    fn create_progress_provider<Cbk: Fn(usize)>(&self, img: &Img, progress_cbk: Cbk) -> ProgressProvider<Cbk> {

        ProgressProvider::new(progress_cbk, img.d())
    }
}

impl StringFromTo for ExtractChannel {
    fn try_from_string(string: &str) -> Result<Self, crate::my_err::MyError> where Self: Sized {
        let channel = ImgChannel::try_from_string(string)?;

        Ok(ExtractChannel::new(channel))
    }

    fn content_to_string(&self) -> String {
        self.channel.content_to_string()
    }
}

impl Default for ExtractChannel {
    fn default() -> Self {
        ExtractChannel::new(ImgChannel::R)
    }
}

impl Into<StepAction> for ExtractChannel {
    fn into(self) -> StepAction {
        StepAction::ExtractChannel(self)
    }
}


#[derive(Clone)]
pub struct NeutralizeChannel {
    channel: ImgChannel
}

impl NeutralizeChannel {
    pub fn new(channel: ImgChannel) -> Self {
        NeutralizeChannel { channel }
    }
}

impl ImgFilter for NeutralizeChannel {
    fn filter<Cbk: Fn(usize)>(&self, img: &Img, prog_prov: &mut ProgressProvider<Cbk>) -> Img {
        let mut img_res = img.clone();

        if let Some(layer) = img_res.layers_mut().into_iter().find(|layer| layer.channel() == self.channel) {
            for pos in layer.get_iter() {
                layer[pos] = 0_f64;
            }
        }

        prog_prov.complete_action();
        
        img_res
    }

    fn get_description(&self) -> String {
        format!("Подавление канала {}", self.channel)
    }

    fn create_progress_provider<Cbk: Fn(usize)>(&self, _img: &Img, progress_cbk: Cbk) -> ProgressProvider<Cbk> {
        ProgressProvider::new(progress_cbk, 1)
    }
}

impl StringFromTo for NeutralizeChannel {
    fn try_from_string(string: &str) -> Result<Self, crate::my_err::MyError> where Self: Sized {
        let channel = ImgChannel::try_from_string(string)?;

        Ok(NeutralizeChannel::new(channel))
    }

    fn content_to_string(&self) -> String {
        self.channel.content_to_string()
    }
}

impl Default for NeutralizeChannel {
    fn default() -> Self {
        NeutralizeChannel::new(ImgChannel::R)
    }
}

impl Into<StepAction> for NeutralizeChannel {
    fn into(self) -> StepAction {
        StepAction::NeutralizeChannel(self)
    }
}