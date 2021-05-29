use std::result;
use fltk::{app::{self, Receiver}, button, dialog, enums::{Align, FrameType, Shortcut}, frame::{self, Frame}, group::{self, PackType}, image::RgbImage, menu, prelude::{GroupExt, ImageExt, MenuExt, WidgetExt}, window};
use crate::{filter::{Filter, LinearFilter, MedianFilter}, img::{self, Img}, my_app::{Message}, my_err::MyError, small_dlg::{self, err_msg}, step_editor::StepEditor};

pub const PADDING: i32 = 3;
pub const BTN_WIDTH: i32 = 100;
pub const BTN_HEIGHT: i32 = 30;
pub const BTN_TEXT_PADDING: i32 = 10;
const LEFT_MENU_WIDTH: i32 = 200;

#[derive(Clone)]
pub enum StepAction {
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

        let mut btn_add_step = menu::MenuButton::default()
            .with_label("Добавить");
        {
            let (w, h) = btn_add_step.measure_label();
            btn_add_step.set_size(w + PADDING, h + PADDING);
        }
        btn_add_step = btn_add_step.below_of(&label, PADDING);
        btn_add_step.add_emit("Линейный фильтр", Shortcut::None, menu::MenuFlag::Normal, sender, Message::AddStepLin);
        btn_add_step.add_emit("Медианный фильтр", Shortcut::None, menu::MenuFlag::Normal, sender, Message::AddStepMed);

        btn_add_step.end();
    
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
        frame_img.set_align(Align::Center);   

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

    pub fn add(&mut self, step_action: StepAction) -> () {
        self.scroll_pack.begin();

        self.steps.push(ProcessingStep::new(&self, step_action));

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
                    Message::AddStepLin => {
                        match self.step_editor.add_step_action_with_dlg(
                            app, 
                            StepAction::Linear(LinearFilter::default()))
                        {
                            Some(step_action) => self.add(step_action),
                            None => {}
                        }
                    },
                    Message::AddStepMed => {
                        match self.step_editor.add_step_action_with_dlg(
                            app, 
                            StepAction::Median(MedianFilter::default())) 
                        {
                            Some(step_action) => self.add(step_action),
                            None => {}
                        }
                    },
                    Message::EditStep { step_num } => {
                        match self.steps[step_num].action {
                            Some(ref action) => {
                                match self.step_editor.add_step_action_with_dlg(app, action.clone()) 
                                {
                                    Some(edited_action) => {
                                        self.steps[step_num].action = Some(edited_action);
                                    },
                                    None => {}
                                }
                            },
                            None => {
                                return Err(MyError::new("В данном компоненте нет фильтра".to_string()));
                            }
                        }
                    },
                    Message::DeleteStep { step_num } => {
                        self.scroll_pack.begin();
                        self.scroll_pack.remove(&self.steps[step_num].hpack);
                        self.scroll_pack.remove(&self.steps[step_num].btn_process);
                        self.scroll_pack.remove(&self.steps[step_num].btn_edit_step);
                        self.scroll_pack.remove(&self.steps[step_num].btn_del_step);
                        self.scroll_pack.remove(&self.steps[step_num].frame_img);
                        self.scroll_pack.remove(&self.steps[step_num].label);
                        self.scroll_pack.end();
                        self.steps.remove(step_num);
                        
                        let (sender, _) = app::channel::<Message>();

                        for i in step_num..self.steps.len() {
                            self.steps[i].btn_process.emit(sender, Message::DoStep { step_num: i } );
                            self.steps[i].btn_edit_step.emit(sender, Message::EditStep { step_num: i } );
                            self.steps[i].btn_del_step.emit(sender, Message::DeleteStep { step_num: i } );
                            self.steps[i].label.redraw_label();
                            self.steps[i].frame_img.set_damage(true);
                        }
                        self.scroll_pack.top_window().unwrap().set_damage(true);
                    }
                }
            }
        }
    
        Ok(())
    }

    fn try_load(&mut self) -> result::Result<(), MyError> {
        if self.initial_img.is_some() {
            if small_dlg::confirm(&self.scroll_pack, "Для открытия нового изображения нужно удалить предыдущие результаты. Продолжить?") {
                for step_num in 0..self.steps.len() {
                    self.steps[step_num].frame_img.set_image(Option::<RgbImage>::None);
                }
            } else {
                return Ok(());
            }
        }

        let mut dlg = dialog::FileDialog::new(dialog::FileDialogType::BrowseFile);
        dlg.show();
        let path_buf = dlg.filename();

        match path_buf.to_str() {
            Some(p) => if p.is_empty() { return Ok(()); }
            _ => {}
        }        

        let init_image = img::Img::load(path_buf)?;

        let mut img_copy = init_image.get_drawable_copy()?;
        img_copy.scale(self.frame_img.w(), self.frame_img.h(), true, true);
        self.frame_img.set_image(Some(img_copy.clone()));
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
}

