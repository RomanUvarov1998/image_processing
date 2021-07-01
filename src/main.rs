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

use my_err::MyError;

use crate::{my_component::Alignable};
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
    
    wind.end();
    wind.show();

    let (mut window_w_prev, mut window_h_prev) = (wind.w(), wind.h());

    while app.wait() {
        let (window_w, window_h) = (wind.w(), wind.h());

        if window_w != window_w_prev || window_h != window_h_prev {
            steps_line.resize(window_w, window_h);
            window_w_prev = window_w;
            window_h_prev = window_h;
            wind.redraw();
        }
        
        steps_line.process_event_loop(app)?;
    }

    Ok(())
}
