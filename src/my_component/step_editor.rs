use fltk::{app::{self}, frame, prelude::{DisplayExt, GroupExt, WidgetBase, WidgetExt, WindowExt}, text, window};
use crate::{img::filter::{FilterBase, filter_trait::Filter}, my_component::{Alignable, container::{MyColumn, MyRow}, usual::MyButton}};

use super::message::AddStep;

const WIN_WIDTH: i32 = 600;
const WIN_HEIGHT: i32 = 500;
const INP_HEIGHT: i32 = 30;
const PADDING: i32 = 20;
const INPUT_FIELD_SIZE: (i32, i32) = (150, 30);

#[derive(Debug, Clone, Copy)]
enum StepEditMessage { 
    TrySave,  
    Exit 
}

pub fn create(msg: AddStep, app: app::App) -> Option<FilterBase> {
    let mut filter: FilterBase = msg.into();

    if edit(app, &mut filter) {
        Some(filter)
    } else {
        None
    }
}

pub fn edit(app: app::App, filter: &mut Box<dyn Filter>) -> bool {
    let filter_settings = match filter.params_to_string() {
        Some(params_str) => params_str,
        None => return true,
    };

    let mut wind = window::Window::default()
        .with_size(WIN_WIDTH, WIN_HEIGHT)
        .with_label("Редактирование");

    let mut main_col = MyColumn::new(WIN_WIDTH - PADDING, INP_HEIGHT);

    let mut row = MyRow::new(WIN_WIDTH - PADDING);

    let (tx, rx) = app::channel::<StepEditMessage>();

    let mut btn_save = MyButton::with_label("Сохранить");
    btn_save.set_emit(tx, StepEditMessage::TrySave);

    let mut lbl_message = frame::Frame::default()
        .with_size(WIN_WIDTH - (btn_save.x() + btn_save.w() + PADDING), INPUT_FIELD_SIZE.1);
    lbl_message.set_label("");

    row.end();

    let mut text_editor = text::TextEditor::default()
        .with_pos(PADDING, row.y() + row.h() + PADDING)
        .with_size(WIN_WIDTH - PADDING * 2, WIN_HEIGHT - row.h() - PADDING);
    text_editor.set_buffer(text::TextBuffer::default()); 
    text_editor.buffer()
        .expect("Text editor has no TextBuffer")
        .set_text(&filter_settings);

    main_col.end();

    wind.end();
    wind.make_resizable(true);
    wind.make_modal(true);

    let row_copy = row.widget().clone();
    let mut text_editor_copy = text_editor.clone();
    wind.draw(move |w| {
        text_editor_copy.set_size(w.w(), w.h() - row_copy.h());
    });

    // if window is closed by user, "Close" message helps exit the message loop
    wind.handle(move |_, event| {
        match event {
            fltk::enums::Event::Hide => {
                tx.send(StepEditMessage::Exit);
                return true;
            },
            _ => {}
        }
        return false;
    });

    wind.show();

    loop {
        if !app.wait() { break; }

        if let Some(msg) = rx.recv() {
            match msg {
                StepEditMessage::TrySave => {
                    let text = text_editor.buffer()
                        .expect("Text editor has no TextBuffer")
                        .text();
                        
                    match filter.try_set_from_string(&text) {
                        Ok(_) => {
                            wind.hide();
                            return true;
                        },
                        Err(err) => lbl_message.set_label(&err.get_message()),
                    }
                },
                StepEditMessage::Exit => {
                    return false;
                }
            }
        }
    }

    return false;
}