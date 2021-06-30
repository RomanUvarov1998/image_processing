use std::{fs::{self, File}, io::{Read, Write}, usize};
use chrono::{Local, format::{DelayedFormat, StrftimeItems}};
use fltk::{app::{self, Receiver}, dialog, group, prelude::{GroupExt, ImageExt, WidgetExt}};
use crate::{filter::{channel::{EqualizeHist, ExtractChannel, NeutralizeChannel, Rgb2Gray}, linear::{LinearCustom, LinearGaussian, LinearMean}, non_linear::{CutBrightness, HistogramLocalContrast, MedianFilter}}, img::Img, message::{self, AddStep, ImportType, Message, Processing, Project, StepOp}, my_component::{Alignable, container::{MyColumn, MyRow}, img_presenter::MyImgPresenter, usual::{MyButton, MyLabel, MyMenuButton, MyProgressBar}}, my_err::MyError, small_dlg::{self, confirm_with_dlg, show_err_msg, show_info_msg}, utils::{self, Pos, RectArea}};
use super::{FilterBase, PADDING, background_worker::{BackgroundWorker}, progress_provider::{HaltMessage}, step::ProcessingStep, step_editor::StepEditor};


pub struct ProcessingLine {
    steps: Vec<ProcessingStep>,
    x: i32, y: i32, w: i32, h: i32,
    receiver: Receiver<Message>,
    background_worker: BackgroundWorker,
    // graphical parts
    img_presenter: MyImgPresenter,
    main_row: MyRow,
    init_img_col: MyColumn,

    btns_row: MyRow,
    btn_project: MyMenuButton,
    btn_import: MyMenuButton,
    btn_add_step: MyMenuButton,
    btn_export: MyMenuButton,
    btn_halt_processing: MyButton,

    lbl_init_img: MyLabel,
    whole_proc_prog_bar: MyProgressBar,
    processing_col: MyColumn,
    scroll_area: group::Scroll,
    scroll_pack: group::Pack,    
}

impl ProcessingLine {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        let (sender, receiver) = app::channel::<Message>();

        let mut main_row = MyRow::new(w, h).with_pos(x, y);
            
        let mut init_img_col = MyColumn::new(w / 2, h);

        let mut btns_row = MyRow::new(w / 2, 100);

        let mut btn_project = MyMenuButton::with_label("Проект");
        btn_project.add_emit("Зарузить", sender, Message::Project(Project::LoadProject));
        btn_project.add_emit("Сохранить как", sender, Message::Project(Project::SaveProject));

        let mut btn_import = MyMenuButton::with_img_and_tooltip("import.png", "Импорт");
        btn_import.add_emit("Файл", sender, 
            Message::Project(Project::Import(ImportType::FromFile)));
        btn_import.add_emit("Системный буфер обмена", sender, 
            Message::Project(Project::Import(ImportType::FromSystemClipoard)));
            
        let mut btn_add_step = MyMenuButton::with_img_and_tooltip("add step.png", "Добавить шаг");
        btn_add_step.add_emit("Цветной => ч\\/б", sender, Message::AddStep(AddStep::AddStepRgb2Gray));
        btn_add_step.add_emit("Линейный фильтр (усредняющий)", sender, Message::AddStep(AddStep::AddStepLinMean));
        btn_add_step.add_emit("Линейный фильтр (гауссовский)", sender, Message::AddStep(AddStep::AddStepLinGauss));
        btn_add_step.add_emit("Линейный фильтр (другой)", sender, Message::AddStep(AddStep::AddStepLinCustom));
        btn_add_step.add_emit("Медианный фильтр", sender, Message::AddStep(AddStep::AddStepMed));
        btn_add_step.add_emit("Локальный контраст (гистограмма)", sender, Message::AddStep(AddStep::AddStepHistogramLocalContrast));
        btn_add_step.add_emit("Обрезание яркости", sender, Message::AddStep(AddStep::AddStepCutBrightness));
        btn_add_step.add_emit("Эквализация гистограммы", sender, Message::AddStep(AddStep::AddStepHistogramEqualizer));
        btn_add_step.add_emit("Убрать канал", sender, Message::AddStep(AddStep::AddStepNeutralizeChannel));
        btn_add_step.add_emit("Выделить канал", sender, Message::AddStep(AddStep::AddStepExtractChannel));

        let mut btn_export = MyMenuButton::with_img_and_tooltip("export.png", "Экспорт");
        btn_export.add_emit("Сохранить результаты", sender, Message::Project(Project::SaveResults));

