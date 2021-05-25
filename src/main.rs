use std::{
    result
};

mod my_err;
mod filter;
use my_err::MyError;

mod my_app;
mod img;
mod pixel_pos;

fn main() -> result::Result<(), MyError> {
    my_app::create_app()?;

    Ok(())
}
