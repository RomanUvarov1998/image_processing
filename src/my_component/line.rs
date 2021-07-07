use std::{fs::{File}, io::{Read, Write}, usize};
use chrono::{Local, format::{DelayedFormat, StrftimeItems}};
use fltk::{app::{self, Receiver, Sender}, dialog, group, prelude::{GroupExt, WidgetExt}};
use crate::{AssetItem, filter::{color_channel::*, linear::*, non_linear::*}, my_component::{Alignable, container::*, img_presenter::MyImgPresenter, step_editor, usual::{MyButton, MyLabel, MyMenuButton, MyProgressBar}}, my_err::MyError, small_dlg::{self, *}, utils::{self, Pos}};

use super::{PADDING, message::*, step::ProcessingStep};
use crate::processing::*;


#[derive(Clone, Copy, Debug)]
enum CurrentTask {
    Importing,
    Processing { step_num: usize, process_until_end: bool },
    Exporting
}


pub struct ProcessingLine {
    steps_widgets: Vec<ProcessingStep>,
    tx: Sender<Msg>, rx: Receiver<Msg>,
    bw: BackgroundWorker,
    rx_task: std::sync::mpsc::Receiver<TaskMsg>,
    current_task: Option<CurrentTask>,

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
        btn_export.add_emit("Сохранить результаты", tx, Msg::Project ( Project::Export ) );

        let mut btn_halt_processing = MyButton::with_img_and_tooltip(AssetItem::HaltProcessing, "Прервать обработку");
        btn_halt_processing.set_emit(tx, Msg::Proc ( Proc::HaltStepsChain ) );
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

        let (tx_task, rx_task) = std::sync::mpsc::channel::<TaskMsg>();
        let background_worker = BackgroundWorker::new(tx_task);

        let mut line = ProcessingLine {
            steps_widgets: Vec::<ProcessingStep>::new(),
            tx, rx,
            bw: background_worker,
            rx_task,
            current_task: None,
            
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
        while let Some(msg) = self.rx.recv() {
            if let Err(err) = match msg {
                Msg::Project(msg) => self.process_project_msg(msg),
                Msg::StepOp(msg) => self.process_step_op_msg(msg, app),
                Msg::Proc(msg) => self.process_proc_msg(msg)
            } {
                show_err_msg(self.get_center_pos(), err);
            }
        }      

        crate::notify_content_changed();  
        
        Ok(())
    }


    fn process_project_msg(&mut self, msg: Project) -> Result<(), MyError> {
        match msg {
            Project::Import (import_type) => self.process_project_import_msg(import_type),
            Project::SaveProject => self.process_project_save_msg(),
            Project::LoadProject => self.process_project_load_msg(),
            Project::Export => self.process_project_start_export_msg()
        }
    }

    fn process_step_op_msg(&mut self, msg: StepOp, app: app::App) -> Result<(), MyError> {
        match msg {
            StepOp::AddStep(msg) => self.process_step_op_add_step_msg(msg, app),
            StepOp::Edit { step_num } => self.process_step_op_edit_step_msg(step_num, app),
            StepOp::Delete { step_num } => self.process_step_op_remove_step_msg(step_num),
            StepOp::Move { step_num, direction } => self.process_step_op_reorder_step_msg(step_num, direction),
        }
    }

    fn process_proc_msg(&mut self, msg: Proc) -> Result<(), MyError> {
        match msg {
            Proc::StartStepsChain { step_num, process_until_end } => 
                self.process_proc_start_chain_msg(step_num, process_until_end),
            Proc::HaltStepsChain => self.process_proc_halt_msg(),
        }
    }


    fn process_project_import_msg(&mut self, import_type: ImportType) -> Result<(), MyError> {
        if self.bw.unlocked().has_initial_img() {
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

                self.img_presenter.clear_image();
                self.set_controls_active(false);
                self.total_progress_bar.show();
                self.total_progress_bar.reset("Импорт".to_string());
                self.current_task = Some( CurrentTask::Importing );
        
                self.bw.start_task(ImportTask::new(path.to_string()));
            },
            ImportType::SystemClipoard => {
                todo!()
            },
        };