        let (halt_msg_sender, halt_msg_receiver) = std::sync::mpsc::channel::<HaltMessage>();
        let mut btn_halt_processing = MyButton::with_img_and_tooltip("stop processing.png", "Прервать обработку");
        btn_halt_processing.widget_mut().set_callback(move |_| {
            halt_msg_sender.send(HaltMessage).unwrap_or(());
        });
        btn_halt_processing.set_active(false);
        
        let btns_heights = [btn_project.h(), btn_import.h(), btn_add_step.h(), btn_export.h(), btn_halt_processing.h()];

        btns_row.resize(btns_row.w(), *btns_heights.iter().max().unwrap());
        btns_row.end();

        let lbl_init_img = MyLabel::new("Исходное изображение");

        let mut whole_proc_prog_bar = MyProgressBar::new(w / 2, 30);
        whole_proc_prog_bar.hide();
            
        let img_presenter = MyImgPresenter::new(
            w / 2, h - lbl_init_img.h() - btns_row.h());
        
        init_img_col.end();

        let mut processing_col = MyColumn::new(w / 2, h - btns_row.h());

        let scroll_area = group::Scroll::default()
            .with_pos(x, y)
            .with_size(w / 2, h);

        let scroll_pack = group::Pack::default()
            .with_pos(x, y)
            .with_size(w / 2 - PADDING, h);

        scroll_pack.end();
        scroll_area.end();
        processing_col.end();

        main_row.end();

        let background_worker = BackgroundWorker::new(sender.clone(), halt_msg_receiver);

        let mut line = ProcessingLine {
            steps: Vec::<ProcessingStep>::new(),
            x, y, w, h,
            receiver,
            background_worker,
            // graphical parts
            img_presenter,
            main_row,
            init_img_col,
            
            btns_row,
            btn_project,
            btn_import,
            btn_add_step,
            btn_export,
            btn_halt_processing,

            lbl_init_img,
            whole_proc_prog_bar,
            processing_col,
            scroll_area,
            scroll_pack,
        };

        line.auto_resize(RectArea::new(line.x, line.y, line.w, line.h));

