use fltk::enums::ColorDepth;
use crate::{filter::{*, filter_option::*, filter_trait::*}, img::Img, processing::*, utils::LinesIter};
use super::super::process_each_layer;


#[derive(Clone)]
pub struct HistogramLocalContrast {
    size: FilterWindowSize,
    extend_value: ExtendValue,
    mean_filter: LinearMean,
    a_values: ARange,
    name: String
}

impl HistogramLocalContrast {
    pub fn new(size: FilterWindowSize, ext_value: ExtendValue, a_values: ARange) -> Self 
    {
        HistogramLocalContrast { 
            size, 
            extend_value: ext_value, 
            mean_filter: LinearMean::new(FilterWindowSize::new(3, 3), ExtendValue::Given(0_f64)),
            a_values,
            name: "Локальный контраст (гистограмма)".to_string()
        }
    }

    pub fn w(&self) -> usize { self.size.width }
    pub fn h(&self) -> usize { self.size.height }
}

impl Filter for HistogramLocalContrast {
    fn filter(&self, img: &Img, prog_prov: &mut ProgressProvider) -> Result<Img, Halted> {
        process_each_layer(img, self, prog_prov)
    }

    fn get_steps_num(&self, img: &Img) -> usize {
        let fil_size_half = self.w() / 2;
        let mean_filter = 
            img.h() + fil_size_half * 2 + 1
            + img.w() + fil_size_half * 2 + 1
            + (img.h() + 2) * (img.w() + 2);

        let count_hists = (img.h() + 0) * (img.w() + 0);

        let count_c = (img.h() + 2) * (img.w() + 2);
        let count_c_power = (img.h() + 0) * (img.w() + 0);
        let write_res = (img.h() + 0) * (img.w() + 0);

        let per_layer = mean_filter + count_hists + count_c + count_c_power + write_res;

        let layers_count = match img.color_depth() {
            ColorDepth::L8 => img.d(),
            ColorDepth::La8 => img.d() - 1,
            ColorDepth::Rgb8 => img.d(),
            ColorDepth::Rgba8 => img.d() - 1,
        };

        layers_count * per_layer
    }

    fn get_description(&self) -> String { format!("{} {}x{}", &self.name, self.h(), self.w()) }
    
    fn get_save_name(&self) -> String {
        "HistogramLocalContrast".to_string()
    }

    fn get_copy(&self) -> FilterBase {
        let copy = self.clone();
        Box::new(copy) as FilterBase
    }
}

impl WindowFilter for HistogramLocalContrast {
    fn process_window(&self, window_buffer: &mut [f64]) -> f64 {
        //count histogram bins            
        let mut hist_counts: [u32; 256_usize] = [0; 256_usize];
        for v in &window_buffer[..] {
            hist_counts[(*v as u8) as usize] += 1;
        }

        //count min and max 
        let mut max_value = hist_counts[0];
        let mut min_value = hist_counts[0];
        for v in &hist_counts[1..] {
            if *v == 0 { continue; }
            if max_value < *v { max_value = *v; }
            if min_value < *v { min_value = *v; }
        }
        
        return if min_value == max_value {
            0_f64
        } else {
            (max_value as f64 - min_value as f64) / max_value as f64
        }
    }
    
    fn w(&self) -> usize { self.size.width }

    fn h(&self) -> usize { self.size.height }

    fn get_extend_value(&self) -> ExtendValue {
        self.extend_value
    }

    fn get_iter(&self) -> FilterIterator {
        FilterIterator {
            width: self.w(),
            height: self.h(),
            cur_pos: PixelPos::default()
        }
    }
}

impl StringFromTo for HistogramLocalContrast {
    fn try_set_from_string(&mut self, string: &str) -> Result<(), MyError> {
        let mut lines_iter = LinesIter::new(string);
        if lines_iter.len() != 3 { return Err(MyError::new("Должно быть 2 строки".to_string())); }

        let size = FilterWindowSize::try_from_string(lines_iter.next_or_empty())?
            .check_size_be_3()?
            .check_w_equals_h()?
            .check_w_h_odd()?;

        let extend_value = ExtendValue::try_from_string(lines_iter.next_or_empty())?;

        let a_values = ARange::try_from_string(&lines_iter.next_or_empty())?;

        self.size = size;
        self.a_values = a_values;
        self.extend_value = extend_value;

        Ok(())
    }
    
