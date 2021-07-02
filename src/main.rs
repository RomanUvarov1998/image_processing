use std::{cell::RefCell, rc::Rc};

use my_err::MyError;
use crate::{my_component::Alignable, utils::Pos};

mod my_err;
mod filter;
mod processing;
mod message;
mod img;
mod small_dlg;
mod utils;
mod my_component;


#[macro_use]
extern crate rust_embed;

#[derive(RustEmbed)]
#[folder = "icons\\"]
pub struct Asset;

fn main() -> Result<(), MyError> {
    use fltk::{prelude::*, app::{App, Scheme}, enums::Damage, window::Window};

    const WIN_WIDTH: i32 = 640;
    const WIN_HEIGHT: i32 = 480;
    
    let app = App::default().with_scheme(Scheme::Plastic);

    let mut wind = Window::default()
        .with_size(WIN_WIDTH, WIN_HEIGHT)
        .center_screen()
        .with_label("Обработка изображений");
    wind.set_damage_type(Damage::All | Damage::Child | Damage::Scroll);
    wind.make_resizable(true);
    
    use crate::processing::line::ProcessingLine;
    let mut steps_line = ProcessingLine::new(0, 0, WIN_WIDTH, WIN_HEIGHT);

    let win_sz_event_loop = Rc::new(RefCell::new(None));
    let win_sz_handle = Rc::clone(&win_sz_event_loop);

    wind.handle(move |w, event| {
        use fltk::enums::Event;
        match event {
            Event::Resize => {
                win_sz_handle.borrow_mut().replace(Pos::new(w.w(), w.h()));
                true
            },
            _ => false
        }
    });

    wind.end();
    wind.show();

    while app.wait() {
        if let Some(sz) = win_sz_event_loop.borrow_mut().take() {
            steps_line.resize(sz.x, sz.y);
            wind.redraw();
        }
        
        steps_line.process_event_loop(app)?;
    }

    Ok(())
}
