use std::path::{PathBuf};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::{thread};
use std::{fs::{self, File}, io::{Read, Write}, result};
use chrono::{Local, format::{DelayedFormat, StrftimeItems}};
use fltk::{app::{self, Receiver}, button, dialog, enums::{Align, FrameType, Shortcut}, frame::{self, Frame}, group::{self, PackType}, image::RgbImage, menu, prelude::{GroupExt, ImageExt, MenuExt, WidgetExt}, window};
use crate::filter::filter_trait::{StringFromTo};
use crate::img::Matrix2D;
use crate::{filter::{linear::{LinearCustom, LinearGaussian, LinearMean}, non_linear::{MedianFilter, HistogramLocalContrast, CutBrightness}}, img::{self}, my_app::{Message}, my_err::MyError, small_dlg::{self, confirm, err_msg, info_msg}, step_editor::StepEditor};

pub const PADDING: i32 = 3;
pub const BTN_WIDTH: i32 = 100;
pub const BTN_HEIGHT: i32 = 30;
pub const BTN_TEXT_PADDING: i32 = 10;

#[derive(Clone)]
pub enum StepAction {
    LinearCustom(LinearCustom),
    LinearMean(LinearMean),
    LinearGauss(LinearGaussian),
    Median(MedianFilter),
    HistogramLocalContrast(HistogramLocalContrast),
    CutBrightness(CutBrightness)
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
            StepAction::CutBrightness(old_filter) => {
                let res = step_editor.add_step_action_with_dlg(app, old_filter.clone());
                match res {
                    Some(new_filter) => StepAction::CutBrightness(new_filter),
                    None => StepAction::CutBrightness(old_filter.clone()),
                }
            },          
        }
    }

    fn try_parce(string: &str) -> Option<Self> {
        if let Ok(filter) = LinearCustom::try_from_string(string) {
            Some(StepAction::LinearCustom(filter))
        } 
        else if let Ok(filter) = LinearMean::try_from_string(string) {
            Some(StepAction::LinearMean(filter))
        } 
        else if let Ok(filter) = LinearGaussian::try_from_string(string) {
            Some(StepAction::LinearGauss(filter))
        } 
        else if let Ok(filter) = LinearMean::try_from_string(string) {
            Some(StepAction::LinearMean(filter))
        } 
        else if let Ok(filter) = HistogramLocalContrast::try_from_string(string) {
            Some(StepAction::HistogramLocalContrast(filter))
        } 
        else {
            None
        }
    }
}

pub struct ProcessingLine<'wind> {
    parent_window: &'wind window::Window,
    initial_img: Option<img::Matrix2D>,
    frame_img: frame::Frame,
    steps: Vec<ProcessingStep>,
    w: i32, h: i32,
    scroll_pack: group::Pack,    
    receiver: Receiver<Message>,
    step_editor: StepEditor,
    processing_data: Arc<Mutex<Option<ProcessingData>>>,
    processing_thread: JoinHandle<()>
}

