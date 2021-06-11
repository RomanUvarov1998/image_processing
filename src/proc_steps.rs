use std::path::{PathBuf};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::{thread};
use std::{fs::{self, File}, io::{Read, Write}, result};
use chrono::{Local, format::{DelayedFormat, StrftimeItems}};
use fltk::app::{App, Sender};
use fltk::menu::MenuFlag;
use fltk::{app::{self, Receiver}, button, dialog, enums::{Align, FrameType, Shortcut}, frame::{self, Frame}, group::{self, PackType}, image::RgbImage, menu, prelude::{GroupExt, ImageExt, MenuExt, WidgetExt}, window};
use crate::filter::filter_trait::{Filter, StringFromTo};
use crate::img::Matrix2D;
use crate::message::{self, Message, Processing, Project, Step};
use crate::{filter::{linear::{LinearCustom, LinearGaussian, LinearMean}, non_linear::{MedianFilter, HistogramLocalContrast, CutBrightness}}, img::{self}, my_err::MyError, small_dlg::{self, confirm, err_msg, info_msg}, step_editor::StepEditor};

pub const PADDING: i32 = 3;
pub const BTN_WIDTH: i32 = 100;
pub const BTN_HEIGHT: i32 = 30;
pub const BTN_TEXT_PADDING: i32 = 10;
pub const IMG_PADDING: i32 = 10;

#[derive(Clone)]
pub enum StepAction {
    LinearCustom(LinearCustom),
    LinearMean(LinearMean),
    LinearGaussian(LinearGaussian),
    MedianFilter(MedianFilter),
    HistogramLocalContrast(HistogramLocalContrast),
    CutBrightness(CutBrightness)
}

impl StepAction {
    fn filter_description(&self) -> String {
        match self {
            StepAction::LinearCustom(filter) => filter.get_description(),
            StepAction::LinearMean(filter) => filter.get_description(),
            StepAction::LinearGaussian(filter) => filter.get_description(),
            StepAction::MedianFilter(filter) => filter.get_description(),
            StepAction::HistogramLocalContrast(filter) => filter.get_description(),
            StepAction::CutBrightness(filter) => filter.get_description(),
        }
    }

    fn edit_with_dlg(&self, app: App, step_editor: &mut StepEditor) -> StepAction {
        if let Some(edited_action) = step_editor.add_with_dlg(app, self.clone()) {
            edited_action
        } else {
            self.clone()
        }
    }

    fn act<Cbk: Fn(usize)>(&mut self, init_img: &Matrix2D, progress_cbk: Cbk) -> Matrix2D{
        match self {
            StepAction::LinearCustom(ref mut filter) => 
                init_img.processed_copy(filter, progress_cbk),
            StepAction::LinearMean(ref mut filter) => 
                init_img.processed_copy(filter, progress_cbk),
            StepAction::LinearGaussian(ref mut filter) => 
                init_img.processed_copy(filter, progress_cbk),
            StepAction::MedianFilter(ref mut filter) => 
                init_img.processed_copy(filter, progress_cbk),
            StepAction::HistogramLocalContrast(ref mut filter) => 
                init_img.processed_copy(filter, progress_cbk),
            StepAction::CutBrightness(ref mut filter) => 
                init_img.processed_copy(filter, progress_cbk),
        }
    }
}


pub struct ProcessingLine<'wind> {
    parent_window: &'wind mut window::Window,
    initial_img: Option<img::Matrix2D>,
    steps: Vec<ProcessingStep>,
    x: i32, y: i32, w: i32, h: i32,
    receiver: Receiver<Message>,
    step_editor: StepEditor,
    processing_data: Arc<Mutex<Option<ProcessingData>>>,
    processing_thread: JoinHandle<()>,
    are_steps_chained: bool,
    // graphical parts
    wind_size_prev: (i32, i32),
    frame_img: frame::Frame,
    main_horz_pack: group::Pack,
    init_img_pack: group::Pack,
    processing_pack: group::Pack,
    scroll_area: group::Scroll,
    scroll_pack: group::Pack,    
}