        Ok(())
    }

    fn process_project_save_msg(&mut self) -> Result<(), MyError> {
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
            return Ok(());
        }

        let mut file = match File::create(proj_path) {
            Ok(f) => f,
            Err(err) => { return Err(MyError::new(err.to_string())); }
        };

        let mut file_content = String::new();

        for step_num in 0..self.steps_widgets.len() {
            let filter_save_name: String = self.bw.unlocked().get_filter_save_name(step_num);

            file_content.push_str(&filter_save_name);
            file_content.push_str("\n");

            if let Some(params_str) = self.bw.unlocked().get_filter_params_as_str(step_num) {
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
        
        show_info_msg(self.get_center_pos(), "Проект успешно сохранен");
        
        Ok(())
    }

    fn process_project_load_msg(&mut self) -> Result<(), MyError> {
        if self.steps_widgets.len() > 0 {
            if confirm_with_dlg(self.get_center_pos(),
                "Есть несохраненный проект. Открыть вместо него?") 
            {
                while self.steps_widgets.len() > 0 {
                    self.process_step_op_remove_step_msg(0)?;
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

    fn process_project_start_export_msg(&mut self) -> Result<(), MyError> {
        match self.bw.unlocked().check_if_can_export() {
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
            return Ok(());
        }
        
        self.set_controls_active(false);
        self.total_progress_bar.show();
        self.total_progress_bar.reset("Экспорт".to_string());
        self.current_task = Some( CurrentTask::Exporting );

        proj_path.push_str("/");
        let dir_name = format!("Results {}", Self::cur_time_str());
        proj_path.push_str(&dir_name);
        
        self.bw.start_task(ExportTask::new(proj_path));

        Ok(())        
    }


    fn process_step_op_add_step_msg(&mut self, msg: AddStep, app: app::App) -> Result<(), MyError> {
        if let Some(filter) = step_editor::create(msg, app) {
            self.add_step_to_background_worker_and_as_widget(filter);
        }

        Ok(())
    }

    fn process_step_op_edit_step_msg(&mut self, step_num: usize, app: app::App) -> Result<(), MyError> {
        let step = &mut self.steps_widgets[step_num];

        let edited = self.bw.unlocked().edit_step(
            step_num, 
            |filter| {
                step_editor::edit(app, filter)
            }
        );

        if edited {
            step.set_step_descr(&self.bw.unlocked().get_step_descr(step_num));
        }

        Ok(())
    }
    
    fn process_step_op_remove_step_msg(&mut self, step_num: usize) -> Result<(), MyError> {
        self.scroll_pack.begin();
        self.steps_widgets[step_num].remove_self_from(&mut self.scroll_pack);
        self.scroll_pack.end();

        self.steps_widgets.remove(step_num);

        for sn in step_num..self.steps_widgets.len() {
            self.steps_widgets[sn].update_btn_emits(sn);
        }

        self.bw.unlocked().remove_step(step_num);
    
        Ok(())
    }
    
    fn process_step_op_reorder_step_msg(&mut self, step_num: usize, direction: MoveStep) -> Result<(), MyError> {
        let (upper_num, lower_num) = match direction {
            MoveStep::Up => if step_num > 0 { 
                (step_num - 1, step_num) 
            } else { 
                return Ok(()); 
            },
            MoveStep::Down => if step_num < self.steps_widgets.len() - 1 {
                (step_num, step_num + 1)
            } else {
                return Ok(());
            },
        };     

        self.bw.unlocked().swap_steps(upper_num, lower_num);

        for step in self.steps_widgets[upper_num..].iter_mut() {
            step.clear_displayed_result();
        } 
        self.steps_widgets[upper_num].set_step_descr(&self.bw.unlocked().get_step_descr(upper_num));
        self.steps_widgets[lower_num].set_step_descr(&self.bw.unlocked().get_step_descr(lower_num));

        Ok(())
    }


    fn process_proc_start_chain_msg(&mut self, step_num: usize, process_until_end: bool) -> Result<(), MyError> {
        match self.bw.check_if_can_start_processing(step_num) {
            StartProcResult::NoInitialImg => {
                Err(MyError::new("Необходимо загрузить изображение для обработки".to_string()))
            },
            StartProcResult::NoPrevStepImg => {
                Err(MyError::new("Необходим результат предыдущего шага для обработки текущего".to_string()))
            },
            StartProcResult::CanStart => {
                self.set_controls_active(false);
                self.total_progress_bar.show();
                self.total_progress_bar.reset("Общий прогресс".to_string());

                for step_widget in &mut self.steps_widgets[step_num..] {
                    step_widget.clear_displayed_result();
                }

                self.start_step_processing(step_num, process_until_end);

                Ok(())
            }
        }
    }

    fn process_proc_halt_msg(&mut self) -> Result<(), MyError> {
        self.bw.halt_processing();
        Ok(())
    }

    

    pub fn process_task_message_loop(&mut self) -> Result<(), MyError> {
        if let Some(task) = self.current_task {
            while let Ok(msg) = self.rx_task.try_recv() {
                if let Err(err) = match task {
                    CurrentTask::Importing => self.process_task_import_msg(msg),
                    CurrentTask::Processing { step_num, process_until_end } => 
                        self.process_task_processing_msg(msg, step_num, process_until_end),
                    CurrentTask::Exporting => 
                        self.process_task_export_msg(msg),
                } {
                    show_err_msg(self.get_center_pos(), err);
                }
            }
        
            crate::notify_content_changed();
        }

        Ok(())
    }

    fn process_task_import_msg(&mut self, msg: TaskMsg) -> Result<(), MyError> {
        match msg {
            TaskMsg::Progress { percents } => {
                self.total_progress_bar.set_value(percents);
                Ok(())
            },
            TaskMsg::Finished => {
                self.current_task = None;

                self.set_controls_active(true);
                self.total_progress_bar.hide();

                self.lbl_init_img.set_text(&self.bw.unlocked().get_init_img_descr());
        
                self.img_presenter.set_img(self.bw.unlocked().get_init_img_drawable());
                
                self.bw.unlocked().get_task_result()
            },
        }
    }

    fn process_task_export_msg(&mut self, msg: TaskMsg) -> Result<(), MyError> {
        match msg {
            TaskMsg::Progress { percents } => {
                self.total_progress_bar.set_value(percents);
                Ok(())
            },
            TaskMsg::Finished => {
                self.current_task = None;

                self.set_controls_active(true);
                self.total_progress_bar.hide();
                let export_result = self.bw.unlocked().get_task_result();

                match export_result {
                    Ok(()) => {
                        show_info_msg(self.get_center_pos(), "Результаты успешно сохранены");
                        Ok(())
                    },
                    Err(err) => Err(err),
                }
            },
        }
    }
    
    fn process_task_processing_msg(&mut self, msg: TaskMsg, step_num: usize, process_until_end: bool) -> Result<(), MyError> {
        match msg {
            TaskMsg::Progress { percents } => {
                let total_percents = (step_num * 100 + percents) / self.steps_widgets.len();
                self.total_progress_bar.set_value(total_percents);
                self.steps_widgets[step_num].display_progress(percents);
                Ok(())
            },
            TaskMsg::Finished => {
                let mut bw_unlocked = self.bw.unlocked();

                let drawable = bw_unlocked.get_step_img_drawable(step_num);
                let processing_was_halted = drawable.is_none();
        
                self.steps_widgets[step_num].display_result(drawable);
                self.steps_widgets[step_num].set_step_descr(&bw_unlocked.get_step_descr(step_num));
        
                let it_is_the_last_step: bool = step_num >= bw_unlocked.get_steps_count() - 1;
                
                bw_unlocked.get_task_result()?; 
        
                drop(bw_unlocked);
        
                let processing_continues: bool = 
                    process_until_end
                    && !it_is_the_last_step
                    && !processing_was_halted;
        
                if processing_continues {
                    self.start_step_processing(step_num + 1, true);
                } else {
                    self.set_controls_active(true);
                    self.total_progress_bar.hide();
                    self.current_task = None;
                }
        
                Ok(())
            },
        }
    }



    fn set_controls_active(&mut self, active: bool) {
        for step in self.steps_widgets.iter_mut() {
            step.set_buttons_active(active);
        }

        self.btn_project.set_active(active);
        self.btn_import.set_active(active);
        self.btn_add_step.set_active(active);
        self.btn_export.set_active(active);
        self.btn_halt_processing.set_active(!active);
    }

    fn get_center_pos(&self) -> Pos { 
        Pos::new(
            self.main_row.x() + self.main_row.w() / 2, 
            self.main_row.y() + self.main_row.h() / 2) 
    }

    fn start_step_processing(&mut self, step_num: usize, process_until_end: bool) {
        self.current_task = Some( CurrentTask::Processing { step_num, process_until_end } );

        self.steps_widgets[step_num].display_processing_start();

        let crop_area = if step_num == 0 {
            self.img_presenter.get_selection_rect()
        } else {
            self.steps_widgets[step_num - 1].get_selection_rect()
        };

        self.bw.start_task(ProcTask::new(step_num, crop_area));
    }

    const PROJECT_EXT: &'static str = "ps";
    const FILTER_SAVE_SEPARATOR: &'static str = "||";

    fn cur_time_str() -> String {
        let current_datetime_formatter: DelayedFormat<StrftimeItems> = 
            Local::now().format("%d-%m(%b)-%Y_%a_%_H.%M.%S"); 
        format!("{}", current_datetime_formatter)
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

    fn add_step_to_background_worker_and_as_widget(&mut self, filter: FilterBase) {
        self.bw.unlocked().add_step(filter);

        self.scroll_pack.begin();

        self.scroll_pack.set_size(self.scroll_pack.w(), self.scroll_pack.h() + self.h());

        let step_num = self.steps_widgets.len();

        let mut new_step = ProcessingStep::new(
            self.w() / 2, self.h(), 
            step_num, 
            self.tx);

        new_step.set_step_descr(&self.bw.unlocked().get_step_descr(step_num));
        
        self.steps_widgets.push(new_step);

        self.scroll_pack.end();
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