use std::{fs::{self, File}, io::{Read, Write}, usize};
use chrono::{Local, format::{DelayedFormat, StrftimeItems}};
use fltk::{app::{self, Receiver, Sender}, dialog, group, prelude::{GroupExt, WidgetExt}};
use crate::{AssetItem, filter::{color_channel::*, linear::*, non_linear::*}, message::*, my_component::{Alignable, container::*, img_presenter::MyImgPresenter, step_editor, usual::{MyButton, MyLabel, MyMenuButton, MyProgressBar}}, my_err::MyError, small_dlg::{self, *}, utils::{self, Pos}};

use super::{PADDING, step::ProcessingStep};
use crate::processing::*;


pub struct ProcessingLine {
    steps_widgets: Vec<ProcessingStep>,
    tx: Sender<Msg>, rx: Receiver<Msg>,
    background_worker: BackgroundWorker,
    process_until_end: Option<bool>,

    // graphical parts
    main_row: MyRow,

    btns_row: MyRow,
    btn_project: MyMenuButton,
    btn_import: MyMenuButton,
    btn_add_step: MyMenuButton,
    btn_export: MyMenuButton,
    btn_halt_processing: MyButton,

    init_img_col: MyColumn,
    total_progress_bar: MyProgressBar,
    lbl_init_img: MyLabel,
    img_presenter: MyImgPresenter,

    processing_col: MyColumn,
    scroll_area: group::Scroll,
    scroll_pack: group::Pack,    
}

impl ProcessingLine {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        let (tx, rx) = app::channel::<Msg>();

        let mut main_row = MyRow::new(w).with_pos(x, y);
            
        let mut init_img_col = MyColumn::new(w / 2, h);

        let mut btns_row = MyRow::new(w / 2);

        let mut btn_project = MyMenuButton::with_label("Проект");
        btn_project.add_emit("Зарузить", tx, Msg::Project ( Project::LoadProject ) );
        btn_project.add_emit("Сохранить как", tx, Msg::Project ( Project::SaveProject ) );

        let mut btn_import = MyMenuButton::with_img_and_tooltip(AssetItem::Import, "Импорт");
        btn_import.add_emit("Файл", tx, 
            Msg::Project(Project::Import ( ImportType::File ) ) );
        btn_import.add_emit("Системный буфер обмена", tx, 
            Msg::Project(Project::Import ( ImportType::SystemClipoard ) ) );
            
        let mut btn_add_step = MyMenuButton::with_img_and_tooltip(AssetItem::AddStep, "Добавить шаг");
        btn_add_step.add_emit("Цветной => ч\\/б", tx, Msg::StepOp ( StepOp::AddStep( AddStep::Rgb2Gray ) ) );
        btn_add_step.add_emit("Линейный фильтр (усредняющий)", tx, Msg::StepOp ( StepOp::AddStep ( AddStep::LinMean ) ) );
        btn_add_step.add_emit("Линейный фильтр (гауссовский)", tx, Msg::StepOp ( StepOp::AddStep ( AddStep::LinGauss ) ) );
        btn_add_step.add_emit("Линейный фильтр (другой)", tx, Msg::StepOp ( StepOp::AddStep ( AddStep::LinCustom ) ) );
        btn_add_step.add_emit("Медианный фильтр", tx, Msg::StepOp ( StepOp::AddStep ( AddStep::Median ) ) );
        btn_add_step.add_emit("Локальный контраст (гистограмма)", tx, Msg::StepOp ( StepOp::AddStep ( AddStep::HistogramLocalContrast ) ) );
        btn_add_step.add_emit("Обрезание яркости", tx, Msg::StepOp( StepOp::AddStep ( AddStep::CutBrightness ) ) );
        btn_add_step.add_emit("Эквализация гистограммы", tx, Msg::StepOp ( StepOp::AddStep ( AddStep::HistogramEqualizer ) ) );
        btn_add_step.add_emit("Убрать канал", tx, Msg::StepOp ( StepOp::AddStep ( AddStep::NeutralizeChannel ) ) );
        btn_add_step.add_emit("Выделить канал", tx, Msg::StepOp ( StepOp::AddStep ( AddStep::ExtractChannel ) ) );

        let mut btn_export = MyMenuButton::with_img_and_tooltip(AssetItem::Export, "Экспорт");
        btn_export.add_emit("Сохранить результаты", tx, Msg::Project ( Project::SaveResults ( SaveResults::Start ) ) );

        let mut btn_halt_processing = MyButton::with_img_and_tooltip(AssetItem::HaltProcessing, "Прервать обработку");
        btn_halt_processing.set_emit(tx, Msg::Proc ( Proc::Halted ) );
        btn_halt_processing.set_active(false);
        
