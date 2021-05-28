use fltk::{app::{self}, button, enums::Damage, group::{self, PackType}, input, prelude::{GroupExt, InputExt, WidgetBase, WidgetExt, WindowExt}, window};

use crate::{filter::{Filter}, proc_steps::{StepAction}};

const WIN_WIDTH: i32 = 600;
const WIN_HEIGHT: i32 = 500;
const BTN_TEXT_PADDING: i32 = 10;
const INP_HEIGHT: i32 = 30;
const PADDING: i32 = 20;
const INPUT_FIELD_SIZE: (i32, i32) = (150, 30);

#[derive(Debug, Clone, Copy)]
enum StepEditMessage { 
    SetCoeff { row: usize, col: usize, coeff: f64 }, 
    SetSize { w: usize, h: usize }, 
    Save,  
    Cancel 
}

pub struct StepEditor {
    wind: window::Window,
    scroll_editor: group::Scroll,
    btn_save: button::Button,
}

impl StepEditor {
    pub fn new() -> Self {
        let mut wind = window::Window::default()
            .with_size(WIN_WIDTH, WIN_HEIGHT)
            .with_label("Добавление");
        wind.set_damage_type(Damage::All | Damage::Child | Damage::Scroll);

        let mut hpack = group::Pack::default()
            .with_pos(PADDING, PADDING)
            .with_size(WIN_WIDTH - PADDING, INP_HEIGHT);
        hpack.set_type(PackType::Horizontal);
        hpack.set_spacing(PADDING);

        let mut btn_save = button::Button::default()
            .with_label("Сохранить");
        let (w,h) = btn_save.measure_label();
        btn_save.set_size(w + BTN_TEXT_PADDING, h + BTN_TEXT_PADDING);

        hpack.end();

        let scroll_editor = group::Scroll::default()
            .with_size(WIN_WIDTH - PADDING, WIN_HEIGHT - (hpack.y() + hpack.h() + PADDING))
            .with_pos(0, hpack.y() + hpack.h() + PADDING);
        scroll_editor.scrollbar().set_damage_type(Damage::All);
        scroll_editor.begin();
        
        scroll_editor.end();

        wind.end();
        wind.make_resizable(true);
        wind.make_modal(true);

        StepEditor {
            wind, btn_save, scroll_editor
        }
    }

    pub fn add_step_action_with_dlg(&mut self, app: app::App, mut step_action: StepAction) -> Option<StepAction> {
        let (sender, receiver) = app::channel::<StepEditMessage>();

        self.btn_save.emit(sender, StepEditMessage::Save);

        self.wind.begin();
        self.scroll_editor.begin();
        
        while self.scroll_editor.children() > 2 {  // the first 2 children are scrollbars, if remove them, the app crashes
            self.scroll_editor.remove_by_index(0);
        }

        self.scroll_editor.resize(self.scroll_editor.x(), self.scroll_editor.y(), 
            WIN_WIDTH, WIN_HEIGHT - self.scroll_editor.y());
        self.scroll_editor.scrollbar().set_damage(true);

        match step_action {
            StepAction::Linear(ref filter) => {
                let mut rows_pack = group::Pack::default()
                    .with_size(
                        INPUT_FIELD_SIZE.0 * filter.w() as i32,
                        INPUT_FIELD_SIZE.1 * filter.h() as i32);
                rows_pack.set_type(PackType::Vertical);
                for row in 0..filter.h() {
                    let mut cols_pack = group::Pack::default()
                        .with_size(INPUT_FIELD_SIZE.0 * filter.w() as i32, INPUT_FIELD_SIZE.1);
                    cols_pack.set_type(PackType::Horizontal);
                    cols_pack.begin();
                    for col in 0..filter.w(){
                        let mut inp = input::FloatInput::default()
                            .with_size(INPUT_FIELD_SIZE.0, INPUT_FIELD_SIZE.1);
                        inp.set_value(&filter.get_coeff(row, col).to_string());
                        inp.set_callback(move |inp| {
                            match inp.value().parse::<f64>() {
                                Ok(coeff) => sender.send(StepEditMessage::SetCoeff { row, col, coeff }),
                                Err(_) => inp.set_value("0")
                            }
                            
                        });
                    }
                    cols_pack.end();
                }
                rows_pack.end();            
            }
            StepAction::Median(ref filter) => {
                let mut rows_pack = group::Pack::default();

                let mut inp = input::IntInput::default()
                    .with_size(INPUT_FIELD_SIZE.0, INPUT_FIELD_SIZE.1);
                inp.set_value(&filter.window_size().to_string());
                inp.set_callback(move |inp| {
                    match inp.value().parse::<usize>() {
                        Ok(val) => {
                            sender.send(StepEditMessage::SetSize { w: val, h: val });
                        }
                        Err(_) => inp.set_value("0")
                    }
                });
                inp.set_label("Размер");
                let (w, _) = inp.measure_label();
                rows_pack.set_size(INPUT_FIELD_SIZE.0 + w, INPUT_FIELD_SIZE.1);

                rows_pack.end();
            }
        }
  
        self.scroll_editor.end(); 
        self.wind.end();
        self.wind.set_damage(true);

        self.wind.handle(move |_, event| {
            match event {
                fltk::enums::Event::Hide => {
                    sender.send(StepEditMessage::Cancel);
                    return true;
                },
                _ => {}
            }
            return false;
        });

        self.wind.show();

        loop {
            if !app.wait() { break; }

            if let Some(msg) = receiver.recv() {
                match msg {
                    StepEditMessage::SetSize { w, h } => {
                        match step_action {
                            StepAction::Linear(ref mut filter) => {
                                while filter.w() < w { filter.add_col(); }
                                while filter.h() < h { filter.add_row(); }
                            },
                            StepAction::Median(ref mut filter) => {
                                assert_eq!(w, h);
                                filter.set_size(w);
                            }
                        }
                    },
                    StepEditMessage::SetCoeff { row, col, coeff } => {
                        match step_action {
                            StepAction::Linear(ref mut filter) => {
                                filter.set_coeff(row, col, coeff);
                            },
                            StepAction::Median(_) => {}
                        }
                    },
                    StepEditMessage::Save => {
                        self.wind.hide();
                        return Some(step_action);
                    },
                    StepEditMessage::Cancel => {
                        return None;
                    }
                }
            }
        }

        return None;
    }
}