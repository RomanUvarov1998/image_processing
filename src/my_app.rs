use std::result;
use crate::{my_err::MyError, proc_steps::{ProcessingLine}};
use fltk::{app, enums::Damage, prelude::*, window};

pub const WIN_WIDTH: i32 = 640;
pub const WIN_HEIGHT: i32 = 480;

pub fn create_app() -> result::Result<(), MyError> {
    let app = app::App::default();
    let mut wind = window::Window::default()
        .with_size(WIN_WIDTH, WIN_HEIGHT)
        .center_screen()
        .with_label("Обработка изображений");
    wind.set_damage_type(Damage::All | Damage::Child | Damage::Scroll);
    wind.end();
    wind.make_resizable(true);
    wind.show();
    
    let mut steps_line = ProcessingLine::new(&mut wind, 0, 0, WIN_WIDTH, WIN_HEIGHT);
    steps_line.end();

    steps_line.run(app)?;

    Ok(())
}