use crate::{filter::{filter_trait::{Filter}}};

pub mod line;
pub mod progress_provider;
mod step;
mod background_worker;

const PADDING: i32 = 20;

pub type FilterBase = Box<dyn Filter>;
