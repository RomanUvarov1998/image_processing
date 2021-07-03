use fltk::{app::{self, Sender}, group, prelude::{GroupExt}};
use crate::{AssetItem, img::Img, message::{self, Msg, Proc, StepOp}, my_component::{Alignable, container::{MyColumn, MyRow}, img_presenter::MyImgPresenter, usual::{MyButton, MyLabel, MyMenuButton, MyProgressBar}}, my_err::MyError};
use super::{FilterBase, PADDING};

pub struct ProcessingStep {
    main_column: MyColumn,
    btn_run: MyMenuButton,
    btn_edit: MyButton,
    btn_delete: MyButton,
    btn_reorder: MyMenuButton,
    label_step_name: MyLabel,
    prog_bar: MyProgressBar,
    img_presenter: MyImgPresenter,
    filter: FilterBase,
    step_num: usize,
    tx: Sender<Msg>
}

impl ProcessingStep {
    pub fn new(w: i32, h: i32, step_num: usize, filter: FilterBase) -> Self {
        let name: String = filter.get_description();

        let mut main_column = MyColumn::new(w, h);

        let label_step_name = MyLabel::new(&name, w);

        let (tx, _) = app::channel::<Msg>();

        let mut btns_row = MyRow::new(w, 100);

        let btn_run = MyMenuButton::with_img_and_tooltip(AssetItem::RunStepsChain, "Запустить");
        let btn_edit = MyButton::with_img_and_tooltip(AssetItem::EditStep, "Изменить");
        let btn_delete = MyButton::with_img_and_tooltip(AssetItem::DeleteStep, "Удалить");
        let btn_reorder = MyMenuButton::with_img_and_tooltip(AssetItem::ReorderSteps, "Переупорядочить");

        let btns = [btn_run.h(), btn_edit.h(), btn_delete.h(), btn_reorder.h()];

        btns_row.resize(
            btns_row.w(), 
            *btns.iter().max().unwrap());
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
            filter,
            step_num,
            tx
        };

        step.update_btn_emits(step_num);

        step
    }


    pub fn draw_self_on(&mut self, pack: &mut group::Pack) {
        pack.add(self.main_column.widget_mut());
    }

    pub fn remove_self_from(&mut self, pack: &mut group::Pack) {
        pack.remove(self.main_column.widget_mut());
    }
    
    
    pub fn clear_result(&mut self) {
        self.img_presenter.clear_image();
    }

    pub fn update_step_description(&mut self) {
        let filter_description: String = self.filter.get_description();
        let img_description: String = match self.img_presenter.image_ref() {
            Some(img) => img.get_description(),
            None => String::new(),
        };
        self.label_step_name.set_text(&format!("{} {}", &filter_description, &img_description));
    }

    pub fn update_btn_emits(&mut self, step_num: usize) {
        self.btn_run.add_emit("Только этот шаг", self.tx, 
            Msg::Proc(Proc::ChainIsStarted { step_num, do_until_end: false }));
        self.btn_run.add_emit("Этот шаг и все шаги ниже", self.tx, 
            Msg::Proc(Proc::ChainIsStarted { step_num, do_until_end: true }));
        self.btn_edit.set_emit(self.tx, Msg::StepOp(StepOp::Edit { step_num }));
        self.btn_delete.set_emit(self.tx, Msg::StepOp(StepOp::Delete { step_num }));
        self.btn_reorder.add_emit("Сдвинуть вверх", self.tx, Msg::StepOp(StepOp::Move { step_num, direction: message::MoveStep::Up } ));
        self.btn_reorder.add_emit("Сдвинуть вниз", self.tx, Msg::StepOp(StepOp::Move { step_num, direction: message::MoveStep::Down } ));
        self.step_num = step_num;
    }

    pub fn set_buttons_active(&mut self, active: bool) {
        self.btn_run.set_active(active);
        self.btn_edit.set_active(active);
        self.btn_delete.set_active(active);
        self.btn_reorder.set_active(active);
    }


    pub fn get_data_copy(&self) -> Result<Img, MyError> {
        match self.img_presenter.image_copy() {
            Some(img) => Ok(img),
            None => Err(MyError::new("Шаг не содержит изображения".to_string())),
        }
    }

    pub fn has_image(&self) -> bool { self.img_presenter.has_image() }
    
    pub fn image_ref<'own>(&'own self) -> Option<&'own Img> { self.img_presenter.image_ref() }
    
    pub fn filter<'own>(&'own self) -> &'own FilterBase { &self.filter }
    pub fn filter_mut<'own>(&'own mut self) -> &'own mut FilterBase { &mut self.filter }


    pub fn start_processing(&mut self) {
        self.prog_bar.show();
        self.prog_bar.reset();
        self.img_presenter.clear_image(); 
    }

    pub fn display_progress(&mut self, percents: usize) {
        self.prog_bar.set_value(percents);
        self.img_presenter.clear_image(); 
    }

    pub fn display_result(&mut self, processed_img: Option<Img>) -> Result<(), MyError>  {
        self.prog_bar.hide();

                        
        match processed_img {
            Some(img) => self.img_presenter.set_image(img)?,
            None => self.img_presenter.clear_image(),
        }

        self.update_step_description();

        Ok(())
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