impl<'wind> ProcessingLine<'wind> {
    pub fn new(wind_parent: &'wind mut window::Window, x: i32, y: i32, w: i32, h: i32) -> Self {
        wind_parent.begin();

        let (sender, receiver) = app::channel::<Message>();

        let mut main_horz_pack = group::Pack::default()
            .with_pos(x, y)
            .with_size(w, h);
        main_horz_pack.set_type(PackType::Horizontal);
            
        let mut init_img_pack = group::Pack::default()
            .with_pos(x, y)
            .with_size(w / 2, h);
        init_img_pack.set_type(PackType::Vertical);

        let mut menu = menu::SysMenuBar::default().with_size(w / 2, BTN_HEIGHT);
        menu.add_emit("Проект/Зарузить", Shortcut::None, menu::MenuFlag::Normal, sender, 
            Message::Project(Project::LoadProject));
        menu.add_emit("Проект/Сохранить как", Shortcut::None, menu::MenuFlag::Normal, sender, 
            Message::Project(Project::SaveProject));
        menu.add_emit("Экспорт/Сохранить результаты", Shortcut::None, menu::MenuFlag::Normal, sender, 
            Message::Project(Project::SaveResults));
        menu.end();

        frame::Frame::default()
            .with_size(w / 2, BTN_HEIGHT)
            .with_label("Исходное изображение");

        let mut btn_load_initial_img = button::Button::default()
            .with_size(BTN_WIDTH, BTN_HEIGHT)
            .with_label("Загрузить");
        btn_load_initial_img.emit(sender, Message::Project(Project::LoadImage));        
        {
            let (bw, bh) = btn_load_initial_img.measure_label();
            btn_load_initial_img.set_size(bw + BTN_TEXT_PADDING, bh + BTN_TEXT_PADDING);
        }
            
        let mut frame_img = frame::Frame::default()
            .with_size(w / 2, h - BTN_HEIGHT * 2);
        frame_img.set_frame(FrameType::EmbossedFrame);
        frame_img.set_align(Align::Center);   
        
        init_img_pack.end();

        let mut processing_pack = group::Pack::default()
            .with_size(w / 2, h);
        processing_pack.set_type(PackType::Vertical);

        let mut btn_add_step = menu::MenuButton::default().with_size(w / 2, BTN_HEIGHT);
        btn_add_step.set_label("Добавить");
        btn_add_step.add_emit("Линейный фильтр (усредняющий)", Shortcut::None, menu::MenuFlag::Normal, sender, 
            Message::Step(Step::AddStepLinMean));
        btn_add_step.add_emit("Линейный фильтр (гауссовский)", Shortcut::None, menu::MenuFlag::Normal, sender, 
            Message::Step(Step::AddStepLinGauss));
        btn_add_step.add_emit("Линейный фильтр (другой)", Shortcut::None, menu::MenuFlag::Normal, sender, 
            Message::Step(Step::AddStepLinCustom));
        btn_add_step.add_emit("Медианный фильтр", Shortcut::None, menu::MenuFlag::Normal, sender, 
            Message::Step(Step::AddStepMed));
        btn_add_step.add_emit("Локальный контраст (гистограмма)", Shortcut::None, menu::MenuFlag::Normal, sender, 
            Message::Step(Step::AddStepHistogramLocalContrast));
        btn_add_step.add_emit("Обрезание яркости", Shortcut::None, menu::MenuFlag::Normal, sender, 
            Message::Step(Step::AddStepCutBrightness));
        btn_add_step.end();

        let scroll_area = group::Scroll::default()
            .with_pos(x, y + btn_add_step.h())
            .with_size(w / 2, h - btn_add_step.h());

        let scroll_pack = group::Pack::default()
            .with_pos(x, y + btn_add_step.h())
            .with_size(w / 2, h - btn_add_step.h());

        scroll_pack.end();
        scroll_area.end();
        processing_pack.end();

        main_horz_pack.end();

        wind_parent.end();

        let sender_copy = sender.clone();
        let processing_data = Arc::new(Mutex::new(Option::<ProcessingData>::None));
        let processing_data_copy = processing_data.clone();
        let processing_thread = thread::spawn(move || {
            loop {
                thread::park();
                match processing_data_copy.try_lock() {
                    Ok(mut guard) => match guard.as_mut() {
                        Some(pd) => {
                            let step_num = pd.step_num;
                            let progress_cbk = |cur_percents: usize| {
                                sender_copy.send(Message::Processing(Processing::StepProgress { step_num, cur_percents }));
                            };

                            pd.result_img = Some(pd.step_action.act(&pd.init_img, progress_cbk));

                            sender_copy.send(Message::Processing(Processing::StepIsComplete { step_num: pd.step_num }));
                        },
                        None => { }
                    },
                    Err(_) => { }
                };
            }
        });

        let wind_size_prev = (wind_parent.w(), wind_parent.h());

        ProcessingLine {
            parent_window: wind_parent,
            initial_img: None,
            steps: Vec::<ProcessingStep>::new(),
            x, y, w, h,
            receiver,
            step_editor: StepEditor::new(),
            processing_data,
            processing_thread,
            are_steps_chained: true,
            
            // graphical parts
            wind_size_prev,
            frame_img,
            main_horz_pack,
            init_img_pack,
            processing_pack,
            scroll_area,
            scroll_pack,
        }
    }

