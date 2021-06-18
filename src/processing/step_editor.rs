use fltk::{app::{self}, frame, prelude::{DisplayExt, GroupExt, WidgetBase, WidgetExt, WindowExt}, text, window};

use crate::{filter::{channel::{ExtractChannel, NeutralizeChannel}, filter_trait::StringFromTo, linear::{LinearCustom, LinearGaussian, LinearMean}, non_linear::{CutBrightness, HistogramLocalContrast, MedianFilter}}, my_component::{Alignable, container::{MyColumn, MyRow}, usual::MyButton}};

use super::StepAction;

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

pub struct StepEditor {
    wind: window::Window,
    text_editor: text::TextEditor,
    lbl_message: frame::Frame,
    btn_save: MyButton,
}

impl StepEditor {
    pub fn new() -> Self {
        let mut wind = window::Window::default()
            .with_size(WIN_WIDTH, WIN_HEIGHT)
            .with_label("Редактирование");

        let mut main_col = MyColumn::new(WIN_WIDTH - PADDING, INP_HEIGHT);
        let mut row = MyRow::new(WIN_WIDTH - PADDING, INP_HEIGHT);

        let btn_save = MyButton::with_label("Сохранить");

        let lbl_message = frame::Frame::default()
            .with_size(WIN_WIDTH - (btn_save.x() + btn_save.w() + PADDING), INPUT_FIELD_SIZE.1);

        row.end();

        let mut text_editor = text::TextEditor::default()
            .with_pos(PADDING, row.y() + row.h() + PADDING)
            .with_size(WIN_WIDTH - PADDING * 2, WIN_HEIGHT - row.h() - PADDING);
        text_editor.set_buffer(text::TextBuffer::default()); 

        main_col.end();

        wind.end();
        wind.make_resizable(true);
        wind.make_modal(true);

        let row_copy = row.widget().clone();
        let mut text_editor_copy = text_editor.clone();
        wind.draw(move |w| {
            text_editor_copy.set_size(w.w(), w.h() - row_copy.h());
        });

        StepEditor {
            wind, btn_save, text_editor, lbl_message
        }
    }

    pub fn add_with_dlg(&mut self, app: app::App, action: StepAction) -> Option<StepAction> {
        let (sender, receiver) = app::channel::<StepEditMessage>();

        self.btn_save.set_emit(sender, StepEditMessage::TrySave);

        let filter_settings: String = action.content_to_string();
        self.text_editor.buffer().unwrap().set_text(&filter_settings);

        // if window is closed by user, "Close" message helps exit the message loop
        self.wind.handle(move |_, event| {
            match event {
                fltk::enums::Event::Hide => {
                    sender.send(StepEditMessage::Exit);
                    return true;
                },
                _ => {}
            }
            return false;
        });

        self.lbl_message.set_label("");

        self.wind.show();
        self.wind.redraw();

        loop {
            if !app.wait() { break; }

            if let Some(msg) = receiver.recv() {
                match msg {
                    StepEditMessage::TrySave => {
                        let text = match self.text_editor.buffer() {
                            Some(ref buf) => buf.text(),
                            None => continue
                        };
                        match &action {
                            StepAction::LinearCustom(_) => match LinearCustom::try_from_string(&text) {
                                Ok(filter) => {
                                    self.wind.hide();
                                    return Some(filter.into());
                                },
                                Err(err) => self.lbl_message.set_label(&err.get_message())
                            },
                            StepAction::LinearMean(_) => match LinearMean::try_from_string(&text) {
                                Ok(filter) => {
                                    self.wind.hide();
                                    return Some(filter.into());
                                },
                                Err(err) => self.lbl_message.set_label(&err.get_message())
                            },
                            StepAction::LinearGaussian(_) => match LinearGaussian::try_from_string(&text) {
                                Ok(filter) => {
                                    self.wind.hide();
                                    return Some(filter.into());
                                },
                                Err(err) => self.lbl_message.set_label(&err.get_message())
                            },
                            StepAction::MedianFilter(_) => match MedianFilter::try_from_string(&text) {
                                Ok(filter) => {
                                    self.wind.hide();
                                    return Some(filter.into());
                                },
                                Err(err) => self.lbl_message.set_label(&err.get_message())
                            },
                            StepAction::HistogramLocalContrast(_) => match HistogramLocalContrast::try_from_string(&text) {
                                Ok(filter) => {
                                    self.wind.hide();
                                    return Some(filter.into());
                                },
                                Err(err) => self.lbl_message.set_label(&err.get_message())
                            },
                            StepAction::CutBrightness(_) => match CutBrightness::try_from_string(&text) {
                                Ok(filter) => {
                                    self.wind.hide();
                                    return Some(filter.into());
                                },
                                Err(err) => self.lbl_message.set_label(&err.get_message())
                            },
                            StepAction::NeutralizeChannel(_) => match NeutralizeChannel::try_from_string(&text) {
                                Ok(filter) => {
                                    self.wind.hide();
                                    return Some(filter.into());
                                },
                                Err(err) => self.lbl_message.set_label(&err.get_message())
                            },
                            StepAction::ExtractChannel(_) => match ExtractChannel::try_from_string(&text) {
                                Ok(filter) => {
                                    self.wind.hide();
                                    return Some(filter.into());
                                },
                                Err(err) => self.lbl_message.set_label(&err.get_message())
                            },
                            _ => { 
                                self.wind.hide();
                                return Some(action); 
                            },
                        };
                    },
                    StepEditMessage::Exit => {
                        return None;
                    }
                }
            }
        }

        return None;
    }
}