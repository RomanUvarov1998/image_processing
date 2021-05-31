use std::{fs, path::PathBuf, result};
use chrono::{Local, format::{DelayedFormat, StrftimeItems}};
use fltk::{app::{self, Receiver}, button, dialog, enums::{Align, FrameType, Shortcut}, frame::{self, Frame}, group::{self, PackType}, image::RgbImage, menu, prelude::{GroupExt, ImageExt, MenuExt, WidgetExt}, window};
use crate::{filter::{Filter, HistogramLocalContrast, LinearCustom, LinearMean, MedianFilter, LinearGaussian}, img::{self}, my_app::{Message}, my_err::MyError, small_dlg::{self, err_msg}, step_editor::StepEditor};

pub const PADDING: i32 = 3;
pub const BTN_WIDTH: i32 = 100;
pub const BTN_HEIGHT: i32 = 30;
pub const BTN_TEXT_PADDING: i32 = 10;
const LEFT_MENU_WIDTH: i32 = 200;

#[derive(Clone)]
pub enum StepAction {
    LinearCustom(LinearCustom),
    LinearMean(LinearMean),
    LinearGauss(LinearGaussian),
    Median(MedianFilter),
    HistogramLocalContrast(HistogramLocalContrast)
}
impl StepAction {
    fn edit_action_with_dlg(&self, app: app::App, step_editor: &mut StepEditor) -> StepAction {
        match self {
            StepAction::LinearCustom(old_filter) => {
                let res = step_editor.add_step_action_with_dlg(app, old_filter.clone());
                return match res {
                    Some(new_filter) => StepAction::LinearCustom(new_filter),
                    None => StepAction::LinearCustom(old_filter.clone()),
                };
            },
            StepAction::LinearMean(old_filter) => {
                let res = step_editor.add_step_action_with_dlg(app, old_filter.clone());
                match res {
                    Some(new_filter) => StepAction::LinearMean(new_filter),
                    None => StepAction::LinearMean(old_filter.clone()),
                }
            },
            StepAction::LinearGauss(old_filter) => {
                let res = step_editor.add_step_action_with_dlg(app, old_filter.clone());
                match res {
                    Some(new_filter) => StepAction::LinearGauss(new_filter),
                    None => StepAction::LinearGauss(old_filter.clone()),
                }
            },
            StepAction::Median(old_filter) => {
                let res = step_editor.add_step_action_with_dlg(app, old_filter.clone());
                match res {
                    Some(new_filter) => StepAction::Median(new_filter),
                    None => StepAction::Median(old_filter.clone()),
                }
            },
            StepAction::HistogramLocalContrast(old_filter) => {
                let res = step_editor.add_step_action_with_dlg(app, old_filter.clone());
                match res {
                    Some(new_filter) => StepAction::HistogramLocalContrast(new_filter),
                    None => StepAction::HistogramLocalContrast(old_filter.clone()),
                }
            },
        }
    }
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
        btn_add_step.add_emit("Линейный фильтр (усредняющий)", Shortcut::None, menu::MenuFlag::Normal, sender, Message::AddStepLinMean);
        btn_add_step.add_emit("Линейный фильтр (гауссовский)", Shortcut::None, menu::MenuFlag::Normal, sender, Message::AddStepLinGauss);
        btn_add_step.add_emit("Линейный фильтр (другой)", Shortcut::None, menu::MenuFlag::Normal, sender, Message::AddStepLinCustom);
        btn_add_step.add_emit("Медианный фильтр", Shortcut::None, menu::MenuFlag::Normal, sender, Message::AddStepMed);
        btn_add_step.add_emit("Локальный контраст (гистограмма)", Shortcut::None, menu::MenuFlag::Normal, 
            sender, Message::AddStepHistogramLocalContrast);

        btn_add_step.end();        

