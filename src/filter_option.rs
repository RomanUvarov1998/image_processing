use crate::{filter_trait::StringFromTo, my_err::MyError, utils};

#[derive(Clone)]
pub struct FilterWindowSize { pub width: usize, pub height: usize }
impl FilterWindowSize {
    pub fn new(width: usize, height: usize) -> Self { 
        FilterWindowSize { width, height } 
    }
    pub fn check_w_equals_h(self) -> Result<Self, MyError> {
        if self.width != self.height { 
            return Err(MyError::new("Размеры фильтра должны быть равны".to_string())); 
        }
        Ok(self)
    }
    pub fn check_size_be_3(self) -> Result<Self, MyError> {
        if self.width < 3 || self.height < 3 { 
            return Err(MyError::new("Размеры фильтра должны быть >= 3".to_string())); 
        }
        Ok(self)
    }
    pub fn check_w_h_odd(self) -> Result<Self, MyError> {
        if self.width % 2 == 0 || self.height % 2 == 0 { 
            return Err(MyError::new("Размеры фильтра должны быть нечетными".to_string())); 
        }
        Ok(self)
    }
}
impl StringFromTo for FilterWindowSize {
    fn try_from_string(string: &str) -> Result<Self, MyError> where Self: Sized {
        let format_err_msg = "Формат размера окна фильтра: '<целое число> x <целое число>".to_string();
        
        let lines = utils::text_to_lines(string);
        if lines.len() != 1 { return Err(MyError::new(format_err_msg)); }
        
        let words = utils::line_to_words(lines[0], " ");
        if words.len() != 3 { return Err(MyError::new(format_err_msg)); }
        
        let height = match words[0].parse::<usize>() {
            Ok(val) => val,
            Err(_) => { return Err(MyError::new(format_err_msg)); }
        };

        if words[1] != "x" { return Err(MyError::new(format_err_msg)); }

        let width = match words[2].parse::<usize>() {
            Ok(val) => val,
            Err(_) => { return Err(MyError::new(format_err_msg)); }
        };

        Ok(FilterWindowSize::new(width, height))
    }

    fn content_to_string(&self) -> String {
        format!("{} x {}", self.height, self.width)
    }
}

#[derive(Clone, Copy)]
pub enum NormalizeOption {
    Normalized,
    NotNormalized
}
impl NormalizeOption {
    pub fn normalize(&self, values: &mut [f64]) {
        match self {            
            NormalizeOption::Normalized => {
                let mut sum = 0_f64;
        
                for v in values.iter() { sum += v; }
                
                if f64::abs(sum) > f64::EPSILON{
                    for v in values.iter_mut() { *v /= sum; }
                }
            }
            NormalizeOption::NotNormalized => {}
        }
    }
}
impl StringFromTo for NormalizeOption {
    fn try_from_string(string: &str) -> Result<Self, MyError> {
        let ext_words: Vec<&str> = utils::line_to_words(string, " ");

        let format_err_msg = "После граничных условий должно быть указано условие нормализации коэффициентов: 'Normalize: true' или 'Normalize: false'".to_string();

        if ext_words.len() != 2 {
            return Err(MyError::new(format_err_msg));
        }

        if ext_words[0] != "Normalize:" {
            return Err(MyError::new(format_err_msg));
        }

        let norm = match ext_words[1] {
            "true" => NormalizeOption::Normalized,
            "false" => NormalizeOption::NotNormalized,
            _ => { return Err(MyError::new(format_err_msg)); }
        };

        Ok(norm)
    }

    fn content_to_string(&self) -> String {
        match self {
            NormalizeOption::Normalized => "Normalize: true".to_string(),
            NormalizeOption::NotNormalized => "Normalize: false".to_string()
        }        
    }
}


#[derive(Clone, Copy)]
pub enum ExtendValue {
    Closest,
    Given(f64)
}
impl StringFromTo for ExtendValue {
    fn try_from_string(string: &str) -> Result<Self, MyError> {
        let ext_words: Vec<&str> = utils::line_to_words(string, " ");

        let foemat_err_msg = "Формат граничных условий: 'Ext: near' или 'Ext: 0'".to_string();

        if ext_words.len() != 2 {
            return Err(MyError::new(foemat_err_msg));
        }

        if ext_words[0] != "Ext:" {
            return Err(MyError::new(foemat_err_msg));
        }

        let ext_value = match ext_words[1] {
            "0" => ExtendValue::Given(0_f64),
            "near" => ExtendValue::Closest,
            _ => { return Err(MyError::new(foemat_err_msg)); }
        };

        Ok(ext_value)
    }

    fn content_to_string(&self) -> String {
        match self {
            ExtendValue::Closest => "Ext: near".to_string(),
            ExtendValue::Given(val) => format!("Ext: {}", val)
        }        
    }
}