    pub fn add(&mut self, step_action: StepAction) -> () {
        self.scroll_pack.begin();

        self.steps.push(ProcessingStep::new(&self, step_action));

        self.scroll_pack.end();
    }

    pub fn run(&mut self, app: app::App) -> result::Result<(), MyError> {
        while app.wait() {
            if let Some(msg) = self.receiver.recv() {
                match msg {
                    Message::Project(msg) => {
                        match msg {
                            Project::LoadImage => {
                                match self.try_load() {
                                    Ok(_) => {}
                                    Err(err) => err_msg(&self.parent_window, &err.to_string())
                                };
                                self.parent_window.redraw();
                            },
                            Project::SaveProject => {
                                match self.try_save_project() {
                                    Ok(_) => info_msg(&self.parent_window, "Проект успешно сохранен"),
                                    Err(err) => err_msg(&self.parent_window, &err.get_message()),
                                }
                            },
                            Project::LoadProject => {
                                match self.try_load_project() {
                                    Ok(_) => info_msg(&self.parent_window, "Проект успешно загружен"),
                                    Err(err) => err_msg(&self.parent_window, &err.get_message()),
                                }
                            },
                            Project::SaveResults => {
                                match self.try_save_results() {
                                    Ok(_) => info_msg(&self.parent_window, "Результаты успешно сохранены"),
                                    Err(err) => err_msg(&self.parent_window, &err.get_message()),
                                }
                            }
                        };
                        self.parent_window.redraw();
                    },
                    Message::Step(msg) => {
                        match msg {
                            Step::AddStepLinCustom => {
                                if let Some(new_action) = self.step_editor.add_with_dlg(app, LinearCustom::default().into()) {
                                    self.add(new_action);
                                }
                            },
                            Step::AddStepLinMean => {
                                if let Some(new_action) = self.step_editor.add_with_dlg(app, LinearMean::default().into()) {
                                    self.add(new_action);
                                }
                            },
                            Step::AddStepLinGauss => {
                                if let Some(new_action) = self.step_editor.add_with_dlg(app, LinearGaussian::default().into()) {
                                    self.add(new_action);
                                }
                            },
                            Step::AddStepMed => {
                                if let Some(new_action) = self.step_editor.add_with_dlg(app, MedianFilter::default().into()) {
                                    self.add(new_action);
                                }
                            },
                            Step::AddStepHistogramLocalContrast => {
                                if let Some(new_action) = self.step_editor.add_with_dlg(app, HistogramLocalContrast::default().into()) {
                                    self.add(new_action);
                                }
                            },
                            Step::AddStepCutBrightness => {
                                if let Some(new_action) = self.step_editor.add_with_dlg(app, CutBrightness::default().into()) {
                                    self.add(new_action);
                                }
                            },
                            Step::EditStep { step_num } => {
                                self.steps[step_num].edit_action_with_dlg(app, &mut self.step_editor);
                            },
                            Step::DeleteStep { step_num } => self.delete_step(step_num),
                            Step::MoveStep { step_num, direction } => {
                                match direction {
                                    message::MoveStep::Up => {
                                        if step_num > 0 {                                            
                                            self.scroll_pack.begin();

                                            for step in self.steps[step_num - 1..].iter_mut() {
                                                step.remove_self_from(&mut self.scroll_pack);
                                            }

                                            self.steps.swap(step_num - 1, step_num);

                                            for step in self.steps[step_num - 1..].iter_mut() {
                                                step.draw_self_on(&mut self.scroll_pack);
                                            }

                                            self.scroll_pack.end();
                                        }
                                    },
                                    message::MoveStep::Down => {                                        
                                        if step_num < self.steps.len() - 1 {                                            
                                            self.scroll_pack.begin();

                                            for step in self.steps[step_num..].iter_mut() {
                                                step.remove_self_from(&mut self.scroll_pack);
                                            }

                                            self.steps.swap(step_num, step_num + 1);

                                            for step in self.steps[step_num..].iter_mut() {
                                                step.draw_self_on(&mut self.scroll_pack);
                                            }

                                            self.scroll_pack.end();
                                        }
                                    },
                                };
                                for i in 0..self.steps.len() {
                                    self.steps[i].set_step_num(i);
                                }
                            },
                        };
                        self.parent_window.redraw();
                    }
                    Message::Processing(msg) => {
                        match msg {
                            Processing::StepIsStarted { step_num, do_chaining } => {
                                self.are_steps_chained = do_chaining;
                                match self.try_start_step(step_num) {
                                    Ok(_) => {}
                                    Err(err) => err_msg(&self.parent_window, &err.to_string())
                                };
                            },
                            Processing::StepProgress { step_num, cur_percents } => {
                                self.steps[step_num].display_progress(cur_percents);
                            },
                            Processing::StepIsComplete { step_num } => {
                                match self.steps[step_num].display_result(self.processing_data.clone()) {
                                    Ok(_) => { 
                                        if self.are_steps_chained && step_num < self.steps.len() - 1 {
                                            match self.try_start_step(step_num + 1) {
                                                Ok(_) => {}
                                                Err(err) => err_msg(&self.parent_window, &err.to_string())
                                            };
                                        }
                                    }
                                    Err(err) => err_msg(&self.parent_window, &err.to_string())
                                };
                            },
                        };
                        self.parent_window.redraw();
                    }
                };
            }            
                  
            self.auto_resize()?;
        }
    
        Ok(())
    }

