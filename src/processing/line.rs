use std::{fs::{self, File}, io::{Read, Write}, sync::{Arc, Mutex}, thread::{self, JoinHandle}, usize};
use chrono::{Local, format::{DelayedFormat, StrftimeItems}};
use fltk::{app::{self, Receiver}, dialog, group, prelude::{GroupExt, WidgetExt}, window};
use crate::{filter::{channel::{EqualizeHist, ExtractChannel, NeutralizeChannel, Rgb2Gray}, linear::{LinearCustom, LinearGaussian, LinearMean}, non_linear::{CutBrightness, HistogramLocalContrast, MedianFilter}}, img::Img, message::{self, AddStep, Message, Processing, Project, StepOp}, my_component::{Alignable, container::{MyColumn, MyRow}, img_presenter::MyImgPresenter, usual::{MyLabel, MyMenuBar, MyProgressBar}}, my_err::MyError, small_dlg::{self, confirm_with_dlg, show_err_msg, show_info_msg}, utils};
use super::{FilterBase, PADDING, ProcessingData, progress_provider::ProgressProvider, step::ProcessingStep, step_editor::StepEditor};


pub struct ProcessingLine<'wind> {
    parent_window: &'wind mut window::Window,
    steps: Vec<ProcessingStep>,
    x: i32, y: i32, w: i32, h: i32,
    receiver: Receiver<Message>,
    step_editor: StepEditor,
    processing_data: Arc<Mutex<Option<ProcessingData>>>,
    processing_thread_handle: JoinHandle<()>,
    // graphical parts
    wind_size_prev: (i32, i32),
    img_presenter: MyImgPresenter,
    main_row: MyRow,
    init_img_col: MyColumn,
    main_menu: MyMenuBar,
    lbl_init_img: MyLabel,
    whole_proc_prog_bar: MyProgressBar,
    processing_col: MyColumn,
    scroll_area: group::Scroll,
    scroll_pack: group::Pack,    
}

