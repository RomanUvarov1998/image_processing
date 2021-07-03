mod my_err;
mod filter;
mod processing;
mod message;
mod img;
mod small_dlg;
mod utils;
mod my_component;

use std::{cell::RefCell, rc::Rc};
use my_err::MyError;
use crate::{my_component::Alignable};

#[macro_use]
extern crate rust_embed;

#[derive(RustEmbed)]
#[folder = "icons\\"]
pub struct Asset;

#[derive(Clone, Copy, Debug)]
pub enum AssetItem {
    AddStep,
    DeleteStep,
    EditStep,
    Export,
    Import,
    ReorderSteps,
    RunStepsChain,
    HaltProcessing,
    FitImage,
    CropImage,
}

impl AssetItem {
    pub fn to_path(&self) -> &'static str {
        match self {
            AssetItem::AddStep => "add step.png",
            AssetItem::DeleteStep => "delete step.png",
            AssetItem::EditStep => "edit step.png",
            AssetItem::Export => "export.png",
            AssetItem::Import => "import.png",
            AssetItem::ReorderSteps => "reorder steps.png",
            AssetItem::RunStepsChain => "run step.png",
            AssetItem::HaltProcessing => "stop processing.png",
            AssetItem::FitImage => "stretch.png",
            AssetItem::CropImage => "crop.png",
        }
    }
}

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
    
    use crate::my_component::line::ProcessingLine;
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
                steps_line_rc.borrow_mut().resize(w.w(), w.h());
                w.redraw();
                true
            },
            _ => false
        }
    });

    wind.end();
    wind.show();

    while app.wait() {        
        steps_line.borrow_mut().process_event_loop(app)?;
    }

    Ok(())
}

const EVENT_CONTENT_CHANGED: i32 = 40;

pub fn notify_content_changed() {
    fltk::app::handle_main(EVENT_CONTENT_CHANGED).unwrap();
}