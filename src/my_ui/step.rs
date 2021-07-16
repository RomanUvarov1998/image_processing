use fltk::{app::{Sender}, group, image::RgbImage, prelude::{GroupExt}};
use crate::{img::PixelsArea, my_ui::{Alignable, container::{MyColumn, MyRow}, img_presenter::MyImgPresenter, usual::{MyButton, MyLabel, MyMenuButton, MyProgressBar}}};
use super::{PADDING, message::*};
use super::embedded_images::AssetItem;

pub struct ProcessingStep {
    step_num: usize,
    tx: Sender<Msg>,

    main_column: MyColumn,
    btn_run: MyMenuButton,
    btn_edit: MyButton,
    btn_delete: MyButton,
    btn_reorder: MyMenuButton,
    label_step_name: MyLabel,
    prog_bar: MyProgressBar,
    img_presenter: MyImgPresenter,
}

impl ProcessingStep {
    pub fn new(w: i32, h: i32, step_num: usize, tx: fltk::app::Sender<Msg>) -> Self {
        let mut main_column = MyColumn::new(w, h);

        let label_step_name = MyLabel::new("", w);

        let mut btns_row = MyRow::new(w);

        let btn_run = MyMenuButton::with_img_and_tooltip(AssetItem::RunStepsChain, "Запустить");
        let btn_edit = MyButton::with_img_and_tooltip(AssetItem::EditStep, "Изменить");
        let btn_delete = MyButton::with_img_and_tooltip(AssetItem::DeleteStep, "Удалить");
        let btn_reorder = MyMenuButton::with_img_and_tooltip(AssetItem::ReorderSteps, "Переупорядочить");

        btns_row.end();        

        let mut prog_bar = MyProgressBar::new(w - PADDING, 30);
        prog_bar.hide();
            
        let img_presenter = MyImgPresenter::new(
            w - PADDING, h - btns_row.h() * 2);
        
        main_column.end();
        
        let mut step = ProcessingStep { 
            main_column,
            btn_run,
            btn_edit,
            btn_delete,
            btn_reorder,
            label_step_name,
            prog_bar,
            img_presenter, 
            step_num,
            tx
        };

        step.update_btn_emits(step_num);

        step
    }


    pub fn remove_self_from(&mut self, pack: &mut group::Pack) {
        pack.remove(self.main_column.widget_mut());
    }
    
    
    pub fn clear_displayed_result(&mut self) {
        self.img_presenter.clear_image();
    }

    pub fn set_step_descr(&mut self, descr: &str) {
        self.label_step_name.set_text(descr);
    }

    pub fn update_btn_emits(&mut self, step_num: usize) {
        self.btn_run.add_emit("Только этот шаг", self.tx, 
            Msg::Proc(Proc::StartStepsChain { step_num, process_until_end: false }));
        self.btn_run.add_emit("Этот шаг и все шаги ниже", self.tx, 
            Msg::Proc(Proc::StartStepsChain { step_num, process_until_end: true }));
        self.btn_edit.set_emit(self.tx, Msg::StepOp(StepOp::Edit { step_num }));
        self.btn_delete.set_emit(self.tx, Msg::StepOp(StepOp::Delete { step_num }));
        self.btn_reorder.add_emit("Сдвинуть вверх", self.tx, Msg::StepOp(StepOp::Move { step_num, direction: MoveStep::Up } ));
        self.btn_reorder.add_emit("Сдвинуть вниз", self.tx, Msg::StepOp(StepOp::Move { step_num, direction: MoveStep::Down } ));
        self.step_num = step_num;
    }

    pub fn set_buttons_active(&mut self, active: bool) {
        self.btn_run.set_active(active);
        self.btn_edit.set_active(active);
        self.btn_delete.set_active(active);
        self.btn_reorder.set_active(active);
    }

    pub fn get_selection_rect(&self) -> Option<PixelsArea> {
        self.img_presenter.get_selection_rect()
    }

    pub fn display_processing_start(&mut self) {
        self.prog_bar.show();
        self.prog_bar.reset("Обработка".to_string());
        self.img_presenter.clear_image(); 
    }

    pub fn display_progress(&mut self, percents: usize) {
        self.prog_bar.set_value(percents);
    }

    pub fn display_result(&mut self, processed_img: Option<RgbImage>) {
        self.prog_bar.hide();
                        
        match processed_img {
            Some(img) => self.img_presenter.set_img(img),
            None => self.img_presenter.clear_image(),
        }
    }
}

impl Alignable for ProcessingStep {
    fn resize(&mut self, w: i32, h: i32) {
        self.label_step_name.resize(w, self.label_step_name.h());
        self.prog_bar.resize(w, self.prog_bar.h());
        self.img_presenter.resize(w, h - self.label_step_name.h() - self.prog_bar.h());
    }

    fn x(&self) -> i32 { self.main_column.x() }

    fn y(&self) -> i32 { self.main_column.y() }

    fn w(&self) -> i32 { self.main_column.w() }

    fn h(&self) -> i32 { self.main_column.h() }
}