impl<'wind> ProcessingLine<'wind> {
    pub fn new(wind: &'wind window::Window, x: i32, y: i32, w: i32, h: i32) -> Self {
        wind.begin();

        let (sender, receiver) = app::channel::<Message>();

        let mut menu = menu::SysMenuBar::default().with_size(800, 35);
        menu.add_emit("Проект/Зарузить", Shortcut::None, menu::MenuFlag::Normal, sender, Message::LoadProject);
        menu.add_emit("Проект/Сохранить как", Shortcut::None, menu::MenuFlag::Normal, sender, Message::SaveProject);
        menu.add_emit("Добавить/Линейный фильтр (усредняющий)", Shortcut::None, menu::MenuFlag::Normal, sender, Message::AddStepLinMean);
        menu.add_emit("Добавить/Линейный фильтр (гауссовский)", Shortcut::None, menu::MenuFlag::Normal, sender, Message::AddStepLinGauss);
        menu.add_emit("Добавить/Линейный фильтр (другой)", Shortcut::None, menu::MenuFlag::Normal, sender, Message::AddStepLinCustom);
        menu.add_emit("Добавить/Медианный фильтр", Shortcut::None, menu::MenuFlag::Normal, sender, Message::AddStepMed);
        menu.add_emit("Добавить/Локальный контраст (гистограмма)", Shortcut::None, menu::MenuFlag::Normal, sender, Message::AddStepHistogramLocalContrast);
        menu.add_emit("Добавить/Обрезание яркости", Shortcut::None, menu::MenuFlag::Normal, sender, Message::AddStepCutBrightness);
        menu.add_emit("Экспорт/Сохранить результаты", Shortcut::None, menu::MenuFlag::Normal, sender, Message::SaveResults);
        menu.end();
        
        let scroll_area = group::Scroll::default()
            .with_pos(x, y + menu.h())
            .with_size(w, h - menu.h());

        let scroll_pack = group::Pack::default()
            .with_pos(x, y + menu.h())
            .with_size(w, h - menu.h());
            
        frame::Frame::default()
            .with_size(w, BTN_HEIGHT)
            .with_label("Исходное изображение");

        let mut btn_load_initial_img = button::Button::default()
            .with_size(BTN_WIDTH, BTN_HEIGHT)
            .with_label("Загрузить");
        btn_load_initial_img.emit(sender, Message::LoadImage );        
        {
            let (bw, bh) = btn_load_initial_img.measure_label();
            btn_load_initial_img.set_size(bw + BTN_TEXT_PADDING, bh + BTN_TEXT_PADDING);
        }
            
        let mut frame_img = frame::Frame::default()
            .with_size(w, h - BTN_HEIGHT * 2);
        frame_img.set_frame(FrameType::EmbossedFrame);
        frame_img.set_align(Align::Center);   

        scroll_pack.end();
        scroll_area.end();

        wind.end();

        let sender_copy = sender.clone();
        let processing_data = Arc::new(Mutex::new(Option::<ProcessingData>::None));
        let processing_data_copy = processing_data.clone();
        let processing_thread = thread::spawn(move || {
            thread::park();
            loop {
                println!("act");
                match processing_data_copy.try_lock() {
                    Ok(mut guard) => match guard.as_mut() {
                        Some(pd) => {
                            println!("thread completed");
                            let result_img = match pd.step_action {
                                StepAction::LinearCustom(ref mut filter) => 
                                    (pd.init_img.processed_copy(filter, pd.step_num, sender_copy)),
                                StepAction::LinearMean(ref mut filter) => 
                                    (pd.init_img.processed_copy(filter, pd.step_num, sender_copy)),
                                StepAction::LinearGauss(ref mut filter) => 
                                    (pd.init_img.processed_copy(filter, pd.step_num, sender_copy)),
                                StepAction::Median(ref mut filter) => 
                                    (pd.init_img.processed_copy(filter, pd.step_num, sender_copy)),
                                StepAction::HistogramLocalContrast(ref mut filter) => 
                                    (pd.init_img.processed_copy(filter, pd.step_num, sender_copy)),
                                StepAction::CutBrightness(ref mut filter) => 
                                    (pd.init_img.processed_copy(filter, pd.step_num, sender_copy)),
                            };
                            pd.result_img = Some(result_img);
                            sender_copy.send(Message::StepIsComplete { step_num: pd.step_num });
                            println!("thread completed");
                        },
                        None => {
                            println!("yield");
                            thread::yield_now();
                            continue;
                        }
                    },
                    Err(_) => {
                        println!("yield");
                        thread::yield_now();
                        continue;
                    }
                };
                thread::park();
                println!("resumed");
            }
        });

        ProcessingLine {
            parent_window: wind,
            initial_img: None,
            frame_img,
            steps: Vec::<ProcessingStep>::new(),
            w, h,
            scroll_pack,
            receiver,
            step_editor: StepEditor::new(),
            processing_data,
            processing_thread
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
                        Err(err) => err_msg(&self.parent_window, &err.to_string())
                    }
                    Message::StepIsStarted { step_num } => match self.try_do_step(step_num) {
                        Ok(_) => {}
                        Err(err) => err_msg(&self.parent_window, &err.to_string())
                    },
                    Message::StepProgress { step_num, cur_percents: progress } => {
                        self.steps[step_num].display_progress(progress);
                    },
                    Message::StepIsComplete { step_num } => {
                        self.steps[step_num].display_result(self.processing_data.clone())?;
                    },
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
                    Message::AddStepCutBrightness => {
                        match self.step_editor.add_step_action_with_dlg(app, CutBrightness::default()) {
                            Some(filter) => self.add(StepAction::CutBrightness(filter)),
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
                    Message::DeleteStep { step_num } => self.delete_step(step_num),
                    Message::SaveProject => {
                        match self.try_save_project() {
                            Ok(_) => info_msg(&self.parent_window, "Проект успешно сохранен"),
                            Err(err) => err_msg(&self.parent_window, &err.get_message()),
                        }
                    },
                    Message::LoadProject => {
                        match self.try_load_project() {
                            Ok(_) => info_msg(&self.parent_window, "Проект успешно загружен"),
                            Err(err) => err_msg(&self.parent_window, &err.get_message()),
                        }
                    },
                    Message::SaveResults => {
                        match self.try_save_results() {
                            Ok(_) => info_msg(&self.parent_window, "Результаты успешно сохранены"),
                            Err(err) => err_msg(&self.parent_window, &err.get_message()),
                        }

                    }
                }
            }
        }
    
