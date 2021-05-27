use std::result;
use fltk::{app::{self, Receiver}, button, dialog, enums::{Align, FrameType}, frame::{self, Frame}, group::{self, PackType}, prelude::{GroupExt, ImageExt, WidgetBase, WidgetExt}, window};
use crate::{filter::{Filter, LinearFilter, MedianFilter}, img, my_app::{Message}, my_err::MyError, small_dlg::err_msg, step_editor::StepEditor};

pub enum StepType {
    LinearMeanFilter (usize),
    LinearFilter { coeffs: Vec<f64>, width: usize, height: usize },
    MedianFilter (usize)
}

pub const PADDING: i32 = 3;
pub const BTN_WIDTH: i32 = 100;
pub const BTN_HEIGHT: i32 = 30;
pub const BTN_TEXT_PADDING: i32 = 10;
const LEFT_MENU_WIDTH: i32 = 200;

enum StepAction {
    Linear(LinearFilter),
    Median(MedianFilter),
}

pub struct ProcessingLine {
    initial_img: Option<img::Img>,
    frame_img: frame::Frame,
    steps: Vec<ProcessingStep>,
    w: i32, h: i32,
    scroll_pack: group::Pack,
    receiver: Receiver<Message>,
    step_editor: StepEditor
}

impl ProcessingLine {
    pub fn new(wind: window::Window, x: i32, y: i32, w: i32, h: i32) -> Self {
        wind.begin();

        let mut left_menu = group::Pack::default()
            .with_pos(x, y)
            .with_size(LEFT_MENU_WIDTH, h);
        left_menu.set_type(PackType::Vertical);

        let (sender, _) = app::channel::<Message>();

        let mut label = frame::Frame::default()
            .with_label("Редактирование шагов");
        let (lw, lh) = label.measure_label();
        label.set_size(std::cmp::min(lw, LEFT_MENU_WIDTH), lh);

        let mut btn_add_step = button::Button::default()
            .with_label("Добавить");
        {
            let (w, h) = btn_add_step.measure_label();
            btn_add_step.set_size(w + PADDING, h + PADDING);
        }
        btn_add_step = btn_add_step.below_of(&label, PADDING);
        btn_add_step.emit(sender, Message::AddStep);
    
        left_menu.end();

        let scroll_area = group::Scroll::default()
            .with_pos(x + LEFT_MENU_WIDTH, y)
            .with_size(w - LEFT_MENU_WIDTH, h);
        let scroll_pack = group::Pack::default()
            .with_pos(x + LEFT_MENU_WIDTH, y)
            .with_size(w - LEFT_MENU_WIDTH, h);

        frame::Frame::default()
            .with_size(w - LEFT_MENU_WIDTH, BTN_HEIGHT)
            .with_label("Загрузка изображения");

        let (sender, receiver) = app::channel::<Message>();

        let mut btn = button::Button::default()
            .with_size(BTN_WIDTH, BTN_HEIGHT)
            .with_label("Загрузить");
        btn.emit(sender, Message::LoadImage );
        
        {
            let (bw, bh) = btn.measure_label();
            btn.set_size(bw + BTN_TEXT_PADDING, bh + BTN_TEXT_PADDING);
        }
            
        let mut frame_img = frame::Frame::default()
            .with_size(w - LEFT_MENU_WIDTH, h - BTN_HEIGHT * 2);
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

        scroll_pack.end();
        scroll_area.end();

        wind.end();

        ProcessingLine {
            initial_img: None,
            frame_img,
            steps: Vec::<ProcessingStep>::new(),
            w, h,
            scroll_pack,
            receiver,
            step_editor: StepEditor::new()
        }
    }

