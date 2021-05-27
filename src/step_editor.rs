use fltk::{app::{self}, button, frame, group::{self, PackType}, input, prelude::{GroupExt, InputExt, WidgetBase, WidgetExt, WindowExt}, window};

use crate::proc_steps::{StepType};


const WIN_WIDTH: i32 = 600;
const WIN_HEIGHT: i32 = 500;
const BTN_TEXT_PADDING: i32 = 10;
const INP_HEIGHT: i32 = 30;
const PADDING: i32 = 20;

#[derive(Debug, Clone, Copy)]
enum StepEditMessage { Save,  Cancel }
pub struct StepEditor {
    wind: window::Window,
    lbl_msg: frame::Frame,
    mat_inp: input::MultilineInput,
    btn_save: button::Button,
}

impl StepEditor {
    pub fn new() -> Self {
        let mut wind = window::Window::default()
            .with_size(WIN_WIDTH, WIN_HEIGHT)
            .with_label("Добавление");

        let mut hpack = group::Pack::default()
            .with_pos(PADDING, PADDING)
            .with_size(WIN_WIDTH - PADDING, INP_HEIGHT);
        hpack.set_type(PackType::Horizontal);
        hpack.set_spacing(PADDING);

        let mut btn_save = button::Button::default()
            .with_label("Сохранить");
        let (w,h) = btn_save.measure_label();
        btn_save.set_size(w + BTN_TEXT_PADDING, h + BTN_TEXT_PADDING);
        
        let lbl_msg = frame::Frame::default()
            .with_size(WIN_WIDTH, INP_HEIGHT)
            .with_label("...");

        hpack.end();

        let mat_inp = input::MultilineInput::default()
            .with_pos(0, hpack.y() + hpack.h() + PADDING)
            .with_size(WIN_WIDTH, WIN_HEIGHT);

        wind.end();
        wind.make_resizable(true);
        wind.make_modal(true);

        StepEditor {
            wind, btn_save, lbl_msg, mat_inp
        }
    }

    pub fn add_linear_filter_with_dlg(&mut self, app: app::App) -> Option<StepType> {
        let (sender, receiver) = app::channel::<StepEditMessage>();

        self.btn_save.emit(sender, StepEditMessage::Save);

        self.mat_inp.set_value("");

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

        let mut result: Option<Option<StepType>> = None;

        self.wind.show();

        'event_loop: loop {
            match result {
                Some(_) => {
                    self.wind.hide();
                    break 'event_loop;
                },
                None => {}
            }

            if !app.wait() { break; }

            if let Some(msg) = receiver.recv() {
                match msg {
                    StepEditMessage::Save => {
                        let mut rows = Vec::<Vec<f64>>::new();

                        'parsing: for line in self.mat_inp.value().split('\n') {
                            let mut row = Vec::<f64>::new();
                            for word in line.trim().split(',').map(|w| w.trim()) {
                                if word.is_empty() { continue; }
                                match word.trim().parse::<f64>() {
                                    Ok(value) => { row.push(value) }
                                    Err(_) => { 
                                        self.lbl_msg.set_label("Некорректный формат чисел");
                                        break 'parsing; 
                                    }
                                }
                            }
                            if rows.len() > 0 && row.len() != rows.last().unwrap().len() {
                                self.lbl_msg.set_label("Некорректная разменость матрицы");
                                break 'parsing; 
                            }
                            if row.len() == 0 {
                                self.lbl_msg.set_label("Некорректная разменость матрицы");
                                break 'parsing; 
                            }
                            rows.push(row);
                            self.lbl_msg.set_label(&format!("Матрица {}x{}", rows.len(), rows.last().unwrap().len()));
                        }
                        
                        if rows.len() == 0 { continue; }
                        
                        let mut coeffs = Vec::<f64>::new();
                        for mut row in rows.clone() { 
                            coeffs.append(&mut row); 
                        }
                        let width = rows.last().unwrap().len();
                        let height = rows.len();

                        result = Some(Some(StepType::LinearFilter { coeffs, width, height } ));
                    },
                    StepEditMessage::Cancel => {
                        result = Some(None);
                        // self.wind.hide();
                    }
                }
            }
        }

        return result.unwrap();
    }

    /*
    pub fn edit_linear_filter_with_window(&self, app: app::App, step: &mut ProcessingStep) -> Option<StepType> {
        self.wind.show();

        step.f

        let mut result: Option<Option<StepType>> = None;

        loop {
            match result {
                Some(res) => {
                    result = Some(res);
                },
                None => {}
            }

            if !app.wait() { break; }

            if let Some(msg) = self.receiver.recv() {
                match msg {
                    StepEditMessage::Save => {
                        let mut rows = Vec::<Vec<f64>>::new();

                        'parsing: for line in self.mat_inp.value().split('\n') {
                            let mut row = Vec::<f64>::new();
                            for word in line.trim().split(',').map(|w| w.trim()) {
                                if word.is_empty() { continue; }
                                match word.trim().parse::<f64>() {
                                    Ok(value) => { row.push(value) }
                                    Err(_) => { 
                                        self.lbl_msg.set_label("Некорректный формат чисел");
                                        break 'parsing; 
                                    }
                                }
                            }
                            if rows.len() > 0 && row.len() != rows.last().unwrap().len() {
                                self.lbl_msg.set_label("Некорректная разменость матрицы");
                                break 'parsing; 
                            }
                            if row.len() == 0 {
                                self.lbl_msg.set_label("Некорректная разменость матрицы");
                                break 'parsing; 
                            }
                            rows.push(row);
                            self.lbl_msg.set_label(&format!("Матрица {}x{}", rows.len(), rows.last().unwrap().len()));
                        }
                        
                        if rows.len() == 0 { continue; }
                        
                        let mut coeffs = Vec::<f64>::new();
                        for mut row in rows.clone() { 
                            coeffs.append(&mut row); 
                        }
                        let width = rows.last().unwrap().len();
                        let height = rows.len();

                        result = Some(Some(StepType::LinearFilter { coeffs, width, height } ));

                        self.wind.hide();
                    },
                    StepEditMessage::Cancel => {
                        result = Some(None);
                        self.wind.hide();
                    }
                }
            }
        }

        return result.unwrap();
    }
    */
}