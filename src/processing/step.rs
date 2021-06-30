use fltk::{app::{self, Sender}, group, prelude::{GroupExt}};
use crate::{img::Img, message::{self, Message, Processing, StepOp}, my_component::{Alignable, container::{MyColumn, MyRow}, img_presenter::MyImgPresenter, usual::{MyButton, MyLabel, MyMenuButton, MyProgressBar}}, my_err::MyError};
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
    sender: Sender<Message>
}

impl ProcessingStep {
    pub fn new(w: i32, h: i32, step_num: usize, filter: FilterBase) -> Self {
        let name: String = filter.get_description();

        let mut main_column = MyColumn::new(w, 100);

        let label_step_name = MyLabel::new(&name);

        let (sender, _) = app::channel::<Message>();

        let mut btns_row = MyRow::new(w, 100);

        let btn_run = MyMenuButton::with_img_and_tooltip("run step.png", "Запустить");
        let btn_edit = MyButton::with_img_and_tooltip("edit step.png", "Изменить");
        let btn_delete = MyButton::with_img_and_tooltip("delete step.png", "Удалить");
        let btn_reorder = MyMenuButton::with_img_and_tooltip("reorder steps.png", "Переупорядочить");

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
            sender
        };

        step.update_btn_emits(step_num);

        step
    }


    pub fn auto_resize(&mut self, new_width: i32) {
        self.label_step_name.set_width(new_width);
        self.prog_bar.set_width(new_width);
        self.img_presenter.redraw();
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
        self.btn_run.add_emit("Только этот шаг", self.sender, 
            Message::Processing(Processing::StepsChainIsStarted { step_num, do_until_end: false }));
        self.btn_run.add_emit("Этот шаг и все шаги ниже", self.sender, 
            Message::Processing(Processing::StepsChainIsStarted { step_num, do_until_end: true }));
        self.btn_edit.set_emit(self.sender, Message::StepOp(StepOp::EditStep { step_num }));
        self.btn_delete.set_emit(self.sender, Message::StepOp(StepOp::DeleteStep { step_num }));
        self.btn_reorder.add_emit("Сдвинуть вверх", self.sender, Message::StepOp(StepOp::MoveStep { step_num, direction: message::MoveStep::Up } ));
        self.btn_reorder.add_emit("Сдвинуть вниз", self.sender, Message::StepOp(StepOp::MoveStep { step_num, direction: message::MoveStep::Down } ));
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