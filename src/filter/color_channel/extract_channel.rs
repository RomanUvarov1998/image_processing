use crate::{filter::{*, filter_option::*, filter_trait::*}, img::Img, processing::*};


#[derive(Clone)]
pub struct ExtractChannel {
    channel: ImgChannel
}

impl ExtractChannel {
    pub fn new(channel: ImgChannel) -> Self {
        ExtractChannel { channel }
    }
}

impl Filter for ExtractChannel {
    fn filter(&self, img: &Img, prog_prov: &mut ProgressProvider) -> Result<Img, Halted> {
        prog_prov.reset_and_set_total_actions_count(img.d());

        let mut img_res = img.clone();

        for layer in img_res.layers_mut() {
            if layer.channel() != self.channel && layer.channel() != ImgChannel::A { 
                for pos in layer.get_iter() {
                    layer[pos] = 0_f64;
                }
            }

            prog_prov.complete_action()?;
        }
        
        Ok(img_res)
    }

    fn get_description(&self) -> String {
        format!("Выделение канала {}", self.channel)
    }

    fn get_save_name(&self) -> String {
        "ExtractChannel".to_string()
    }
    
    fn get_copy(&self) -> FilterBase {
        let copy = self.clone();
        Box::new(copy) as FilterBase
    }
}

impl StringFromTo for ExtractChannel {
    fn try_set_from_string(&mut self, string: &str) -> Result<(), MyError> {
        let channel = ImgChannel::try_from_string(string)?;

        self.channel = channel;

        Ok(())
    }

    fn params_to_string(&self) -> Option<String> {
        Some(self.channel.content_to_string())
    }
}

impl Default for ExtractChannel {
    fn default() -> Self {
        ExtractChannel::new(ImgChannel::R)
    }
}

