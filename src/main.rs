use std::{
    result
};

mod my_err;
use my_err::MyError;

mod my_app;
mod img;

fn main() -> result::Result<(), MyError> {
    my_app::create_app()?;

    Ok(())
}