    fn params_to_string(&self) -> Option<String> {
        let params_str = format!("{}\n{}\n{}", self.size.content_to_string(), self.extend_value.content_to_string(), self.a_values.content_to_string());
        Some(params_str)
    }
}

impl Default for HistogramLocalContrast {
    fn default() -> Self {
        HistogramLocalContrast::new(FilterWindowSize::new(3, 3), ExtendValue::Closest, ARange::new(0.5, 0.5))
    }
}

impl ByLayer for HistogramLocalContrast {    
    fn process_layer(
        &self,
        layer: &ImgLayer, 
        prog_prov: &mut ProgressProvider) -> Result<ImgLayer, Halted> 
    {
        let mat = {
            match layer.channel() {
                ImgChannel::A => {
                    return Ok(layer.clone());
                },
                _ => layer.matrix(),
            }
        };

        let win_half = PixelPos::new(self.h() / 2, self.w() / 2);

        let mat_ext = img_ops::extend_matrix(
            mat,
            ExtendValue::Closest, 
            win_half.row, win_half.col, win_half.row, win_half.col);
        
        let mat_ext_filtered = {
            let layer_ext = ImgLayer::new(mat_ext.clone(), layer.channel());
            let layer_ext_filtered = self.mean_filter.process_layer(&layer_ext, prog_prov)?;
            layer_ext_filtered.matrix().clone()
        };

        //-------------------------------- create hist matrix ---------------------------------
        let mut mat_hist = Matrix2D::empty_size_of(&mat_ext);

        let mut pixel_buf = Vec::<f64>::new();
        pixel_buf.resize(self.w() * self.h(), 0_f64);

        for pos_im in mat_ext.get_pixels_area_iter(win_half, win_half + mat.size_vec()) {
            for pos_w in self.get_iter() {
                let buf_ind: usize = pos_w.row * self.w() + pos_w.col;
                let pix_pos: PixelPos = pos_im + pos_w - win_half;
                pixel_buf[buf_ind] = mat_ext[pix_pos];
            }            
            
            mat_hist[pos_im] = self.process_window(&mut pixel_buf[..]);

            prog_prov.complete_action()?;
        }

        //-------------------------------- create C matrix ---------------------------------
        let mut mat_c = Matrix2D::empty_size_of(&mat_ext);

        for pos in mat_ext_filtered.get_pixels_iter() {
            let mut val = mat_ext[pos] - mat_ext_filtered[pos];
            val /= mat_ext[pos] + mat_ext_filtered[pos] + f64::EPSILON;
            mat_c[pos] = f64::abs(val);

            prog_prov.complete_action()?;
        }

        for m_pos in mat_hist.get_pixels_area_iter(win_half, win_half + mat.size_vec()) {
            let mut max_value = mat_hist[m_pos];
            let mut min_value = mat_hist[m_pos];

            for w_pos in mat_hist.get_pixels_area_iter(
                m_pos - win_half, 
                m_pos + win_half) 
            {
                let v = mat_hist[w_pos];
                if f64::abs(v) < f64::EPSILON { continue; }
                if max_value < v { max_value = v; }
                if min_value < v { min_value = v; }
            }

            let mut c_power = (mat_hist[m_pos] - min_value) 
                / (max_value - min_value + f64::EPSILON);
            
            c_power = self.a_values.min + (self.a_values.max - self.a_values.min) * c_power;
            
            mat_c[m_pos] = mat_c[m_pos].powf(c_power);

            prog_prov.complete_action()?;
        }

        //-------------------------------- create result ---------------------------------         
        let mut mat_res = Matrix2D::empty_size_of(&mat);   

        for pos in mat_hist.get_pixels_area_iter(win_half, win_half + mat.size_vec()) 
        {
            let mut val = if mat_ext[pos] > mat_ext_filtered[pos] {
                mat_ext_filtered[pos] * (1_f64 + mat_c[pos]) / (1_f64 - mat_c[pos])
            } else {
                mat_ext_filtered[pos] * (1_f64 - mat_c[pos]) / (1_f64 + mat_c[pos])
            };

            if val < 0_f64 { val = 0_f64; }
            if val > 255_f64 { val = 255_f64; }

            mat_res[pos - win_half] = val;

            prog_prov.complete_action()?;
        }

        Ok(ImgLayer::new(mat_res, layer.channel()))
    }
}