pub struct ProcessingStep {
    name: String,
    hpack: group::Pack,
    btn_process: button::Button,
    btn_edit_step: button::Button,
    btn_del_step: button::Button,
    label: Frame,
    frame_img: Frame,
    pub action: Option<StepAction>,
    image: Option<img::Img>,
    draw_data: Option<fltk::image::RgbImage>
}

impl ProcessingStep {
    fn new(proc_line: &ProcessingLine, filter: StepAction) -> Self {
        let name = match filter {
            StepAction::Linear(_) => "Линейный фильтр".to_string(),
            StepAction::Median(_) => "Медианный фильтр".to_string()
        };

        let mut label = frame::Frame::default()
            .with_size(proc_line.w - LEFT_MENU_WIDTH, BTN_HEIGHT);   

        let (sender, _) = app::channel::<Message>();

        let mut hpack = group::Pack::default()
            .with_size(proc_line.w - LEFT_MENU_WIDTH, BTN_HEIGHT); 
        hpack.set_type(PackType::Horizontal);
        hpack.set_spacing(PADDING);

        let mut btn_process = button::Button::default();
        btn_process.set_label("Отфильтровать");
        btn_process.emit(sender, Message::DoStep { step_num: proc_line.steps.len() } );
        let (w, h) = btn_process.measure_label();
        btn_process.set_size(w + BTN_TEXT_PADDING, h + BTN_TEXT_PADDING);

        let mut btn_edit_step = button::Button::default();
        btn_edit_step.set_label("Изменить");
        btn_edit_step.emit(sender, Message::EditStep { step_num: proc_line.steps.len() } );
        let (w, h) = btn_edit_step.measure_label();
        btn_edit_step.set_size(w + BTN_TEXT_PADDING, h + BTN_TEXT_PADDING);

        let mut btn_del_step = button::Button::default();
        btn_del_step.set_label("Удалить");
        btn_del_step.emit(sender, Message::DeleteStep { step_num: proc_line.steps.len() } );
        let (w, h) = btn_del_step.measure_label();
        btn_del_step.set_size(w + BTN_TEXT_PADDING, h + BTN_TEXT_PADDING);

        hpack.end();
            
        let mut frame_img = frame::Frame::default()
            .with_size(proc_line.w - LEFT_MENU_WIDTH, proc_line.h - BTN_HEIGHT * 2);
        frame_img.set_frame(FrameType::EmbossedFrame);
        frame_img.set_align(Align::Center);    

        label.set_label(&name);
        
        ProcessingStep { 
            name,            
            hpack,
            btn_process,
            btn_edit_step,
            btn_del_step,
            frame_img, 
            label,
            action: Some(filter),
            image: None, 
            draw_data: None 
        }
    }

    pub fn get_data_copy(&self) -> Option<img::Img> {
       self.image.clone()
    }

    pub fn process_image(&mut self, ititial_img: img::Img) -> result::Result<(), MyError> {
        let fil_size: (usize, usize);

        let result_img: Img;
        match self.action {
            Some(ref mut action) => {
                match action {
                    StepAction::Linear(ref mut filter) => {
                        fil_size = (filter.w(), filter.h());
                        result_img = ititial_img.apply_filter(filter);
                    },
                    StepAction::Median(ref mut filter) => {
                        fil_size = (filter.window_size(), filter.window_size());
                        result_img = ititial_img.apply_filter(filter);
                    }
                }
            },
            None => {
                return Err(MyError::new("В данном компоненте нет фильтра".to_string()));
            }
        };
        
        self.label.set_label(&format!("{} {}x{}, изображение {}x{}", 
            &self.name, fil_size.0, fil_size.1, result_img.w(), result_img.h()));
                        
        let mut rgb_image: fltk::image::RgbImage = result_img.get_drawable_copy()?;
        rgb_image.scale(self.frame_img.w(), self.frame_img.h(), true, true);
        self.frame_img.set_image(Some(rgb_image.clone()));
        self.frame_img.redraw();

        self.draw_data = Some(rgb_image);

        self.image = Some(result_img);

        Ok(())
    }
}