    fn auto_resize(&mut self) -> Result<(), MyError> {
        let ww = self.parent_window.w();
        let wh = self.parent_window.h(); 

        if self.wind_size_prev.0 == ww && self.wind_size_prev.1 == wh { return Ok(()); }
        
        self.main_horz_pack.set_size(ww, wh);
        self.init_img_pack.set_size(ww / 2, wh);

        self.processing_pack.set_size(ww / 2, wh);
        self.processing_pack.set_pos(self.x + ww / 2, self.y);
        self.scroll_area.set_size(ww / 2, wh);
        self.scroll_area.set_pos(self.x + ww / 2, self.y);
        self.scroll_pack.set_size(ww / 2, wh);
        self.scroll_pack.set_pos(self.x + ww / 2, self.y);

        if let Some(img) = &self.initial_img {
            let mut rgb_copy = img.get_drawable_copy()?;
            rgb_copy.scale(ww / 2 - IMG_PADDING, self.frame_img.h() - IMG_PADDING, true, true);
            self.frame_img.set_image(Some(rgb_copy));
        }

        for step in self.steps.iter_mut() {
            step.auto_resize(ww / 2)?;
        }

        self.wind_size_prev = (ww, wh);

        self.parent_window.redraw();      

        Ok(())
    }

    fn delete_step(&mut self, step_num: usize) {
        self.scroll_pack.begin();
        self.steps[step_num].remove_self_from(&mut self.scroll_pack);
        self.scroll_pack.end();

        self.steps.remove(step_num);

        for i in step_num..self.steps.len() {
            self.steps[i].set_step_num(i);
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
        img_copy.scale(self.frame_img.w() - IMG_PADDING, self.frame_img.h() - IMG_PADDING, true, true);
        self.frame_img.set_image(Some(img_copy.clone()));
        self.frame_img.redraw(); 

        self.initial_img = Some(init_image);

        Ok(())
    }

    fn try_start_step(&mut self, step_num: usize) -> result::Result<(), MyError> {
        assert!(self.steps.len() > step_num);

        if step_num == 0 {
            match self.initial_img {
                Some(ref img) => {
                    let img_copy = img.clone();
                    self.steps[step_num].start_processing(self.processing_data.clone(), img_copy)?;
                },
                None => return Err(MyError::new("Необходимо загрузить изображение для обработки".to_string()))
            }
        } else {
            let prev_step = &self.steps[step_num - 1];
            match prev_step.get_data_copy() {
                Some(img_copy) => {
                    self.steps[step_num].start_processing(self.processing_data.clone(), img_copy)?;
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
            let filter_content: String = match &self.steps[step_num].action {
                StepAction::LinearCustom(ref filter) => filter.content_to_string(),
                StepAction::LinearMean(ref filter) => filter.content_to_string(),
                StepAction::LinearGaussian(ref filter) => filter.content_to_string(),
                StepAction::MedianFilter(ref filter) => filter.content_to_string(),
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

            if let Some(step_action) = {
                if let Ok(filter) = LinearCustom::try_from_string(&content) { Some(filter.into()) } 
                else if let Ok(filter) = LinearMean::try_from_string(&content) { Some(filter.into()) } 
                else if let Ok(filter) = LinearGaussian::try_from_string(&content) { Some(filter.into()) } 
                else if let Ok(filter) = LinearMean::try_from_string(&content) { Some(filter.into()) } 
                else if let Ok(filter) = HistogramLocalContrast::try_from_string(&content) { Some(filter.into()) } 
                else { None }
            } {
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
    hpack: group::Pack,
    btn_process: menu::MenuButton,
    btn_edit: button::Button,
    btn_delete: button::Button,
    btn_move_step: menu::MenuButton,
    label_step_name: Frame,
    frame_img: Frame,
    pub action: StepAction,
    image: Option<img::Matrix2D>,
    step_num: usize,
    sender: Sender<Message>
}

impl ProcessingStep {
    fn new(proc_line: &ProcessingLine, action: StepAction) -> Self {
        let name: String = action.filter_description();

        let label = frame::Frame::default()
            .with_size(proc_line.w, BTN_HEIGHT)
            .with_label(&name);  

        let (sender, _) = app::channel::<Message>();

        let mut hpack = group::Pack::default()
            .with_size(proc_line.w, BTN_HEIGHT); 
        hpack.set_type(PackType::Horizontal);
        hpack.set_spacing(PADDING);

        let step_num = proc_line.steps.len();

        let mut btn_process = menu::MenuButton::default();
        btn_process.set_label("Запустить");
        let (w, h) = btn_process.measure_label();
        btn_process.set_size(w + BTN_TEXT_PADDING + 30, h + BTN_TEXT_PADDING);
        btn_process.add_emit("Только этот шаг", Shortcut::None, MenuFlag::Normal, sender, 
            Message::Processing(Processing::StepIsStarted { step_num, do_chaining: false }));
        btn_process.add_emit("Этот шаг и все шаги ниже", Shortcut::None, MenuFlag::Normal, sender, 
            Message::Processing(Processing::StepIsStarted { step_num, do_chaining: true }));

        let mut btn_edit = button::Button::default();
        btn_edit.set_label("Изменить");
        btn_edit.emit(sender, Message::Step(Step::EditStep { step_num }));
        let (w, h) = btn_edit.measure_label();
        btn_edit.set_size(w + BTN_TEXT_PADDING, h + BTN_TEXT_PADDING);

        let mut btn_delete = button::Button::default();
        btn_delete.set_label("Удалить");
        btn_delete.emit(sender, Message::Step(Step::DeleteStep { step_num }));
        let (w, h) = btn_delete.measure_label();
        btn_delete.set_size(w + BTN_TEXT_PADDING, h + BTN_TEXT_PADDING);

        let mut btn_move_step = menu::MenuButton::default();
        btn_move_step.set_label("Переупорядочить");
        let (w, h) = btn_move_step.measure_label();
        btn_move_step.set_size(w + BTN_TEXT_PADDING + 30, h + BTN_TEXT_PADDING);
        btn_move_step.add_emit("Сдвинуть вверх", Shortcut::None, MenuFlag::Normal, sender, 
            Message::Step(Step::MoveStep { step_num, direction: message::MoveStep::Up } ));
        btn_move_step.add_emit("Сдвинуть вниз", Shortcut::None, MenuFlag::Normal, sender, 
            Message::Step(Step::MoveStep { step_num, direction: message::MoveStep::Down } ));

        hpack.end();
            
        let mut frame_img = frame::Frame::default()
            .with_size(proc_line.w, proc_line.h - BTN_HEIGHT * 2);
        frame_img.set_frame(FrameType::EmbossedFrame);
        frame_img.set_align(Align::ImageMask | Align::Center);    
        
        ProcessingStep { 
            hpack,
            btn_process, btn_edit, btn_delete, btn_move_step,
            frame_img, 
            label_step_name: label,
            action,
            image: None, 
            step_num,
            sender
        }
    }

    fn auto_resize(&mut self, new_width: i32) -> Result<(), MyError> {
        self.frame_img.set_size(new_width, self.frame_img.h());

        if let Some(img) = &self.image {
            let mut rgb_copy = img.get_drawable_copy()?;
            rgb_copy.scale(self.frame_img.w() - IMG_PADDING, self.frame_img.h() - IMG_PADDING, true, true);
            self.frame_img.set_image(Some(rgb_copy));
        }

        Ok(())
    }

    fn draw_self_on(&mut self, pack: &mut group::Pack) {
        pack.add(&mut self.label_step_name);
        pack.add(&mut self.hpack);
        self.hpack.begin();
        self.hpack.add(&mut self.btn_process);
        self.hpack.add(&mut self.btn_edit);
        self.hpack.add(&mut self.btn_delete);
        self.hpack.add(&mut self.btn_move_step);
        self.hpack.end();
        pack.add(&mut self.frame_img);
    }

    fn remove_self_from(&mut self, pack: &mut group::Pack) {
        pack.remove(&mut self.label_step_name);
        self.hpack.begin();
        self.hpack.remove(&mut self.btn_process);
        self.hpack.remove(&mut self.btn_edit);
        self.hpack.remove(&mut self.btn_delete);
        self.hpack.remove(&mut self.btn_move_step);
        self.hpack.end();
        pack.remove(&mut self.hpack);
        pack.remove(&mut self.frame_img);
    }
    
    fn edit_action_with_dlg(&mut self, app: app::App, step_editor: &mut StepEditor) {
        self.action = self.action.edit_with_dlg(app, step_editor);
        
        let filter_description: String = self.action.filter_description();

        let img_description: String = match self.image {
            Some(ref img) => img.get_description(),
            None => String::new(),
        };

        self.label_step_name.set_label(&format!("{} {}", &filter_description, &img_description));
    }

    fn set_step_num(&mut self, step_num: usize) {
        self.btn_process.add_emit("Только этот шаг", Shortcut::None, MenuFlag::Normal, self.sender, 
            Message::Processing(Processing::StepIsStarted { step_num, do_chaining: false }));
        self.btn_process.add_emit("Этот шаг и все шаги ниже", Shortcut::None, MenuFlag::Normal, self.sender, 
            Message::Processing(Processing::StepIsStarted { step_num, do_chaining: true }));
        self.btn_edit.emit(self.sender, Message::Step(Step::EditStep { step_num } ));
        self.btn_delete.emit(self.sender, Message::Step(Step::DeleteStep { step_num }));
        self.btn_move_step.add_emit("Сдвинуть вверх", Shortcut::None, MenuFlag::Normal, self.sender, 
            Message::Step(Step::MoveStep { step_num, direction: message::MoveStep::Up } ));
        self.btn_move_step.add_emit("Сдвинуть вниз", Shortcut::None, MenuFlag::Normal, self.sender, 
            Message::Step(Step::MoveStep { step_num, direction: message::MoveStep::Down } ));
        self.label_step_name.redraw_label();
        self.frame_img.set_damage(true);
        self.step_num = step_num;
    }

    fn set_buttons_active(&mut self, active: bool) {
        if active {
            self.btn_process.activate();
            self.btn_edit.activate();
            self.btn_delete.activate();
            self.btn_move_step.activate();
        } else {
            self.btn_process.deactivate();
            self.btn_edit.deactivate();
            self.btn_delete.deactivate();
            self.btn_move_step.deactivate();
        }
    }

    fn get_data_copy(&self) -> Option<img::Matrix2D> {
        self.image.clone()
    }
 
    fn start_processing(&mut self, processing_data: Arc<Mutex<Option<ProcessingData>>>, init_img: Matrix2D) -> Result<(), MyError> {
        processing_data.lock().unwrap().replace(ProcessingData::new(self.step_num, self.action.clone(), init_img));
        drop(processing_data);

        self.set_buttons_active(false);

        self.frame_img.set_image(Option::<RgbImage>::None); 

        Ok(())
    }

    fn display_progress(&mut self, progress_percents: usize) {
        self.frame_img.set_label(&format!("{}%", progress_percents));
        self.frame_img.redraw_label();
    }

    fn display_result(&mut self, processing_data: Arc<Mutex<Option<ProcessingData>>>) -> Result<(), MyError>  {
        self.frame_img.set_label("");

        let pd_locked = processing_data.lock().unwrap().take();
        drop(processing_data);
        let result_img = match pd_locked {
            Some(mut p) => match p.get_result() {
                Some(img) => img,
                None => { return Err(MyError::new("Нет результирующего изображения(".to_string())); },
            },
            None => { return Err(MyError::new("Нет данных для обработки(".to_string())); },
        };

        self.set_buttons_active(true);
                        
        let mut rgb_image: fltk::image::RgbImage = result_img.get_drawable_copy()?;
        rgb_image.scale(self.frame_img.w() - IMG_PADDING, self.frame_img.h() - IMG_PADDING, true, true);
                  
        self.frame_img.set_image(Some(rgb_image));
        self.frame_img.redraw();

        self.image = Some(result_img);

        Ok(())
    }
}

