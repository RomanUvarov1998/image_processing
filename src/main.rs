use std::{cell::RefCell, rc::Rc};
use image_processing::{EVENT_CONTENT_CHANGED, my_ui::{Alignable, line::ProcessingLine}, my_err::MyError};


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
    
    let steps_line = ProcessingLine::new(0, 0, WIN_WIDTH, WIN_HEIGHT);

    let steps_line = Rc::new(RefCell::new(steps_line));
    let steps_line_rc = Rc::clone(&steps_line);

    wind.handle(move |w, event| {
        if event.bits() == EVENT_CONTENT_CHANGED {
            w.redraw();
            return true;
        }

        use fltk::enums::Event;
        match event {
            Event::Resize => {
                match steps_line_rc.try_borrow_mut() {
                    Ok(mut ref_mut) => {
                        ref_mut.resize(w.w(), w.h());
                        w.redraw();
                        true
                    },
                    Err(_) => false,
                }
            },
            _ => false
        }
    });

    wind.end();
    wind.show();

    while app.wait() {    
        let mut line_mut = steps_line.borrow_mut();
        line_mut.process_task_message_loop()?;
        line_mut.process_event_loop(app)?;
    }

    Ok(())
}