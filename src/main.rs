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

use crate::utils::RectArea;
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

    while app.wait() {
        if steps_line.auto_resize(RectArea::of_widget(&wind)) {
            wind.redraw();
        }
        steps_line.process_event_loop(app)?;
    }

    Ok(())
}
