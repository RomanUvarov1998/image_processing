use std::result;
use fltk::{app::Sender, button, dialog, enums::{Align, FrameType}, frame::{self, Frame}, group, prelude::{GroupExt, ImageExt, WidgetBase, WidgetExt}};
use crate::{filter::{Filter, LinearFilter, MedianFilter}, img, my_err::MyError};

pub enum StepType {
    LinearFilter (usize),
    MedianFilter (usize)
}

pub const PADDING: i32 = 3;
pub const BTN_WIDTH: i32 = 100;
pub const BTN_HEIGHT: i32 = 30;
pub const BTN_TEXT_PADDING: i32 = 10;

#[derive(Debug, Copy, Clone)]
pub enum Message {
    LoadImage,
    DoStep { step_num: usize },
    AddStep
}

enum StepAction {
    Linear(LinearFilter),
    Median(MedianFilter),
}

pub struct ProcessingLine {
    initial_img: Option<img::Img>,
    frame_img: frame::Frame,
    steps: Vec<ProcessingStep>,
    max_height: i32,
    x: i32, y: i32, w: i32, h: i32,
    scroll_area: group::Scroll,
    sender: Sender<Message>
}

impl ProcessingLine {
    pub fn new(sender: Sender<Message>, x: i32, y: i32, w: i32, h: i32) -> Self {
        let scroll_area = group::Scroll::default()
            .with_pos(x, y)
            .with_size(w, h);

        let mut max_height = 0_i32;

        let label = frame::Frame::default()
            .with_pos(x, y + max_height)
            .with_size(w, BTN_HEIGHT)
            .with_label("Загрузка изображения");
        max_height += label.height() + PADDING;

        let mut btn = button::Button::default()
            .with_size(BTN_WIDTH, BTN_HEIGHT)
            .with_pos(x,  y + max_height)
            .with_label("Загрузить");
        btn.emit(sender, Message::LoadImage );
        
        {
            let (bw, bh) = btn.measure_label();
            btn.set_size(bw + BTN_TEXT_PADDING, bh + BTN_TEXT_PADDING);
        }
        max_height += btn.height() + PADDING;
            
        let mut frame_img = frame::Frame::default()
            .with_pos(x,  y + max_height)
            .with_size(w, h - BTN_HEIGHT * 2);
        frame_img.set_frame(FrameType::EmbossedFrame);
        frame_img.set_align(Align::ImageMask | Align::TextNextToImage | Align::Bottom);    
        frame_img.draw(move |f: &mut frame::Frame| { 
            match f.image() {
                Some(mut img) => {
                    img.scale(f.width(), f.height(), true, true);
                    img.draw(
                        f.x() + f.width() / 2 - img.width() / 2, 
                        f.y() + f.height() / 2 - img.height() / 2, 
                        f.width(), f.height());
                    f.redraw();
                },
                None => { 
                    f.set_label("");
                }
            }
        });
        max_height += frame_img.height() + PADDING;

        ProcessingLine {
            initial_img: None,
            frame_img,
            steps: Vec::<ProcessingStep>::new(),
            max_height: max_height,
            x, y, w, h,
            scroll_area,
            sender
        }
    }

    pub fn add(&mut self, step_type: StepType, sender: Sender<Message>) -> () {
        let label = frame::Frame::default()
            .with_pos(self.x, self.y + self.max_height)
            .with_size(self.w, BTN_HEIGHT);   
        self.max_height += label.height() + PADDING;

        let mut btn = button::Button::default()
            .with_pos(self.x,  self.y + self.max_height);

        match step_type {
            StepType::LinearFilter(_) => {
                btn.set_label("Отфильтровать");
                btn.emit(sender, Message::DoStep { step_num: self.steps.len() } );
            },
            StepType::MedianFilter(_) => {
                btn.set_label("Отфильтровать");
                btn.emit(sender, Message::DoStep { step_num: self.steps.len() } );
            }
        };

        let (w, h) = btn.measure_label();
        btn.set_size(w + BTN_TEXT_PADDING, h + BTN_TEXT_PADDING);

        self.max_height += btn.height() + PADDING;
            
        let mut frame_img = frame::Frame::default()
            .with_pos(self.x,  self.y + self.max_height)
            .with_size(self.w, self.h - BTN_HEIGHT * 2);
        frame_img.set_frame(FrameType::EmbossedFrame);
        frame_img.set_align(Align::ImageMask | Align::TextNextToImage | Align::Bottom);    
        frame_img.draw(|f: &mut frame::Frame| { 
            match f.image() {
                Some(mut img) => {
                    img.scale(f.width(), f.height(), true, true);
                    img.draw(
                        f.x() + f.width() / 2 - img.width() / 2, 
                        f.y() + f.height() / 2 - img.height() / 2, 
                        f.width(), f.height());
                    f.redraw();
                },
                None => { 
                    f.set_label("");
                }
            }
        });
        self.max_height += frame_img.height() + PADDING;

        match step_type {
            StepType::LinearFilter(size) => {
                self.steps.push(ProcessingStep::new(frame_img, label, StepAction::Linear(LinearFilter::mean_filter_of_size(size))))
            },
            StepType::MedianFilter(size) => {
                self.steps.push(ProcessingStep::new(frame_img, label, StepAction::Median(MedianFilter::new(size))))
            }
        };
    }