        let mut btn_save = button::Button::default();
        btn_save.set_label("Сохранить проект");
        btn_save.emit(sender, Message::SaveSession);
        {            
            let (w, h) = btn_save.measure_label();
            btn_save.set_size(w + BTN_TEXT_PADDING, h + BTN_TEXT_PADDING);
        }
    
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
                    Message::AddStepLinCustom => {
                        match self.step_editor.add_step_action_with_dlg(app, LinearCustom::default()) {
                            Some(filter) => self.add(StepAction::LinearCustom(filter)),
                            None => {}
                        }
                    },
                    Message::AddStepLinMean => {
                        match self.step_editor.add_step_action_with_dlg(app, LinearMean::default()) {
                            Some(filter) => self.add(StepAction::LinearMean(filter)),
                            None => {}
                        }
                    },
                    Message::AddStepLinGauss => {
                        match self.step_editor.add_step_action_with_dlg(app, LinearGaussian::default()) {
                            Some(filter) => self.add(StepAction::LinearGauss(filter)),
                            None => {}
                        }
                    },
                    Message::AddStepMed => {
                        match self.step_editor.add_step_action_with_dlg(app, MedianFilter::default()) {
                            Some(filter) => self.add(StepAction::Median(filter)),
                            None => {}
                        }
                    },
                    Message::AddStepHistogramLocalContrast => {
                        match self.step_editor.add_step_action_with_dlg(app, HistogramLocalContrast::default()) {
                            Some(filter) => self.add(StepAction::HistogramLocalContrast(filter)),
                            None => {}
                        }
                    },
                    Message::EditStep { step_num } => {
                        self.steps[step_num].action = match self.steps[step_num].action {
                            Some(ref action) => Some(action.edit_action_with_dlg(app, &mut self.step_editor)),
                            None => {
                                return Err(MyError::new("В данном компоненте нет фильтра".to_string()));
                            }
                        };
                    },
                    Message::DeleteStep { step_num } => {
                        self.scroll_pack.begin();
                        self.scroll_pack.remove(&self.steps[step_num].hpack);
                        self.scroll_pack.remove(&self.steps[step_num].btn_process);
                        self.scroll_pack.remove(&self.steps[step_num].btn_edit);
                        self.scroll_pack.remove(&self.steps[step_num].btn_delete);
                        self.scroll_pack.remove(&self.steps[step_num].frame_img);
                        self.scroll_pack.remove(&self.steps[step_num].label_step_name);
                        self.scroll_pack.end();
                        self.steps.remove(step_num);
                        
                        let (sender, _) = app::channel::<Message>();

                        for i in step_num..self.steps.len() {
                            self.steps[i].btn_process.emit(sender, Message::DoStep { step_num: i } );
                            self.steps[i].btn_edit.emit(sender, Message::EditStep { step_num: i } );
                            self.steps[i].btn_delete.emit(sender, Message::DeleteStep { step_num: i } );
                            self.steps[i].label_step_name.redraw_label();
                            self.steps[i].frame_img.set_damage(true);
                        }
                        self.scroll_pack.top_window().unwrap().set_damage(true);
                    }
                    Message::SaveSession => {
                        match Self::try_save_project() {
                            Ok(_) => {},
                            Err(_) => {},
                        }
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

    fn try_save_project() -> result::Result<(), MyError> {
        let mut chooser = dialog::FileChooser::new(
            ".","*", dialog::FileChooserType::Directory, 
            "Выберите папку для сохранения");
        chooser.show();

        while chooser.shown() { app::wait(); }

        if chooser.value(1).is_none() {
            return Ok(());
        }

        let mut path = chooser.directory().unwrap();
        println!("{}", &path);        

        let current_datetime_formatter: DelayedFormat<StrftimeItems> = Local::now().format("%d-%m(%b)-%Y_%a_%_H.%M.%S"); 
        let dir_name = format!("{}", current_datetime_formatter);

        path.push_str("/");
        path.push_str(&dir_name);
        println!("{}", &path);  
        
        match fs::create_dir(&path) {
            Ok(_) => println!("dir created"),
            Err(err) => println!("{}", &err.to_string())
        };

        // match path_buf.to_str() {
        //     Some(p) => if !p.is_empty() { }
        //     _ => {}
        // };   

        // println!("{}", path_buf.to_str().unwrap());
        // path_buf.push(dir_name);
        // println!("{}", path_buf.to_str().unwrap());
        // assert!(path_buf.is_dir());

        Ok(())
    }
}

pub struct ProcessingStep {
    name: String,
    hpack: group::Pack,
    btn_process: button::Button,
    btn_edit: button::Button,
    btn_delete: button::Button,
    label_step_name: Frame,
    frame_img: Frame,
    pub action: Option<StepAction>,
    image: Option<img::Img>,
    draw_data: Option<fltk::image::RgbImage>
}

impl ProcessingStep {
    fn new(proc_line: &ProcessingLine, filter: StepAction) -> Self {
        let name = match filter {
            StepAction::LinearCustom(_) => "Линейный фильтр".to_string(),
            StepAction::LinearMean(_) => "Линейный фильтр (усредняющий)".to_string(),
            StepAction::LinearGauss(_) => "Линейный фильтр (гауссовский)".to_string(),
            StepAction::Median(_) => "Медианный фильтр".to_string(),
            StepAction::HistogramLocalContrast(_) => "Локальный контраст (гистограмма)".to_string()
        };

        let label = frame::Frame::default()
            .with_size(proc_line.w - LEFT_MENU_WIDTH, BTN_HEIGHT)
            .with_label(&name);  

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

        let mut btn_edit = button::Button::default();
        btn_edit.set_label("Изменить");
        btn_edit.emit(sender, Message::EditStep { step_num: proc_line.steps.len() } );
        let (w, h) = btn_edit.measure_label();
        btn_edit.set_size(w + BTN_TEXT_PADDING, h + BTN_TEXT_PADDING);

        let mut btn_delete = button::Button::default();
        btn_delete.set_label("Удалить");
        btn_delete.emit(sender, Message::DeleteStep { step_num: proc_line.steps.len() } );
        let (w, h) = btn_delete.measure_label();
        btn_delete.set_size(w + BTN_TEXT_PADDING, h + BTN_TEXT_PADDING);

        hpack.end();
            
        let mut frame_img = frame::Frame::default()
            .with_size(proc_line.w - LEFT_MENU_WIDTH, proc_line.h - BTN_HEIGHT * 2);
        frame_img.set_frame(FrameType::EmbossedFrame);
        frame_img.set_align(Align::Center);    
        
        ProcessingStep { 
            name,            
            hpack,
            btn_process, btn_edit, btn_delete,
            frame_img, 
            label_step_name: label,
            action: Some(filter),
            image: None, 
            draw_data: None 
        }
    }

