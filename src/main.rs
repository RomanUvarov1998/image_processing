use std::{
    result
};

mod my_err;
use my_err::MyError;

mod filter;
mod proc_steps;

mod my_app;
mod img;
mod step_editor;
mod small_dlg;
mod utils;

fn main() -> result::Result<(), MyError> {
    my_app::create_app()?;

    Ok(())
}