    pub fn process_message(&mut self, msg: Message) -> result::Result<(), MyError> {
        match msg {
            Message::LoadImage => {
                let mut dlg = dialog::FileDialog::new(dialog::FileDialogType::BrowseFile);
                dlg.show();
                let path_buf = dlg.filename();

                let init_image = img::Img::load(path_buf)?;

                let mut bmp_copy = init_image.get_bmp_copy()?;
                bmp_copy.scale(0, 0, true, true);
                self.frame_img.set_image(Some(bmp_copy));
                self.frame_img.redraw();

                self.initial_img = Some(init_image);
            }
            Message::DoStep { step_num } => {
                assert!(self.steps.len() > step_num);

                if step_num == 0 {
                    match self.initial_img {
                        Some(ref img) => {
                            let img_copy = img.clone();
                            self.steps[step_num].process_image(img_copy)?;
                        },
                        None => return Err(MyError::new("Необходимо загрузить изображение для обработки".to_string()))
                    }
                } else {
                    let prev_step = &self.steps[step_num - 1];
                    match prev_step.get_data_copy() {
                        Some(img) => {
                            self.steps[step_num].process_image(img)?;
                        },
                        None => return Err(MyError::new("Необходим результат предыдущего шага для обработки текущего".to_string()))
                    }
                }
            }
            Message::AddStep => {
                self.scroll_area.begin();
                self.add(StepType::LinearFilter(3), self.sender);
                self.scroll_area.end();
            }
        }

        Ok(())
    }

    pub fn end(&self) {
        self.scroll_area.end();
    }
}

struct ProcessingStep {
    name: String,
    frame: Frame,
    label: Frame,
    action: StepAction,
    image: Option<img::Img>,
    draw_data: Option<fltk::image::BmpImage>
}

impl ProcessingStep {
    fn new(frame: Frame, mut label: Frame, filter: StepAction) -> Self {
        let name = match filter {
            StepAction::Linear(_) => "Линейный фильтр".to_string(),
            StepAction::Median(_) => "Медианный фильтр".to_string()
        };

        label.set_label(&name);
        
        ProcessingStep { 
            name,
            frame, 
            label,
            action: filter,
            image: None, 
            draw_data: None 
        }
    }

    pub fn get_data_copy(&self) -> Option<img::Img> {
       self.image.clone()
    }

    pub fn process_image(&mut self, ititial_img: img::Img) -> result::Result<(), MyError> {
        let result_img = match self.action {
            StepAction::Linear(ref mut filter) => ititial_img.apply_filter(filter),
            StepAction::Median(ref mut filter) => ititial_img.apply_filter(filter),
        };

        let fil_size = match self.action {
            StepAction::Linear(ref f) => (f.w(), f.h()),
            StepAction::Median(ref f) => (f.window_size(), f.window_size())
        };
        self.label.set_label(&format!("{} {}x{}, изображение {}x{}", 
            &self.name, fil_size.0, fil_size.1, result_img.w(), result_img.h()));
                        
        let mut bmp_image = result_img.get_bmp_copy()?;
        bmp_image.scale(0, 0, true, true);
        self.frame.set_image(Some(bmp_image.clone()));
        self.frame.redraw();

        self.draw_data = Some(bmp_image);

        self.image = Some(result_img);

        Ok(())
    }
}

