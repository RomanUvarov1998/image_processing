pub mod my_err;
pub mod processing;
pub mod img;
pub mod utils;
pub mod my_ui;

#[macro_use]
extern crate rust_embed;

pub const EVENT_CONTENT_CHANGED: i32 = 40;

pub fn notify_content_changed() {
    fltk::app::handle_main(EVENT_CONTENT_CHANGED).unwrap();
}