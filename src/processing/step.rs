use fltk::{app::{self, Sender}, group, prelude::GroupExt};

use crate::{img::Img, message::{self, Message, Processing, StepOp}, my_component::{Alignable, container::{MyColumn}, usual::{MyImgPresenter, MyLabel, MyMenuBar, MyProgressBar}}, my_err::MyError};

use super::{PADDING, StepAction, step_editor::StepEditor};

pub struct ProcessingStep {
    main_column: MyColumn,
    menu: MyMenuBar,
    label_step_name: MyLabel,
    prog_bar: MyProgressBar,
    img_presenter: MyImgPresenter,
    action: StepAction,
    step_num: usize,
    sender: Sender<Message>
}

impl ProcessingStep {
    pub fn new(w: i32, h: i32, step_num: usize, action: StepAction) -> Self {
        let name: String = action.filter_description();

        let mut main_column = MyColumn::new(w, 100);

        let label_step_name = MyLabel::new(&name);

        let (sender, _) = app::channel::<Message>();

        let menu = MyMenuBar::new(main_column.widget());

        let mut prog_bar = MyProgressBar::new(w - PADDING, 30);
        prog_bar.hide();
            
        let img_presenter = MyImgPresenter::new(
            w - PADDING, h - menu.h() * 2);
        
        main_column.end();
        
        let mut step = ProcessingStep { 
            main_column,
            menu,
            label_step_name,
            prog_bar,
            img_presenter, 
            action,
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

    pub fn edit_action_with_dlg(&mut self, app: app::App, step_editor: &mut StepEditor) {
        self.action = self.action.edit_with_dlg(app, step_editor);
        
        let filter_description: String = self.action.filter_description();

        let img_description: String = match self.img_presenter.image() {
            Some(img) => img.get_description(),
            None => String::new(),
        };

        self.label_step_name.set_text(&format!("{} {}", &filter_description, &img_description));
    }

    pub fn update_btn_emits(&mut self, step_num: usize) {
        self.menu.add_emit("Запустить/Только этот шаг", self.sender, 
            Message::Processing(Processing::StepsChainIsStarted { step_num, do_until_end: false }));
        self.menu.add_emit("Запустить/Этот шаг и все шаги ниже", self.sender, 
            Message::Processing(Processing::StepsChainIsStarted { step_num, do_until_end: true }));
        self.menu.add_emit("Изменить", self.sender, Message::StepOp(StepOp::EditStep { step_num }));
        self.menu.add_emit("Удалить", self.sender, Message::StepOp(StepOp::DeleteStep { step_num }));
        self.menu.add_emit("Переупорядочить/Сдвинуть вверх", self.sender, Message::StepOp(StepOp::MoveStep { step_num, direction: message::MoveStep::Up } ));
        self.menu.add_emit("Переупорядочить/Сдвинуть вниз", self.sender, Message::StepOp(StepOp::MoveStep { step_num, direction: message::MoveStep::Down } ));
        self.step_num = step_num;
    }

    pub fn set_buttons_active(&mut self, active: bool) {
        self.menu.set_active(active);
    }


    pub fn get_data_copy(&self) -> Option<Img> {
        match self.img_presenter.image() {
            Some(img_ref) => Some(img_ref.clone()),
            None => None,
        }
    }
 
    pub fn action<'own>(&'own self) -> &'own StepAction { &self.action } 

    pub fn has_image(&self) -> bool { self.img_presenter.has_image() }
    
    pub fn image<'own>(&'own self) -> Option<&'own Img> { self.img_presenter.image() }


    pub fn start_processing(&mut self) {
        self.prog_bar.show();
        self.prog_bar.reset();
        self.img_presenter.clear_image(); 
    }

    pub fn display_progress(&mut self, percents: usize) {
        self.prog_bar.set_value(percents);
        self.img_presenter.clear_image(); 
    }

    pub fn display_result(&mut self, img: Img) -> Result<(), MyError>  {
        self.prog_bar.hide();

        self.label_step_name.set_text(&format!("{} {}", self.action.filter_description(), img.get_description()));
                        
        self.img_presenter.set_image(img)?;

        Ok(())
    }
}