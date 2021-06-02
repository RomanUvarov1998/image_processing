use crate::{my_err::MyError, utils::{LinesIter, WordsIter}};
use super::filter_trait::StringFromTo;

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
        let mut lines_iter = LinesIter::new(string);
        assert_eq!(lines_iter.len(), 1);

        let format_err_msg = "Формат размера окна фильтра: '<целое число> x <целое число>".to_string();
        
        let mut words_iter = WordsIter::new(lines_iter.next(), " ");
        if words_iter.len() != 3 { return Err(MyError::new(format_err_msg)); }
        
        let height = match words_iter.next().parse::<usize>() {
            Ok(val) => val,
            Err(_) => { return Err(MyError::new(format_err_msg)); }
        };

        if words_iter.next() != "x" { return Err(MyError::new(format_err_msg)); }

        let width = match words_iter.next().parse::<usize>() {
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
        let mut lines_iter = LinesIter::new(string);
        assert_eq!(lines_iter.len(), 1);

        let mut words_iter = WordsIter::new(lines_iter.next(), " ");

        let format_err_msg = "Формат условия нормализации коэффициентов: 'Normalize: true' или 'Normalize: false'".to_string();

        if words_iter.len() != 2 {
            return Err(MyError::new(format_err_msg));
        }

        if words_iter.next() != "Normalize:" {
            return Err(MyError::new(format_err_msg));
        }

        let norm = match words_iter.next() {
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
        let mut lines_iter = LinesIter::new(string);
        assert_eq!(lines_iter.len(), 1);

        let mut words_iter = WordsIter::new(lines_iter.next(), " ");

        let foemat_err_msg = "Формат граничных условий: 'Ext: near' или 'Ext: 0'".to_string();

        if words_iter.len() != 2 {
            return Err(MyError::new(foemat_err_msg));
        }

        if words_iter.next() != "Ext:" {
            return Err(MyError::new(foemat_err_msg));
        }

        let ext_value = match words_iter.next() {
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


#[derive(Clone, Copy)]
pub struct ARange { pub min: f64, pub max: f64 }
impl ARange {
    pub fn new(min: f64, max: f64) -> Self {
        assert!(min <= max);
        ARange { min, max }
    }
}
impl StringFromTo for ARange {
    fn try_from_string(string: &str) -> Result<Self, MyError> {
        let mut lines_iter = LinesIter::new(string);
        assert_eq!(lines_iter.len(), 1);

        let format_err_msg = "Формат диапазона: '<дробное число> - <дробное число>'".to_string();
        
        let mut words_iter = WordsIter::new(lines_iter.next(), " ");
        if words_iter.len() != 3 { return Err(MyError::new(format_err_msg)); }

        let min = match words_iter.next().parse::<f64>() {
            Ok(val) => val,
            Err(_) => { return Err(MyError::new(format_err_msg)); }
        };

        if words_iter.next() != "-"  { return Err(MyError::new(format_err_msg)); }

        let max = match words_iter.next().parse::<f64>() {
            Ok(val) => val,
            Err(_) => { return Err(MyError::new(format_err_msg)); }
        };

        Ok(ARange { min, max } )
    }

    fn content_to_string(&self) -> String {
        format!("{} - {}", self.min, self.max)
    }
}


#[derive(Clone)]
pub struct CutBrightnessRange { pub min: u8, pub max: u8 }

impl CutBrightnessRange {
    pub fn new(min: u8, max: u8) -> Self {
        CutBrightnessRange { min, max }
    }
}

impl StringFromTo for CutBrightnessRange {
    fn try_from_string(string: &str) -> Result<Self, MyError> {
        let mut lines_iter = LinesIter::new(string);
        assert_eq!(lines_iter.len(), 1);

        let mut words_iter = WordsIter::new(lines_iter.next(), " ");

        let format_err_msg = "Формат диапазона яркости: '<целое число от 0 до 255 включительно> - <целое число от 0 до 255 включительно>'".to_string();

        let min = match words_iter.next().parse::<u8>() {
            Ok(val) => val,
            Err(_) => { return Err(MyError::new(format_err_msg)); }
        };

        if words_iter.next() != "-" { return Err(MyError::new(format_err_msg)); }

        let max = match words_iter.next().parse::<u8>() {
            Ok(val) => val,
            Err(_) => { return Err(MyError::new(format_err_msg)); }
        };
        Ok(CutBrightnessRange::new(min, max))
    }

    fn content_to_string(&self) -> String {
        format!("{} - {}", self.min, self.max)
    }
}


#[derive(Clone)]
pub struct ValueRepaceWith { pub value: u8 }

impl ValueRepaceWith {
    pub fn new(value: u8) -> Self { ValueRepaceWith { value } }
}

impl StringFromTo for ValueRepaceWith {
    fn try_from_string(string: &str) -> Result<Self, MyError> where Self: Sized {
        let mut lines_iter = LinesIter::new(string);
        assert_eq!(lines_iter.len(), 1);

        let format_err_msg = "Формат значения, на которое заменить: '<целое число от 0 до 255 включительно>'".to_string();

        let mut words_iter = WordsIter::new(lines_iter.next(), " ");
        
        let value = match words_iter.next().parse::<u8>() {
            Ok(val) => val,
            Err(_) => { return Err(MyError::new(format_err_msg)); },
        };

        Ok(ValueRepaceWith::new(value))
    }

    fn content_to_string(&self) -> String {
        format!("{}", self.value)
    }
}