        Ok(())
    }

    fn delete_step(&mut self, step_num: usize) {
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
            self.steps[i].btn_process.emit(sender, Message::StepIsStarted { step_num: i } );
            self.steps[i].btn_edit.emit(sender, Message::EditStep { step_num: i } );
            self.steps[i].btn_delete.emit(sender, Message::DeleteStep { step_num: i } );
            self.steps[i].label_step_name.redraw_label();
            self.steps[i].frame_img.set_damage(true);
        }
        self.scroll_pack.top_window().unwrap().set_damage(true);
    }

    fn try_load(&mut self) -> result::Result<(), MyError> {
        if self.initial_img.is_some() {
            if small_dlg::confirm(&self.parent_window, "Для открытия нового изображения нужно удалить предыдущие результаты. Продолжить?") {
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

        let init_image = img::Matrix2D::load(path_buf)?;

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
                    self.steps[step_num].setup_processing(self.processing_data.clone(), img_copy)?;
                },
                None => return Err(MyError::new("Необходимо загрузить изображение для обработки".to_string()))
            }
        } else {
            let prev_step = &self.steps[step_num - 1];
            match prev_step.get_data_copy() {
                Some(img_copy) => {
                    self.steps[step_num].setup_processing(self.processing_data.clone(), img_copy)?;
                },
                None => return Err(MyError::new("Необходим результат предыдущего шага для обработки текущего".to_string()))
            }
        }

        self.processing_thread.thread().unpark();

        Ok(())
    }

    fn try_save_project(&self) -> result::Result<(), MyError> {
        // check if there are any steps
        if self.steps.len() == 0 {
            return Err(MyError::new("В проекте нет шагов для сохранения".to_string()));
        }

        // check if all steps have filters defined
        let all_steps_have_filter = self.steps.iter().all(|s| s.action.is_some());
        if !all_steps_have_filter {
            return Err(MyError::new("У всех шагов должны быть заданы фильтры".to_string()));
        }

        // choose folder
        let mut chooser = dialog::FileChooser::new(
            ".","*", dialog::FileChooserType::Directory, 
            "Выберите папку для сохранения");
        chooser.show();
        while chooser.shown() { app::wait(); }
        if chooser.value(1).is_none() {
            return Ok(());
        }
        
        let mut path = chooser.directory().unwrap();       

        // create project folder
        let current_datetime_formatter: DelayedFormat<StrftimeItems> = Local::now().format("Project %d-%m(%b)-%Y_%a_%_H.%M.%S"); 
        let dir_name = format!("{}", current_datetime_formatter);

        path.push_str("/");
        path.push_str(&dir_name);
        
        match fs::create_dir(&path) {
            Ok(_) => {},
            Err(err) => { return Err(MyError::new(err.to_string())); },
        };

        // save all steps
        for step_num in 0..self.steps.len() {
            let filter_content: String = match self.steps[step_num].action.as_ref().unwrap() {
                StepAction::LinearCustom(ref filter) => filter.content_to_string(),
                StepAction::LinearMean(ref filter) => filter.content_to_string(),
                StepAction::LinearGauss(ref filter) => filter.content_to_string(),
                StepAction::Median(ref filter) => filter.content_to_string(),
                StepAction::HistogramLocalContrast(ref filter) => filter.content_to_string(),
                StepAction::CutBrightness(ref filter) => filter.content_to_string(),
            };

            let mut file_path = path.clone();
            file_path.push_str(&format!("/{}.txt", step_num + 1));

            let mut file = match File::create(file_path) {
                Ok(f) => f,
                Err(err) => { return Err(MyError::new(err.to_string())); }
            };

            file.write_all(&filter_content.as_bytes())?;
            file.sync_all()?;
        }

        Ok(())
    }
    
    fn try_load_project(&mut self) -> result::Result<(), MyError> {
        if self.steps.len() > 0 && confirm(self.parent_window,
             "Есть несохраненный проект. Открыть вместо него?") 
        {
            while self.steps.len() > 0 {
                self.delete_step(0);
            }
        }

        // choose folder
        let mut chooser = dialog::FileChooser::new(
            ".","*", dialog::FileChooserType::Directory, 
            "Выберите папку для загрузки");
        chooser.show();
        while chooser.shown() { app::wait(); }
        if chooser.value(1).is_none() {
            return Ok(());
        }
        
        let dir_path = chooser.directory().unwrap();    

        let mut step_num = 0;
        loop {
            let file_path_str = format!("{}/{}.txt", &dir_path, step_num + 1);
            let file_path = PathBuf::from(file_path_str);

            if !file_path.exists() { 
                break; 
            }

            let mut file = match File::open(file_path) {
                Ok(f) => f,
                Err(err) => { return Err(MyError::new(err.to_string())); }
            };

            let mut content = String::new();
            file.read_to_string(&mut content)?;

            if let Some(step_action) = StepAction::try_parce(&content) 
            {
                self.add(step_action);
            } 
            else 
            {
                if !confirm(self.parent_window, "Не удалось прочитать фильтр из файла. Оставить загруженное?
                    Да - оставить, Нет - удалить.")
                {
                    while self.steps.len() > 0 {
                        self.delete_step(0);
                    }
                } 
                return Ok(());
            }

            step_num += 1;
        }

        Ok(())
    }

    fn try_save_results(&self) -> result::Result<(), MyError> {
        // check if there are any steps
        if self.steps.len() == 0 {
            return Err(MyError::new("В проекте нет результатов для сохранения".to_string()));
        }

        // check if all steps have images
        let all_steps_have_image = self.steps.iter().all(|s| s.image.is_some());
        if !all_steps_have_image {
            return Err(MyError::new("В проекте нет результатов для сохранения".to_string()));
        }

        // choose folder
        let mut chooser = dialog::FileChooser::new(
            ".","*", dialog::FileChooserType::Directory, 
            "Выберите папку для сохранения");
        chooser.show();
        while chooser.shown() { app::wait(); }
        if chooser.value(1).is_none() {
            return Ok(());
        }
        
        let mut path = chooser.directory().unwrap();       

        // create project folder
        let current_datetime_formatter: DelayedFormat<StrftimeItems> = Local::now().format("Results %d-%m(%b)-%Y_%a_%_H.%M.%S"); 
        let dir_name = format!("{}", current_datetime_formatter);

        path.push_str("/");
        path.push_str(&dir_name);
        
        match fs::create_dir(&path) {
            Ok(_) => {},
            Err(err) => { return Err(MyError::new(err.to_string())); },
        };

        // save all images
        for step_num in 0..self.steps.len() {
            let mut file_path = path.clone();
            file_path.push_str(&format!("/{}.bmp", step_num + 1));

            self.steps[step_num].image.as_ref().unwrap().try_save(&file_path)?;
        }

        Ok(())
    }
}