        line
    }

    pub fn process_event_loop(&mut self, app: app::App) -> Result<(), MyError> {
        if let Some(msg) = self.receiver.recv() {
            match msg {
                Message::Project(msg) => {
                    match msg {
                        Project::Import (import_type) => {
                            match self.try_inport_initial_img(import_type) {
                                Ok(_) => {}
                                Err(err) => show_err_msg(self.center(), err)
                            };
                        },
                        Project::SaveProject => {
                            match self.try_save_project() {
                                Ok(done) => if done {
                                    show_info_msg(self.center(), "Проект успешно сохранен");
                                },
                                Err(err) => show_err_msg(self.center(), err),
                            }
                        },
                        Project::LoadProject => {
                            if let Err(err) = self.try_load_project() {
                                show_err_msg(self.center(), err);
                            }
                        },
                        Project::SaveResults => {
                            match self.try_save_results() {
                                Ok(_) => show_info_msg(self.center(), "Результаты успешно сохранены"),
                                Err(err) => show_err_msg(self.center(), err),
                            }
                        }
                    };
                },
                Message::AddStep(msg) => self.add_step_with_dlg(msg, app),
                Message::StepOp(msg) => {
                    match msg {
                        StepOp::EditStep { step_num } => self.edit_step_with_dlg(step_num, app),
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
                },
                Message::Processing(msg) => {
                    let set_all_controls_active = |owner: &mut Self, active: bool| {
                        for step in owner.steps.iter_mut() {
                            step.set_buttons_active(active);
                        }

                        owner.btn_project.set_active(active);
                        owner.btn_import.set_active(active);
                        owner.btn_add_step.set_active(active);
                        owner.btn_export.set_active(active);
                        owner.btn_halt_processing.set_active(!active);
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
                                Err(err) => show_err_msg(self.center(), err)
                            };
                        },
                        Processing::StepProgress { step_num, cur_percents } => {
                            let whole_prog = (step_num * 100 + cur_percents) / self.steps.len();
                            self.whole_proc_prog_bar.set_value(whole_prog);

                            self.steps[step_num].display_progress(cur_percents);
                        },
                        Processing::StepIsCompleted { step_num } => {
                            let processing_continues: bool = match self.on_step_completed(step_num) {
                                Ok(continued) => continued,
                                Err(err) => {
                                    show_err_msg(self.center(), err);
                                    false
                                },
                            };

                            if !processing_continues {
                                set_all_controls_active(self, true);
                                self.whole_proc_prog_bar.hide();
                            }
                        },
                    };
                }
            };
        }            
              
        Ok(())
    }

    fn center(&self) -> Pos { Pos::new(self.x + self.w / 2, self.y + self.h / 2) }


    fn add_step_with_dlg(&mut self, msg: AddStep, app: app::App) -> () {
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

        let mut step_editor = StepEditor::new();
        if !step_editor.edit_with_dlg(app, &mut filter) { return; }
    }

    fn add_step(&mut self, filter: FilterBase) {
        self.scroll_pack.begin();

        self.steps.push(ProcessingStep::new(self.w, self.h, self.steps.len(), filter));

        self.scroll_pack.end();
    }

    fn edit_step_with_dlg(&mut self, step_num: usize, app: app::App) {
        let step = &mut self.steps[step_num];

        let mut step_editor = StepEditor::new();

        if step_editor.edit_with_dlg(app, step.filter_mut()) { 
            step.update_step_description();
        }
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

        let init_img: Img = if step_num == 0 {
            self.img_presenter.image_copy().unwrap()
        } else {
            match self.steps[step_num - 1].get_data_copy() {
                Ok(img_copy) => img_copy,
                Err(err) => { 
                    return Err(MyError::new(format!("Необходим результат предыдущего шага для обработки текущего: {}", err.get_message()))); 
                }
            }
        };
            
        let filter_copy = self.steps[step_num].filter().get_copy();

        self.steps[step_num].start_processing();

        self.background_worker.put_task(step_num, filter_copy, init_img, do_until_end);

        Ok(())
    }

    fn on_step_completed(&mut self, step_num: usize) -> Result<bool, MyError> {
        let task_result = self.background_worker.take_result();

        let processing_continues = task_result.do_until_end && step_num < self.steps.len() - 1;

        let processing_was_halted: bool = task_result.img.is_none();

        self.steps[step_num].display_result(task_result.img)?;

        if processing_was_halted {
            return Ok(false);
        }

        if processing_continues {
            self.try_start_step(step_num + 1, processing_continues)?;
        }
        
        Ok(processing_continues)
    }

    pub fn auto_resize(&mut self, area: RectArea) -> bool {
        if self.x == area.x() && self.y == area.y() && self.w == area.w() && self.h == area.h() {
            return false;
        }

        self.x = area.x();
        self.y = area.y();
        self.w = area.w();
        self.h = area.h();

        self.main_row.resize(self.w, self.h);

        self.init_img_col.resize(self.w / 2, self.h);

        self.processing_col.resize(self.w / 2, self.h);

        self.scroll_area.set_size(self.w / 2, self.h);

        self.scroll_pack.set_size(self.w / 2 - PADDING, self.h);

        let img_pres_y = self.btns_row.h() + self.lbl_init_img.h();
        self.img_presenter.resize(self.w / 2, self.h - img_pres_y);

        for step in self.steps.iter_mut() {
            step.auto_resize(self.w / 2);
        }

        return true;
    }

    
    fn try_inport_initial_img(&mut self, import_type: ImportType) -> Result<(), MyError> {
        if self.img_presenter.has_image() {
            if small_dlg::confirm_with_dlg(self.center(), "Для открытия нового изображения нужно удалить предыдущие результаты. Продолжить?") {
                for step in self.steps.iter_mut() {
                    step.clear_result();
                }
            } else {
                return Ok(());
            }
        }

        let init_image = match import_type {
            ImportType::FromFile => {
                let mut dlg = dialog::FileDialog::new(dialog::FileDialogType::BrowseFile);
                dlg.show();
                let path_buf = dlg.filename();
        
                if let Some(p) = path_buf.to_str() {
                    if p.is_empty() { return Ok(()); }
                }     

                let sh_im = fltk::image::SharedImage::load(path_buf)?;

                if sh_im.w() < 0 { return Err(MyError::new("Ширина загруженного изображения < 0".to_string())); }
                if sh_im.h() < 0 { return Err(MyError::new("Высота загруженного изображения < 0".to_string())); }
        
                Img::from(sh_im)
            },
            ImportType::FromSystemClipoard => {
                match app::event_clipboard_image() {
                    Some(rgb_img) => Img::from(rgb_img),
                    None => { return Err(MyError::new("Не удалось загрузить изображение из системного буфера".to_string())); },
                }
            },
        };


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
    
    fn try_load_project(&mut self) -> Result<(), MyError> {
        if self.steps.len() > 0 {
            if confirm_with_dlg(self.center(),
                "Есть несохраненный проект. Открыть вместо него?") 
            {
                while self.steps.len() > 0 {
                    self.delete_step(0);
                }
            } else {
                return Ok(());
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
            return Ok(());
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

                    if !confirm_with_dlg(self.center(), &question) {
                        break 'out;
                    }
                },
            }
        }

        Ok(())
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

            self.steps[step_num].image_ref().unwrap().try_save(&file_path)?;
        }

        Ok(true)
    }
}