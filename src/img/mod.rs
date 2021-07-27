use self::filter::filter_option::*;
use self::filter::filter_trait::WindowFilter;
use crate::my_err::MyError;
use fltk::{
    enums::ColorDepth,
    image::{self},
    prelude::ImageExt,
};
use std::{
    ops::{Index, IndexMut},
    path::PathBuf,
    result,
};

pub mod filter;
mod img;
mod img_layer;
mod iterators;
mod matrix2d;

pub use img::Img;
pub use img_layer::ImgLayer;
pub use iterators::*;
pub use matrix2d::Matrix2D;

pub const PIXEL_VALUES_COUNT: usize = 256_usize;