struct ProcessingData {
    step_num: usize,
    step_action: StepAction,
    init_img: Matrix2D,
    result_img: Option<Matrix2D>,
}

impl ProcessingData {
    fn new(step_num: usize, step_action: StepAction, init_img: Matrix2D) -> Self {
        ProcessingData {
            step_num,
            step_action,
            init_img,
            result_img: None,
        }
    }

    fn get_result(&mut self) -> Option<Matrix2D> { self.result_img.take() }
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
    image: Option<img::Matrix2D>,
    draw_data: Option<fltk::image::RgbImage>,
    step_num: usize
}

impl ProcessingStep {
    fn new(proc_line: &ProcessingLine, filter: StepAction) -> Self {
        let name = match filter {
            StepAction::LinearCustom(_) => "Линейный фильтр".to_string(),
            StepAction::LinearMean(_) => "Линейный фильтр (усредняющий)".to_string(),
            StepAction::LinearGauss(_) => "Линейный фильтр (гауссовский)".to_string(),
            StepAction::Median(_) => "Медианный фильтр".to_string(),
            StepAction::HistogramLocalContrast(_) => "Локальный контраст (гистограмма)".to_string(),
            StepAction::CutBrightness(_) => "Вырезание яркости".to_string(),
        };

        let label = frame::Frame::default()
            .with_size(proc_line.w, BTN_HEIGHT)
            .with_label(&name);  

        let (sender, _) = app::channel::<Message>();

        let mut hpack = group::Pack::default()
            .with_size(proc_line.w, BTN_HEIGHT); 
        hpack.set_type(PackType::Horizontal);
        hpack.set_spacing(PADDING);

        let step_num = proc_line.steps.len();

        let mut btn_process = button::Button::default();
        btn_process.set_label("Отфильтровать");
        btn_process.emit(sender, Message::StepIsStarted { step_num } );
        let (w, h) = btn_process.measure_label();
        btn_process.set_size(w + BTN_TEXT_PADDING, h + BTN_TEXT_PADDING);

        let mut btn_edit = button::Button::default();
        btn_edit.set_label("Изменить");
        btn_edit.emit(sender, Message::EditStep { step_num } );
        let (w, h) = btn_edit.measure_label();
        btn_edit.set_size(w + BTN_TEXT_PADDING, h + BTN_TEXT_PADDING);

        let mut btn_delete = button::Button::default();
        btn_delete.set_label("Удалить");
        btn_delete.emit(sender, Message::DeleteStep { step_num } );
        let (w, h) = btn_delete.measure_label();
        btn_delete.set_size(w + BTN_TEXT_PADDING, h + BTN_TEXT_PADDING);

        hpack.end();
            
        let mut frame_img = frame::Frame::default()
            .with_size(proc_line.w, proc_line.h - BTN_HEIGHT * 2);
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
            draw_data: None,
            step_num
        }
    }

    pub fn get_data_copy(&self) -> Option<img::Matrix2D> {
       self.image.clone()
    }

    fn setup_processing(&mut self, processing_data: Arc<Mutex<Option<ProcessingData>>>, init_img: Matrix2D) -> Result<(), MyError> {
        match self.action {
            Some(ref step_action) => {
                processing_data.lock().unwrap().replace(ProcessingData::new(self.step_num, step_action.clone(), init_img));
                drop(processing_data);
            },
            None => { return Err(MyError::new("В данном шаге нет действия(((".to_string())); },
        };

        self.btn_process.deactivate();
        self.btn_edit.deactivate();
        self.btn_delete.deactivate();

        self.frame_img.deimage();
        self.frame_img.redraw();

        println!("{} started", self.step_num);

        Ok(())
    }

    fn display_progress(&mut self, progress: usize) {
        let mut pr_str = String::from("|");
        let mut i = 10;
        while i < progress {
            pr_str.push_str(".");
            i += 10;
        }
        while i < 100 {
            pr_str.push_str(" ");
            i += 10;
        }
        pr_str.push_str("|");
        self.frame_img.set_label(&pr_str);

        println!("{}: {}", self.step_num, progress);
    }

    fn display_result(&mut self, processing_data: Arc<Mutex<Option<ProcessingData>>>) -> Result<(), MyError>  {
        println!("{} complete", self.step_num);
        self.frame_img.set_label("");

        self.btn_process.activate();
        self.btn_edit.activate();
        self.btn_delete.activate();

        println!("getting data...");
        let pd_locked = processing_data.lock().unwrap().take();
        drop(processing_data);
        println!("got data");
        let result_img = match pd_locked {
            Some(mut p) => match p.get_result() {
                Some(img) => img,
                None => { return Err(MyError::new("Нет результирующего изображения(".to_string())); },
            },
            None => { return Err(MyError::new("Нет данных для обработки(".to_string())); },
        };
        
        self.label_step_name.set_label(&format!("{} {}x{}, изображение {}x{}", 
            &self.name, 1, 1, result_img.w(), result_img.h()));
                        
        let mut rgb_image: fltk::image::RgbImage = result_img.get_drawable_copy()?;
        rgb_image.scale(self.frame_img.w(), self.frame_img.h(), true, true);
        
        match self.frame_img.deimage() {
            Some(old_img) => drop(old_img),
            None => {},
        }
        
        self.frame_img.set_image(Some(rgb_image.clone()));
        self.frame_img.redraw();

        self.draw_data = Some(rgb_image);

        self.image = Some(result_img);

        Ok(())
    }
}

