use crate::{filter::{*, filter_option::*, filter_trait::*}, img::Img, processing::*};


#[derive(Clone)]
pub struct CannyEdgeDetection {
    name: String,
    gaussian_filter: super::super::LinearGaussian
}

impl CannyEdgeDetection {
    pub fn new() -> Self {
        let g_sz = FilterWindowSize::new(5, 5);
        let g_ext_val = ExtendValue::Closest;
        let g_kernel = super::super::LinearGaussian::new(g_sz, g_ext_val);
        
        CannyEdgeDetection {
            name: "Детектор краев Канни".to_string(),
            gaussian_filter: g_kernel
        }
    }
}

impl Filter for CannyEdgeDetection {
    fn filter(&self, img: &Img, prog_prov: &mut ProgressProvider) -> Result<Img, Halted> {
        let result = self.gaussian_filter.filter(img, prog_prov);
        result
    }

    fn get_description(&self) -> String {
        format!("{} {}", self.name, self.gaussian_filter.get_description())
    }

    fn get_save_name(&self) -> String {
        "CannyEdgeDetection".to_string()
    }

    fn get_copy(&self) -> FilterBase {
        Box::new(self.clone()) as FilterBase
    }
}

impl StringFromTo for CannyEdgeDetection {
    fn params_to_string(&self) -> Option<String> {
        None
    }

    fn try_set_from_string(&mut self, string: &str) -> Result<(), MyError> {
        if string.trim().is_empty() {
            Ok(())
        } else {
            Err(MyError::new("У данного фильтра нет настроек".to_string()))
        }
    }
}

impl Default for CannyEdgeDetection {
    fn default() -> Self {
        CannyEdgeDetection::new()
    }
}