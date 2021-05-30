use std::{
    result
};

mod my_err;
mod filter;
mod proc_steps;
use my_err::MyError;

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
