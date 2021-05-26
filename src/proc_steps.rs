use std::result;
use fltk::{app::Sender, button, dialog, enums::{Align, FrameType}, frame::{self, Frame}, prelude::{ImageExt, WidgetBase, WidgetExt}};
use crate::{filter::{LinearFilter}, img, my_err::MyError};
use ::image as ImgLib;

pub enum StepTypes {
    LoadImage,
    LinearFilter (usize),
    MedianFilter (usize)
}

pub const PADDING: i32 = 3;
pub const WIN_WIDTH: i32 = 640;
pub const WIN_HEIGHT: i32 = 480;
pub const BTN_WIDTH: i32 = 100;
pub const BTN_HEIGHT: i32 = 30;

#[derive(Debug, Copy, Clone)]
pub enum Message {
    LoadInitialImage,
    ProcessLoadedImage { step_num: usize },
}

pub struct ProcessingLine {
    steps: Vec<ProcessingStep>,
    max_height: i32
}

impl ProcessingLine {
    pub fn new() -> Self {
        ProcessingLine {
            steps: Vec::<ProcessingStep>::new(),
            max_height: 0_i32
        }
    }

    pub fn add(&mut self, step_type: StepTypes, sender: Sender<Message>) -> () {
        if self.steps.len() == 0 {
            match step_type {
                StepTypes::LoadImage => { }
                _ => panic!("The first step must be StepTypes::LoadImage")
            }
        }

        let mut label = frame::Frame::default()
            .with_pos(PADDING, self.max_height)
            .with_size(WIN_WIDTH, BTN_HEIGHT);   
        self.max_height += label.height() + PADDING;
    
        match step_type {
            StepTypes::LoadImage => label.set_label("Загрузка изображения"),
            StepTypes::LinearFilter(_) => label.set_label("Линейный фильтр"),
            StepTypes::MedianFilter(_) => label.set_label("Медианный фильтр"),
        };

        const BTN_TEXT_PADDING: i32 = 10;
        let mut btn = button::Button::default()
            .with_size(BTN_WIDTH, BTN_HEIGHT)
            .with_pos(PADDING,  self.max_height);

        match step_type {
            StepTypes::LoadImage => {
                btn.set_label("Загрузка изображения");
                btn.emit(sender, Message::LoadInitialImage);
            },
            StepTypes::LinearFilter(_) => {
                btn.set_label("Отфильтровать");
                btn.emit(sender, Message::ProcessLoadedImage { step_num: self.steps.len() } );
            },
            StepTypes::MedianFilter(_) => {
                btn.set_label("Отфильтровать");
            }
        };

        let (w, h) = btn.measure_label();
        btn.set_size(w + BTN_TEXT_PADDING, h + BTN_TEXT_PADDING);

        self.max_height += btn.height() + PADDING;
            
        let mut frame_img = frame::Frame::default()
            .with_pos(PADDING,  self.max_height)
            .with_size(WIN_WIDTH - PADDING * 2, WIN_HEIGHT - BTN_HEIGHT * 2);
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
            StepTypes::LoadImage => {
                self.steps.push(ProcessingStep::new(frame_img, label, None))
            },
            StepTypes::LinearFilter(size) => {
                self.steps.push(ProcessingStep::new(frame_img, label, Some(LinearFilter::mean_filter_of_size(size))))
            },
            StepTypes::MedianFilter(size) => {
                self.steps.push(ProcessingStep::new(frame_img, label, Some(LinearFilter::mean_filter_of_size(size))))
            }
        };
    }

    pub fn process_from_step(&mut self, step: Message) -> result::Result<(), MyError> {
        match step {
            Message::LoadInitialImage => {
                let loaded_img = Self::load_img()?;
                assert!(self.steps.len() > 0);
                self.steps[0].set_image(loaded_img)?;

                assert!(self.steps[0].has_data());

                let mut result = self.steps[0].get_data();
                for ind in 1..self.steps.len() {
                    result = self.steps[ind].process(result)?;
                    self.steps[ind].set_image(result.clone())?;
                }
            }
            Message::ProcessLoadedImage { step_num } => {
                assert!(self.steps.len() > step_num);
                if !self.steps[step_num].has_data() {
                    return Err(MyError::new("This step has no image data".to_string()));
                }

                let mut result = self.steps[step_num].get_data();
                for ind in step_num..self.steps.len() {
                    result = self.steps[ind].process(result)?;
                    self.steps[ind].set_image(result.clone())?;
                }
            }
        }

        Ok(())
    }

    fn load_img() -> result::Result<img::Img, MyError> {
        let mut dlg = dialog::FileDialog::new(dialog::FileDialogType::BrowseFile);
        dlg.show();
        let path_buf = dlg.filename();
    
        let path_str = path_buf.to_str().unwrap();
        let img_dyn = ImgLib::io::Reader::open(path_str.to_string())?.decode()?;    
    
        Ok(img::Img::new(img_dyn))
    }
}

struct ProcessingStep {
    frame: Frame,
    label: Frame,
    filter: Option<LinearFilter>,
    image: Option<img::Img>,
    draw_data: Option<fltk::image::BmpImage>
}

impl ProcessingStep {
    fn new(frame: Frame, label: Frame, filter: Option<LinearFilter>) -> Self {
        ProcessingStep { 
            frame, 
            label,
            filter,
            image: None, 
            draw_data: None 
        }
    }

    pub fn has_data(&self) -> bool { 
        self.image.is_some() && self.draw_data.is_some() 
    }

    pub fn get_data(&mut self) -> img::Img {
       self.image.take().unwrap()
    }

    pub fn set_image(&mut self, image: img::Img) -> result::Result<(), MyError> {
        self.image = Some(image.clone());

        let mut bmp_image = image.give_image()?;

        self.label.set_label(&format!("Размер {}x{}", bmp_image.w(), bmp_image.h()));
                        
        bmp_image.scale(0, 0, true, true);
        self.frame.set_image(Some(bmp_image.clone()));
        self.frame.redraw();

        self.draw_data = Some(bmp_image);

        Ok(())
    }

    pub fn process(&mut self, mut image: img::Img) -> result::Result<img::Img, MyError> {
        match self.filter {
            Some(ref mut filter) => Ok(image.apply_filter(filter)),
            None => Err(MyError::new("There is no filter in this step".to_string()))
        }
    }
}