    pub fn add(&mut self, step_type: StepType) -> () {
        self.scroll_pack.begin();

        let label = frame::Frame::default()
            .with_size(self.w - LEFT_MENU_WIDTH, BTN_HEIGHT);   

        let (sender, _) = app::channel::<Message>();

        let mut hpack = group::Pack::default()
            .with_size(self.w - LEFT_MENU_WIDTH, BTN_HEIGHT); 
        hpack.set_type(PackType::Horizontal);
        hpack.set_spacing(PADDING);

        let mut btn_process = button::Button::default();
        btn_process.set_label("Отфильтровать");
        btn_process.emit(sender, Message::DoStep { step_num: self.steps.len() } );
        let (w, h) = btn_process.measure_label();
        btn_process.set_size(w + BTN_TEXT_PADDING, h + BTN_TEXT_PADDING);

        let mut btn_edit_step = button::Button::default();
        btn_edit_step.set_label("Изменить");
        btn_edit_step.emit(sender, Message::EditStep { step_num: self.steps.len() } );
        let (w, h) = btn_edit_step.measure_label();
        btn_edit_step.set_size(w + BTN_TEXT_PADDING, h + BTN_TEXT_PADDING);

        let mut btn_del_step = button::Button::default();
        btn_del_step.set_label("Удалить");
        btn_del_step.emit(sender, Message::DeleteStep { step_num: self.steps.len() } );
        let (w, h) = btn_del_step.measure_label();
        btn_del_step.set_size(w + BTN_TEXT_PADDING, h + BTN_TEXT_PADDING);

        hpack.end();
            
        let mut frame_img = frame::Frame::default()
            .with_size(self.w - LEFT_MENU_WIDTH, self.h - BTN_HEIGHT * 2);
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

        match step_type {
            StepType::LinearFilter { coeffs, width, height } => {
                self.steps.push(ProcessingStep::new(frame_img, label, StepAction::Linear(LinearFilter::with_coeffs(coeffs, width, height))))
            },
            StepType::MedianFilter(size) => {
                self.steps.push(ProcessingStep::new(frame_img, label, StepAction::Median(MedianFilter::new(size))))
            }
            StepType::LinearMeanFilter(size) => {
                self.steps.push(ProcessingStep::new(frame_img, label, StepAction::Linear(LinearFilter::mean_filter_of_size(size))))
            }
        };

        self.scroll_pack.end();
    }

    pub fn end(&self) {
        self.scroll_pack.end();
    }

    pub fn run(&mut self, app: app::App) -> result::Result<(), MyError> {
        while app.wait() {
            if let Some(msg) = self.receiver.recv() {
                match msg {
                    Message::LoadImage => match self.try_load() {
                        Ok(_) => {}
                        Err(err) => err_msg(&self.scroll_pack, &err.to_string())
                    }
                    Message::DoStep { step_num } => match self.try_do_step(step_num) {
                        Ok(_) => {}
                        Err(err) => err_msg(&self.scroll_pack, &err.to_string())
                    }
                    Message::AddStep => self.add_step(app),
                    Message::EditStep { step_num } => {
                        println!("Edit {}", step_num);
                    },
                    Message::DeleteStep { step_num } => {
                        println!("Delete {}", step_num);
                    }
                }
            }
        }
    
        Ok(())
    }

    fn try_load(&mut self) -> result::Result<(), MyError> {
        let mut dlg = dialog::FileDialog::new(dialog::FileDialogType::BrowseFile);
        dlg.show();
        let path_buf = dlg.filename();

        let init_image = img::Img::load(path_buf)?;

        let mut bmp_copy = init_image.get_bmp_copy()?;
        bmp_copy.scale(0, 0, true, true);
        self.frame_img.set_image(Some(bmp_copy));
        self.frame_img.redraw();

        self.initial_img = Some(init_image);

        Ok(())
    }

    fn try_do_step(&mut self, step_num: usize) -> result::Result<(), MyError> {
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

        Ok(())
    }

    fn add_step(&mut self, app: app::App) -> () {
        match self.step_editor.add_linear_filter_with_dlg(app) {
            Some(step_type) => self.add(step_type),
            None => {}
        }
    }
}

pub struct ProcessingStep {
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

