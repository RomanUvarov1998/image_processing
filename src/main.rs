use std::{
    result
};

mod my_err;
use my_err::MyError;

mod filter;
mod processing;

mod my_app;
mod message;
mod img;
mod small_dlg;
mod utils;
mod my_component;

// ---------------------------------- Embedded images -------------------------------

#[macro_use]
extern crate rust_embed;

#[derive(RustEmbed)]
#[folder = "icons\\"]
pub struct Asset;


fn main() -> result::Result<(), MyError> {
    my_app::create_app()?;

    Ok(())
}
