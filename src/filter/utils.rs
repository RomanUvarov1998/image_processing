use crate::img::{Matrix2D, PIXEL_VALUES_COUNT};

pub type HistBuf = [f64; PIXEL_VALUES_COUNT];

pub fn count_histogram(layer: &Matrix2D, buffer: &mut HistBuf) {
    for elem in buffer.iter_mut() {
        *elem = 0_f64;
    }
    
    for pos in layer.get_pixels_iter() {
        let pix_value = layer[pos] as u8 as usize;
        buffer[pix_value] += 1.0;
    }
}