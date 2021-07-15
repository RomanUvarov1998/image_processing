use std::{ops::{Index, IndexMut}, path::PathBuf, result};
use fltk::{enums::ColorDepth, image::{self}, prelude::ImageExt};
use crate::my_err::MyError;
use self::{filter::filter_trait::WindowFilter};
use self::filter::filter_option::*;

pub mod filter;
mod pixel_pos;
mod img_layer;
mod matrix2d;
mod iterators;
mod img;

pub use pixel_pos::PixelPos;
pub use img_layer::ImgLayer;
pub use matrix2d::Matrix2D;
pub use iterators::*;
pub use img::Img;

pub const PIXEL_VALUES_COUNT: usize = 256_usize;