impl<'wind> ProcessingLine<'wind> {
    pub fn new(wind_parent: &'wind mut window::Window, x: i32, y: i32, w: i32, h: i32) -> Self {
        wind_parent.begin();

        let (sender, receiver) = app::channel::<Message>();

        let mut main_row = MyRow::new(w, h).with_pos(x, y);
            
        let mut init_img_col = MyColumn::new(w / 2, h);

        let mut main_menu = MyMenuBar::new(wind_parent);
        main_menu.add_emit("Проект/Зарузить", sender, Message::Project(Project::LoadProject));
        main_menu.add_emit("Проект/Сохранить как", sender, Message::Project(Project::SaveProject));
        main_menu.add_emit("Импорт/Загрузить", sender, Message::Project(Project::LoadImage));

        main_menu.add_emit("Добавить шаг/Цветной => ч\\/б", sender, Message::AddStep(AddStep::AddStepRgb2Gray));

        main_menu.add_emit("Добавить шаг/Линейный фильтр (усредняющий)", sender, Message::AddStep(AddStep::AddStepLinMean));
        main_menu.add_emit("Добавить шаг/Линейный фильтр (гауссовский)", sender, Message::AddStep(AddStep::AddStepLinGauss));
        main_menu.add_emit("Добавить шаг/Линейный фильтр (другой)", sender, Message::AddStep(AddStep::AddStepLinCustom));
        main_menu.add_emit("Добавить шаг/Медианный фильтр", sender, Message::AddStep(AddStep::AddStepMed));
        main_menu.add_emit("Добавить шаг/Локальный контраст (гистограмма)", sender, Message::AddStep(AddStep::AddStepHistogramLocalContrast));
        main_menu.add_emit("Добавить шаг/Обрезание яркости", sender, Message::AddStep(AddStep::AddStepCutBrightness));
        main_menu.add_emit("Добавить шаг/Эквализация гистограммы", sender, Message::AddStep(AddStep::AddStepHistogramEqualizer));

        main_menu.add_emit("Добавить шаг/Убрать канал", sender, Message::AddStep(AddStep::AddStepNeutralizeChannel));

        main_menu.add_emit("Добавить шаг/Выделить канал", sender, Message::AddStep(AddStep::AddStepExtractChannel));

        main_menu.add_emit("Экспорт/Сохранить результаты", sender, Message::Project(Project::SaveResults));
        main_menu.end();

        let lbl_init_img = MyLabel::new("Исходное изображение");

        let mut whole_proc_prog_bar = MyProgressBar::new(w / 2, 30);
        whole_proc_prog_bar.hide();
            
        let img_presenter = MyImgPresenter::new(
            w / 2, h - lbl_init_img.h() - main_menu.h() - whole_proc_prog_bar.h());
        
        init_img_col.end();

        let mut processing_col = MyColumn::new(w / 2, h - main_menu.h());

        let scroll_area = group::Scroll::default()
            .with_pos(x, y + main_menu.h())
            .with_size(w / 2, h - main_menu.h());

        let scroll_pack = group::Pack::default()
            .with_pos(x, y + main_menu.h())
            .with_size(w / 2 - PADDING, h - main_menu.h());

        scroll_pack.end();
        scroll_area.end();
        processing_col.end();

        main_row.end();

        wind_parent.end();

        let sender_copy = sender.clone();
        let processing_data: Arc<Mutex<Option<ProcessingData>>> = Arc::new(Mutex::new(Option::<ProcessingData>::None));
        let processing_data_copy = processing_data.clone();
        let processing_thread_handle: JoinHandle<()> = thread::spawn(move || 
        {
            loop {
                thread::park();
                match processing_data_copy.try_lock() {
                    Ok(mut guard) => match guard.as_mut() {
                        Some(pd) => {
                            let mut prog_prov = ProgressProvider::new(sender_copy.clone());
                            prog_prov.set_step_num(pd.step_num);

                            pd.result_img = Some(pd.filter_copy.filter(&pd.init_img, &mut prog_prov));

                            sender_copy.send(Message::Processing(Processing::StepIsCompleted { step_num: pd.step_num }));
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
            steps: Vec::<ProcessingStep>::new(),
            x, y, w, h,
            receiver,
            step_editor: StepEditor::new(),
            processing_data,
            processing_thread_handle,
            // graphical parts
            wind_size_prev,
            img_presenter,
            main_row,
            init_img_col,
            main_menu,
            lbl_init_img,
            whole_proc_prog_bar,
            processing_col,
            scroll_area,
            scroll_pack,
        }
    }

    pub fn run(&mut self, app: app::App) -> Result<(), MyError> {
        while app.wait() {
            if let Some(msg) = self.receiver.recv() {
                match msg {
                    Message::Project(msg) => {
                        match msg {
                            Project::LoadImage => {
                                match self.try_load_initial_img() {
                                    Ok(_) => {}
                                    Err(err) => show_err_msg(&self.parent_window, &err.to_string())
                                };
                                self.parent_window.redraw();
                            },
                            Project::SaveProject => {
                                match self.try_save_project() {
                                    Ok(done) => if done {
                                        show_info_msg(&self.parent_window, "Проект успешно сохранен");
                                    },
                                    Err(err) => show_err_msg(&self.parent_window, &err.get_message()),
                                }
                            },
                            Project::LoadProject => {
                                match self.try_load_project() {
                                    Ok(done) => if done { 
                                        show_info_msg(&self.parent_window, "Проект успешно загружен"); 
                                    },
                                    Err(err) => show_err_msg(&self.parent_window, &err.get_message()),
                                }
                            },
                            Project::SaveResults => {
                                match self.try_save_results() {
                                    Ok(_) => show_info_msg(&self.parent_window, "Результаты успешно сохранены"),
                                    Err(err) => show_err_msg(&self.parent_window, &err.get_message()),
                                }
                            }
                        };
                        self.parent_window.redraw();
                    },
                    Message::AddStep(msg) => {
                        let mut filter = match msg {
                            AddStep::AddStepLinCustom => Box::new(LinearCustom::default()) as FilterBase,
                            AddStep::AddStepLinMean => Box::new(LinearMean::default()) as FilterBase,
                            AddStep::AddStepLinGauss => Box::new(LinearGaussian::default()) as FilterBase,
                            AddStep::AddStepMed => Box::new(MedianFilter::default()) as FilterBase,
                            AddStep::AddStepHistogramLocalContrast => Box::new(HistogramLocalContrast::default()) as FilterBase,
                            AddStep::AddStepCutBrightness => Box::new(CutBrightness::default()) as FilterBase,
                            AddStep::AddStepHistogramEqualizer => Box::new(EqualizeHist::default()) as FilterBase,
                            AddStep::AddStepRgb2Gray => Box::new(Rgb2Gray::default()) as FilterBase,
                            AddStep::AddStepNeutralizeChannel => Box::new(NeutralizeChannel::default()) as FilterBase,
                            AddStep::AddStepExtractChannel => Box::new(ExtractChannel::default()) as FilterBase,
                        };
                        if self.step_editor.edit_with_dlg(app, &mut filter) {
                            self.add_step(filter);
                        }
                        self.parent_window.redraw();
                    },
                    Message::StepOp(msg) => {
                        match msg {
                            StepOp::EditStep { step_num } => {
                                self.steps[step_num].edit_filter_with_dlg(app, &mut self.step_editor);
                            },
                            StepOp::DeleteStep { step_num } => self.delete_step(step_num),
                            StepOp::MoveStep { step_num, direction } => {
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
                                for step_num in 0..self.steps.len() {
                                    self.steps[step_num].update_btn_emits(step_num);
                                }
                            },
                        }
                        self.parent_window.redraw();
                    },
                    Message::Processing(msg) => {
                        let set_all_controls_active = |owner: &mut Self, active: bool| {
                            for step in owner.steps.iter_mut() {
                                step.set_buttons_active(active);
                            }
                            owner.main_menu.set_active(active);
                        };

                        match msg {
                            Processing::StepsChainIsStarted { step_num, do_until_end } => {
                                match self.try_start_step(step_num, do_until_end) {
                                    Ok(_) => {
                                        set_all_controls_active(self, false);
        
                                        self.whole_proc_prog_bar.show();
                                        let whole_prog_min = step_num * 100 / self.steps.len();
                                        self.whole_proc_prog_bar.set_value(whole_prog_min);
        
                                        for step in &mut self.steps[step_num..] {
                                            step.clear_result();
                                        }
                                    }
                                    Err(err) => show_err_msg(&self.parent_window, &err.to_string())
                                };
                            },
                            Processing::StepProgress { step_num, cur_percents } => {
                                let whole_prog = (step_num * 100 + cur_percents) / self.steps.len();
                                self.whole_proc_prog_bar.set_value(whole_prog);

                                self.steps[step_num].display_progress(cur_percents);
                            },
                            Processing::StepIsCompleted { step_num } => {
                                let processing_continued: bool = match self.on_step_completed(step_num) {
                                    Ok(continued) => continued,
                                    Err(err) => {
                                        show_err_msg(&self.parent_window, &err.to_string());
                                        false
                                    },
                                };

                                if !processing_continued {
                                    set_all_controls_active(self, true);
                                    self.whole_proc_prog_bar.hide();
                                }
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


    fn add_step(&mut self, filter: FilterBase) -> () {
        self.scroll_pack.begin();

        self.steps.push(ProcessingStep::new(self.w, self.h, self.steps.len(), filter));

        self.scroll_pack.end();
    }

    fn delete_step(&mut self, step_num: usize) {
        self.scroll_pack.begin();
        self.steps[step_num].remove_self_from(&mut self.scroll_pack);
        self.scroll_pack.end();

        self.steps.remove(step_num);

        for sn in step_num..self.steps.len() {
            self.steps[sn].update_btn_emits(sn);
        }

        self.scroll_pack.top_window().unwrap().set_damage(true);
    }


    fn try_start_step(&mut self, step_num: usize, do_until_end: bool) -> Result<(), MyError> {
        assert!(self.steps.len() > step_num);

        if !self.img_presenter.has_image() {
            return Err(MyError::new("Необходимо загрузить изображение для обработки".to_string())); 
        }

        let img_copy = if step_num == 0 {
            self.img_presenter.image().unwrap().clone()
        } else {
            match self.steps[step_num - 1].get_data_copy() {
                Some(img_copy) => img_copy,
                None => { 
                    return Err(MyError::new("Необходим результат предыдущего шага для обработки текущего".to_string())); 
                }
            }
        };

        let filter_copy = self.steps[step_num].filter().get_copy();
        self.processing_data.lock().unwrap().replace(
            ProcessingData::new(step_num, filter_copy, img_copy, do_until_end));
        self.steps[step_num].start_processing();

        self.processing_thread_handle.thread().unpark();

        Ok(())
    }

    fn on_step_completed(&mut self, step_num: usize) -> Result<bool, MyError> {
        let mut pd_locked = self.processing_data.lock()
            .unwrap()
            .take()
            .expect("No processing_data detected");

        let result_img = pd_locked.take_result()
            .expect("No result image in processing_data");

        self.steps[step_num].display_result(result_img)?;

        let should_continue = pd_locked.do_until_end && step_num < self.steps.len() - 1;

        if should_continue {
            self.try_start_step(step_num + 1, should_continue)?;
        }
        
        Ok(should_continue)
    }


    fn auto_resize(&mut self) -> Result<(), MyError> {
        let ww = self.parent_window.w();
        let wh = self.parent_window.h(); 

        if self.wind_size_prev.0 == ww && self.wind_size_prev.1 == wh { return Ok(()); }
        
        self.main_row.widget_mut().set_size(ww, wh);
        self.init_img_col.widget_mut().set_size(ww / 2, wh);

        self.processing_col.widget_mut().set_size(ww / 2, wh);
        self.processing_col.widget_mut().set_pos(self.x + ww / 2, self.y);
        self.scroll_area.set_size(ww / 2, wh);
        self.scroll_area.set_pos(self.x + ww / 2, self.y);
        self.scroll_pack.set_size(ww / 2 - PADDING, wh);
        self.scroll_pack.set_pos(self.x + ww / 2, self.y);

        self.img_presenter.redraw();

        for step in self.steps.iter_mut() {
            step.auto_resize(ww / 2);
        }

        self.wind_size_prev = (ww, wh);

        self.parent_window.redraw();      

        Ok(())
    }

    
    fn try_load_initial_img(&mut self) -> Result<(), MyError> {
        if self.img_presenter.has_image() {
            if small_dlg::confirm_with_dlg(&self.parent_window, "Для открытия нового изображения нужно удалить предыдущие результаты. Продолжить?") {
                for step in self.steps.iter_mut() {
                    step.clear_result();
                }
            } else {
                return Ok(());
            }
        }

        let mut dlg = dialog::FileDialog::new(dialog::FileDialogType::BrowseFile);
        dlg.show();
        let path_buf = dlg.filename();

        if let Some(p) = path_buf.to_str() {
            if p.is_empty() { return Ok(()); }
        }     

        let init_image = Img::load_as_rgb(path_buf)?;

        self.lbl_init_img.set_text(&init_image.get_description());

        self.img_presenter.set_image(init_image)?;

        Ok(())
    }

    const PROJECT_EXT: &'static str = "ps";
    const FILTER_SAVE_SEPARATOR: &'static str = "||";

    fn cur_time_str() -> String {
        let current_datetime_formatter: DelayedFormat<StrftimeItems> = 
            Local::now().format("%d-%m(%b)-%Y_%a_%_H.%M.%S"); 
        format!("{}", current_datetime_formatter)
    }

    fn try_save_project(&self) -> Result<bool, MyError> {
        // check if there are any steps
        if self.steps.len() == 0 {
            return Err(MyError::new("В проекте нет шагов для сохранения".to_string()));
        }

        let mut dlg = dialog::FileDialog::new(dialog::FileDialogType::BrowseSaveFile);
        dlg.set_filter(&format!("*.{}", Self::PROJECT_EXT));

        let file_name = format!("Project {}", Self::cur_time_str());
        dlg.set_preset_file(&file_name);
        dlg.set_title("Сохранение проекта");

        dlg.show(); 

        let mut path_buf = dlg.filename();
        path_buf.set_extension(Self::PROJECT_EXT);

        let proj_path: &str = match path_buf.to_str() {
            Some(path) => path,
            None => { return Err(MyError::new("Не получилось перевести выбранный путь в строку".to_string())); },
        };

        if proj_path.is_empty() {
            return Ok(false);
        }

        let mut file = match File::create(proj_path) {
            Ok(f) => f,
            Err(err) => { return Err(MyError::new(err.to_string())); }
        };

        let mut file_content = String::new();

        for step_num in 0..self.steps.len() {
            let filter = self.steps[step_num].filter();
            let filter_save_name: String = filter.get_save_name();
            let filter_content: String = filter.content_to_string();
            
            file_content.push_str(&filter_save_name);
            file_content.push_str("\n");
            file_content.push_str(&filter_content);
            file_content.push_str("\n");

            if step_num < self.steps.len() - 1 {
                file_content.push_str(Self::FILTER_SAVE_SEPARATOR);
                file_content.push_str("\n");
            }
        }

        file.write_all(&file_content.as_bytes())?;
        file.sync_all()?;

        Ok(true)
    }
    
    fn try_load_project(&mut self) -> Result<bool, MyError> {
        if self.steps.len() > 0 {
            if confirm_with_dlg(self.parent_window,
                "Есть несохраненный проект. Открыть вместо него?") 
            {
                while self.steps.len() > 0 {
                    self.delete_step(0);
                }
            } else {
                return Ok(false);
            }
        } 
        
        let mut dlg = dialog::FileDialog::new(dialog::FileDialogType::BrowseFile);
        dlg.set_filter(&format!("*.{}", Self::PROJECT_EXT));
        dlg.set_title("Загрузка проекта");
        dlg.show(); 

        let path_buf = dlg.filename();

        let proj_path = match path_buf.to_str() {
            Some(path) => path,
            None => { return Err(MyError::new("Не получилось перевести выбранный путь в строку".to_string())); },
        };

        if proj_path.is_empty() {
            return Ok(false);
        }

        let mut file = match File::open(&proj_path) {
            Ok(f) => f,
            Err(err) => { return Err(MyError::new(format!("Ошибка при открытии файла проекта: {}", err.to_string()))); },
        };

        let mut file_content = String::new();
        if let Err(err) = file.read_to_string(&mut file_content) {
            return Err(MyError::new(format!("Ошибка при чтении файла проекта: {}", err.to_string())));
        };

        let mut filters_iter = utils::TextBlocksIter::new(
            &file_content, Self::FILTER_SAVE_SEPARATOR);

        self.steps.reserve(filters_iter.len());

        'out: for filter_str in filters_iter.iter() {
            let mut lines_iter = utils::LinesIter::new(filter_str);
            let filter_name = lines_iter.next_or_empty().to_string();
            let filter_content = lines_iter.all_left(true);

            match Self::try_parce_filter(&filter_name, &filter_content) {
                Ok(filter) => self.add_step(filter),
                Err(err) => {
                    let question = format!(
                        "Ошибка формата при чтении фильтра '{}': '{}'. Продолжить загрузку следующих шагов проекта?", 
                        filter_name, err.to_string());

                    if !confirm_with_dlg(self.parent_window, &question) {
                        break 'out;
                    }
                },
            }
        }

        Ok(true)
    }

    fn try_parce_filter(save_name: &str, content: &str) -> Result<FilterBase, MyError> {
        let mut filter = match save_name {
            "LinearCustom" => Box::new(LinearCustom::default()) as FilterBase,
            "LinearMean" =>  Box::new(LinearMean::default()) as FilterBase,
            "LinearGaussian" =>  Box::new(LinearGaussian::default()) as FilterBase,
            "MedianFilter" =>  Box::new(MedianFilter::default()) as FilterBase,
            "HistogramLocalContrast" =>  Box::new(HistogramLocalContrast::default()) as FilterBase,
            "CutBrightness" =>  Box::new(CutBrightness::default()) as FilterBase,
            "EqualizeHist" => Box::new(EqualizeHist::default()) as FilterBase,
            "Rgb2Gray" => Box::new(Rgb2Gray::default()) as FilterBase,
            "NeutralizeChannel" =>  Box::new(NeutralizeChannel::default()) as FilterBase,
            "ExtractChannel" =>  Box::new(ExtractChannel::default()) as FilterBase,
            _ => {
                return Err(MyError::new(format!("Не удалось загрузить фильтр '{}'", save_name)));
            }
        };
        filter.try_set_from_string(content)?;
        Ok(filter)
    }

    fn try_save_results(&self) -> Result<bool, MyError> {
        // check if there are any steps
        if self.steps.len() == 0 {
            return Err(MyError::new("В проекте нет результатов для сохранения".to_string()));
        }

        // check if all steps have images
        let all_steps_have_image = self.steps.iter().all(|s| s.has_image());
        if !all_steps_have_image {
            return Err(MyError::new("В проекте нет результатов для сохранения".to_string()));
        }

        let mut dlg = dialog::FileDialog::new(dialog::FileDialogType::BrowseSaveDir);
        dlg.set_title("Сохранение результатов");

        dlg.show(); 

        let path_buf = dlg.filename();

        let mut proj_path = match path_buf.to_str() {
            Some(path) => path.to_string(),
            None => { return Err(MyError::new("Не получилось перевести выбранный путь в строку".to_string())); },
        };

        if proj_path.is_empty() {
            return Ok(false);
        }

        proj_path.push_str("/");
        let dir_name = format!("Results {}", Self::cur_time_str());
        proj_path.push_str(&dir_name);
        
        match fs::create_dir(&proj_path) {
            Ok(_) => {},
            Err(err) => { return Err(MyError::new(err.to_string())); },
        };

        // save all images
        for step_num in 0..self.steps.len() {
            let mut file_path = proj_path.clone();
            file_path.push_str(&format!("/{}.bmp", step_num + 1));

            self.steps[step_num].image().unwrap().try_save(&file_path)?;
        }

        Ok(true)
    }
}