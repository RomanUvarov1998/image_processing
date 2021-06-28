use crate::{filter::{filter_trait::{Filter}}};

pub mod line;
pub mod progress_provider;
mod step;
mod step_editor;
mod processing_data;

const PADDING: i32 = 20;

pub type FilterBase = Box<dyn Filter>;