        btns_row.end();

        let lbl_init_img = MyLabel::new("Исходное изображение", w / 2);

        let mut total_progress_bar = MyProgressBar::new(w / 2, 30);
        total_progress_bar.hide();
            
        let presenter_h = init_img_col.height_left();
        let img_presenter = MyImgPresenter::new(w / 2, presenter_h);
        
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

        let background_worker = BackgroundWorker::new(tx.clone());

        let mut line = ProcessingLine {
            steps_widgets: Vec::<ProcessingStep>::new(),
            tx, rx,
            background_worker,
            process_until_end: None,
            
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
            total_progress_bar,
            processing_col,
            scroll_area,
            scroll_pack,
        };

        line.resize(w, h);

        line
    }

    pub fn process_event_loop(&mut self, app: app::App) -> Result<(), MyError> {
        let set_controls_active = |owner: &mut Self, active: bool| {
            for step in owner.steps_widgets.iter_mut() {
                step.set_buttons_active(active);
            }

            owner.btn_project.set_active(active);
            owner.btn_import.set_active(active);
            owner.btn_add_step.set_active(active);
            owner.btn_export.set_active(active);
            owner.btn_halt_processing.set_active(!active);
        };

        'out: while let Some(msg) = self.rx.recv() {
            match msg {
                Msg::Project(msg) => {
                    match msg {
                        Project::Import (import_type) => {
                            match self.try_import_initial_img(import_type) {
                                Ok(_) => {}
                                Err(err) => show_err_msg(self.get_center_pos(), err)
                            };
                        },
                        Project::SaveProject => {
                            match self.try_save_project() {
                                Ok(done) => if done {
                                    show_info_msg(self.get_center_pos(), "Проект успешно сохранен");
                                },
                                Err(err) => show_err_msg(self.get_center_pos(), err),
                            }
                        },
                        Project::LoadProject => {
                            if let Err(err) = self.try_load_project() {
                                show_err_msg(self.get_center_pos(), err);
                            }
                        },
                        Project::SaveResults (msg) => {
                            match msg {
                                SaveResults::Start => {
                                    let started = match self.try_start_saving_results() {
                                        Ok(started) => started,
                                        Err(err) => {
                                            show_err_msg(self.get_center_pos(), err);
                                            false
                                        },
                                    };

                                    if started {
                                        set_controls_active(self, false);
                                        self.total_progress_bar.show();
                                        self.total_progress_bar.reset("Экспорт".to_string());
                                    }
                                },
                                SaveResults::Completed { percents, last_result_is_saved } => {
                                    self.total_progress_bar.set_value(percents);

                                    if last_result_is_saved {
                                        set_controls_active(self, true);
                                        self.total_progress_bar.hide();
                                        show_info_msg(self.get_center_pos(), "Результаты успешно сохранены");
                                    }
                                },
                                SaveResults::Error => {
                                    set_controls_active(self, true);
                                    self.total_progress_bar.hide();
                                    let err = self.background_worker.get_saving_steps_results_error();
                                    show_err_msg(self.get_center_pos(), err.unwrap());
                                },
                            }
                        }
                    };
                },
                Msg::StepOp(msg) => {
                    match msg {
                        StepOp::AddStep(msg) => self.add_step_with_dlg(msg, app),
                        StepOp::Edit { step_num } => self.edit_step_with_dlg(step_num, app),
                        StepOp::Delete { step_num } => self.remove_step(step_num),
                        StepOp::Move { step_num, direction } => {
                            let (upper_num, lower_num) = match direction {
                                MoveStep::Up => if step_num > 0 { 
                                    (step_num - 1, step_num) 
                                } else { 
                                    break 'out; 
                                },
                                MoveStep::Down => if step_num < self.steps_widgets.len() - 1 {
                                    (step_num, step_num + 1)
                                } else {
                                    break 'out;
                                },
                            };           

                            self.scroll_pack.begin();

                            for step in self.steps_widgets[upper_num..].iter_mut() {
                                step.clear_displayed_result();
                                step.remove_self_from(&mut self.scroll_pack);
                            }

                            self.steps_widgets.swap(upper_num, lower_num);

                            for step in self.steps_widgets[upper_num..].iter_mut() {
                                step.draw_self_on(&mut self.scroll_pack);
                            }

                            self.scroll_pack.end();

                            for step_num in upper_num..self.steps_widgets.len() {
                                self.steps_widgets[step_num].update_btn_emits(step_num);
                            }

                            self.main_row.widget_mut().redraw();
                        },
                    }
                },
                Msg::Proc(msg) => {
                    match msg {
                        Proc::ChainIsStarted { step_num, process_until_end: do_until_end } => {
                            match self.try_start_chain(step_num, do_until_end) {
                                Ok(_) => {
                                    set_controls_active(self, false);
    
                                    self.total_progress_bar.show();
                                    self.total_progress_bar.reset("Общий прогресс".to_string());
    
                                    for step_widget in &mut self.steps_widgets[step_num..] {
                                        step_widget.clear_displayed_result();
                                    }
                                }
                                Err(err) => show_err_msg(self.get_center_pos(), err)
                            };
                        },
                        Proc::StepProgress { num, step_percents, total_percents } => {
                            self.total_progress_bar.set_value(total_percents);
                            self.steps_widgets[num].display_progress(step_percents);
                        },
                        Proc::Halted => {
                            self.background_worker.halt_processing();
                        },
                        Proc::CompletedStep { num } => {
                            let mut proc_result = self.background_worker.get_result(); 

                            self.steps_widgets[num].display_result(proc_result.get_image());
                            self.steps_widgets[num].set_step_descr(&self.background_worker.get_step_descr(num));

                            let processing_continues: bool = 
                                self.process_until_end.unwrap()
                                && !proc_result.it_is_the_last_step()
                                && !proc_result.processing_was_halted();

                            if processing_continues {
                                self.start_step(num + 1);
                            } else {
                                set_controls_active(self, true);
                                self.total_progress_bar.hide();
                                self.process_until_end = None;
                            }
                        },
                    };
                }
            };
        }            
              
        Ok(())
    }

    fn get_center_pos(&self) -> Pos { 
        Pos::new(
            self.main_row.x() + self.main_row.w() / 2, 
            self.main_row.y() + self.main_row.h() / 2) 
    }


    fn add_step_with_dlg(&mut self, msg: AddStep, app: app::App) -> () {
        match step_editor::create(msg, app) {
            Some(filter) => {
                self.add_step_to_background_worker_and_as_widget(filter);
            },
            None => return,
        };
    }

    fn edit_step_with_dlg(&mut self, step_num: usize, app: app::App) {
        let step = &mut self.steps_widgets[step_num];

        let edited = self.background_worker.edit_step(
            step_num, 
            |filter| {
                step_editor::edit(app, filter)
            }
        );

        if edited {
            step.set_step_descr(&self.background_worker.get_step_descr(step_num));
        }
    }

    fn remove_step(&mut self, step_num: usize) {
        self.scroll_pack.begin();
        self.steps_widgets[step_num].remove_self_from(&mut self.scroll_pack);
        self.scroll_pack.end();

        self.steps_widgets.remove(step_num);

        for sn in step_num..self.steps_widgets.len() {
            self.steps_widgets[sn].update_btn_emits(sn);
        }

        crate::notify_content_changed();

        self.background_worker.remove_step(step_num);
    }


    fn try_start_chain(&mut self, step_num: usize, process_until_end: bool) -> Result<(), MyError> {
        match self.background_worker.check_if_can_start_processing(step_num) {
            StartProcResult::NoInitialImg => {
                Err(MyError::new("Необходимо загрузить изображение для обработки".to_string()))
            },
            StartProcResult::NoPrevStepImg => {
                Err(MyError::new("Необходим результат предыдущего шага для обработки текущего".to_string()))
            },
            StartProcResult::CanStart => {
                self.process_until_end = Some(process_until_end);
                self.start_step(step_num);
                Ok(())
            }
        }
    }

    fn start_step(&mut self, step_num: usize) {
        self.steps_widgets[step_num].display_processing_start();

        let crop_area = if step_num == 0 {
            self.img_presenter.get_selection_rect()
        } else {
            self.steps_widgets[step_num - 1].get_selection_rect()
        };

        self.background_worker.start_processing(step_num, crop_area);
    }

    
    fn try_import_initial_img(&mut self, import_type: ImportType) -> Result<(), MyError> {
        if self.background_worker.has_initial_img() {
            if small_dlg::confirm_with_dlg(
                self.get_center_pos(), 
                "Для открытия нового изображения нужно удалить предыдущие результаты. Продолжить?"
            ) {
                for step in self.steps_widgets.iter_mut() {
                    step.clear_displayed_result();
                }
            } else {
                return Ok(());
            }
        }

        match import_type {
            ImportType::File => {
                let mut dlg = dialog::FileDialog::new(dialog::FileDialogType::BrowseFile);
                dlg.show();
                let path_buf = dlg.filename();

                let path = path_buf.to_str().unwrap();
        
                if path.is_empty() { return Ok(()); }
        
                self.background_worker.load_initial_img(path)?;
            },
            ImportType::SystemClipoard => {
                todo!()
            },
        };

        self.lbl_init_img.set_text(&self.background_worker.get_init_img_descr());

        self.img_presenter.set_img(self.background_worker.get_init_img_drawable());

        crate::notify_content_changed();

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
        if self.steps_widgets.len() == 0 {
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

        for step_num in 0..self.steps_widgets.len() {
            let filter_save_name: String = self.background_worker.get_filter_save_name(step_num);

            file_content.push_str(&filter_save_name);
            file_content.push_str("\n");

            if let Some(params_str) = self.background_worker.get_filter_params_as_str(step_num) {
                file_content.push_str(&params_str);
            }
            file_content.push_str("\n");

            if step_num < self.steps_widgets.len() - 1 {
                file_content.push_str(Self::FILTER_SAVE_SEPARATOR);
                file_content.push_str("\n");
            }
        }

        file.write_all(&file_content.as_bytes())?;
        file.sync_all()?;

        Ok(true)
    }
    
    fn try_load_project(&mut self) -> Result<(), MyError> {
        if self.steps_widgets.len() > 0 {
            if confirm_with_dlg(self.get_center_pos(),
                "Есть несохраненный проект. Открыть вместо него?") 
            {
                while self.steps_widgets.len() > 0 {
                    self.remove_step(0);
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

        self.steps_widgets.reserve(filters_iter.len());

        'out: for filter_str in filters_iter.iter() {
            let mut lines_iter = utils::LinesIter::new(filter_str);
            let filter_name = lines_iter.next_or_empty().to_string();
            let filter_content = lines_iter.all_left(true);

            match Self::try_parce_filter(&filter_name, &filter_content) {
                Ok(filter) => {
                    self.add_step_to_background_worker_and_as_widget(filter);
                },
                Err(err) => {
                    let question = format!(
                        "Ошибка формата при чтении фильтра '{}': '{}'. Продолжить загрузку следующих шагов проекта?", 
                        filter_name, err.to_string());

                    if !confirm_with_dlg(self.get_center_pos(), &question) {
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

    fn try_start_saving_results(&self) -> Result<bool, MyError> {
        match self.background_worker.check_if_can_save_results() {
            StartResultsSavingResult::NoSteps => 
                return Err(MyError::new("В проекте нет шагов обработки для сохранения их результатов".to_string())),
            StartResultsSavingResult::NotAllStepsHaveResult => 
                return Err(MyError::new("Не все шаги имеют результаты для сохранения".to_string())),
            StartResultsSavingResult::CanStart => {},
        }

        let mut dlg = dialog::FileDialog::new(dialog::FileDialogType::BrowseSaveDir);
        dlg.set_title("Сохранение результатов");

        dlg.show(); 

        let path_buf = dlg.filename();

        let mut proj_path: String = match path_buf.to_str() {
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
        
        self.background_worker.start_saving_steps_results(proj_path);

        Ok(true)
    }
    

    fn add_step_to_background_worker_and_as_widget(&mut self, filter: FilterBase) {
        self.background_worker.add_step(filter);

        self.scroll_pack.begin();

        self.scroll_pack.set_size(self.scroll_pack.w(), self.scroll_pack.h() + self.h());

        let step_num = self.steps_widgets.len();

        let mut new_step = ProcessingStep::new(
            self.w() / 2, self.h(), 
            step_num, 
            self.tx);

        new_step.set_step_descr(&self.background_worker.get_step_descr(step_num));
        
        self.steps_widgets.push(new_step);

        self.scroll_pack.end();

        crate::notify_content_changed();
    }
}

impl Alignable for ProcessingLine {
    fn resize(&mut self, w: i32, h: i32) {
        self.main_row.resize(w, h);

        self.init_img_col.resize(w / 2, h);

        self.processing_col.resize(w / 2, h);

        self.scroll_area.set_size(w / 2, h);

        self.scroll_pack.set_size(w / 2 - PADDING, self.scroll_pack.h());

        let img_pres_y = self.btns_row.h() + self.lbl_init_img.h();
        self.img_presenter.resize(w / 2, h - img_pres_y);

        for step in self.steps_widgets.iter_mut() {
            step.resize(w / 2, h);
        }
    }

    fn x(&self) -> i32 { self.main_row.x() }

    fn y(&self) -> i32 { self.main_row.y() }

    fn w(&self) -> i32 { self.main_row.w() }

    fn h(&self) -> i32 { self.main_row.h() }
}