    pub fn get_data_copy(&self) -> Option<img::Img> {
       self.image.clone()
    }

    pub fn process_image(&mut self, initial_img: img::Img) -> result::Result<(), MyError> {
        let (result_img, fil_w, fil_h) = match self.action {
            Some(ref mut action) => {
                match action {
                    StepAction::LinearCustom(ref mut filter) => 
                        (initial_img.processed_copy(filter), filter.w(), filter.h()),
                        StepAction::LinearMean(ref mut filter) => 
                        (initial_img.processed_copy(filter), filter.w(), filter.h()),
                        StepAction::LinearGauss(ref mut filter) => 
                        (initial_img.processed_copy(filter), filter.w(), filter.h()),
                    StepAction::Median(ref mut filter) => 
                        (initial_img.processed_copy(filter), filter.w(), filter.h()),
                    StepAction::HistogramLocalContrast(ref mut filter) => 
                        (initial_img.processed_copy(filter), filter.w(), filter.h()),
                }
            },
            None =>  return Err(MyError::new("В данном компоненте нет фильтра".to_string())) 
        };
        
        self.label_step_name.set_label(&format!("{} {}x{}, изображение {}x{}", 
            &self.name, fil_w, fil_h, result_img.w(), result_img.h()));
                        
        let mut rgb_image: fltk::image::RgbImage = result_img.get_drawable_copy()?;
        rgb_image.scale(self.frame_img.w(), self.frame_img.h(), true, true);
        self.frame_img.set_image(Some(rgb_image.clone()));
        self.frame_img.redraw();

        self.draw_data = Some(rgb_image);

        self.image = Some(result_img);

        Ok(())
    }
}

