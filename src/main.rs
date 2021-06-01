use std::{
    result
};

mod my_err;
use my_err::MyError;

mod filter;
mod filter_option;
mod filter_trait;
mod proc_steps;

mod my_app;
mod img;
mod pixel_pos;
mod step_editor;
mod small_dlg;
mod matrix2d;
mod utils;

fn main() -> result::Result<(), MyError> {
    my_app::create_app()?;

    Ok(())
}
