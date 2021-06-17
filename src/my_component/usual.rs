use std::{borrow::{Borrow}};
use fltk::{app::{Sender}, button, enums::Shortcut, frame, menu, misc, prelude::{ImageExt, MenuExt, WidgetBase, WidgetExt}};
use crate::{img::Img, message::{Message}, my_err::MyError};
use super::{Alignable, TEXT_PADDING};


pub struct MyButton {
    btn: button::Button,
}

impl MyButton {
    pub fn with_label<'label>(label: &'label str) -> Self {
        let mut btn = button::Button::default();
        btn.set_label(label);

        let (w, h) = btn.measure_label();
        btn.set_size(w + TEXT_PADDING, h + TEXT_PADDING);

        MyButton { btn }
    }

    pub fn set_emit<TMsg>(&mut self, sender: Sender<TMsg>, msg: TMsg) 
        where TMsg: 'static + Clone + Copy + Send + Sync
    {
        self.btn.emit(sender, msg);
    }

    pub fn set_active(&mut self, active: bool) {
        if active { 
            self.btn.activate(); 
        } else {
            self.btn.deactivate();
        }
    }
}

impl Alignable for MyButton {
    fn resize(&mut self, x: i32, y: i32, w: i32, h: i32) { self.btn.resize(x, y, w, h); }

    fn x(&self) -> i32 { self.btn.x() }

    fn y(&self) -> i32 { self.btn.y() }

    fn w(&self) -> i32 { self.btn.w() }

    fn h(&self) -> i32 { self.btn.h() }
}


pub struct MyLabel {
    label: frame::Frame,
}

impl MyLabel {
    pub fn new<'text>(text: &'text str) -> Self {
        let mut label = frame::Frame::default();
        label.set_label(text);
        
        let (w, h) = label.measure_label();
        label.set_size(w + TEXT_PADDING, h + TEXT_PADDING);
        
        MyLabel { label }
    }

    pub fn set_text<'text>(&mut self, text: &'text str) {
        self.label.set_label(text);
        self.label.redraw_label();
    }

    pub fn set_width(&mut self, new_w: i32) {
        self.label.set_size(new_w, self.label.h());
    }
}

impl Alignable for MyLabel {
    fn resize(&mut self, x: i32, y: i32, w: i32, h: i32) { self.label.resize(x, y, w, h); }

    fn x(&self) -> i32 { self.label.x() }

    fn y(&self) -> i32 { self.label.y() }

    fn w(&self) -> i32 { self.label.w() }

    fn h(&self) -> i32 { self.label.h() }
}


pub struct MyMenuBar {
    mb: menu::MenuBar
}

impl MyMenuBar {
    pub fn new<P: WidgetExt>(parent: &P) -> Self {
        MyMenuBar {
            mb: menu::MenuBar::default().with_size(parent.w(), 30)
        }
    }

    pub fn add_emit<'label, TMsg>(&mut self, label: &'label str, sender: Sender<TMsg>, msg: TMsg)
        where TMsg: 'static + Clone + Copy + Send + Sync
    {
        self.mb.add_emit(label, Shortcut::None, menu::MenuFlag::Normal, sender, msg);
    }

    pub fn end(&mut self) { 
        self.mb.end(); 
    }

    pub fn set_active(&mut self, active: bool) {
        if active { 
            self.mb.activate(); 
        } else {
            self.mb.deactivate();
        }
    }
}

impl Alignable for MyMenuBar {
    fn resize(&mut self, x: i32, y: i32, w: i32, h: i32) { self.mb.resize(x, y, w, h); }

    fn x(&self) -> i32 { self.mb.x() }

    fn y(&self) -> i32 { self.mb.y() }

    fn w(&self) -> i32 { self.mb.w() }

    fn h(&self) -> i32 { self.mb.h() }
}


pub struct MyMenuButton<'label, TMsg> 
    where TMsg: 'static + Clone + Copy + Send + Sync
{
    btn: menu::MenuButton,
    emmits: Vec<(&'label str, Sender<TMsg>, TMsg)>
}

impl<'label> MyMenuButton<'label, Message> {
    pub fn new(label: &'label str) -> Self {
        let mut btn = menu::MenuButton::default();

        btn.set_label(label);

        let (w, h) = btn.measure_label();
        const MENU_BTN_ARROW_W: i32 = 30;
        btn.set_size(w + TEXT_PADDING + MENU_BTN_ARROW_W, h + TEXT_PADDING);

        let emmits = Vec::<(&'label str, Sender<Message>, Message)>::new();

        MyMenuButton::<Message>{ btn, emmits }
    }

    pub fn add_emit(&mut self, label: &'label str, sender: Sender<Message>, msg: Message) {
        self.emmits.push((label, sender, msg));
        self.btn.add_emit(label, Shortcut::None, menu::MenuFlag::Normal, sender, msg);
    }

    pub fn set_active(&mut self, active: bool) {
        if active { 
            self.btn.activate(); 
        } else {
            self.btn.deactivate();
        }
    }
}

impl<'label, TMsg> Alignable for MyMenuButton<'label, TMsg>
    where TMsg: 'static + Clone + Copy + Send + Sync
{
    fn resize(&mut self, x: i32, y: i32, w: i32, h: i32) { self.btn.resize(x, y, w, h); }

    fn x(&self) -> i32 { self.btn.x() }

    fn y(&self) -> i32 { self.btn.y() }

    fn w(&self) -> i32 { self.btn.w() }

    fn h(&self) -> i32 { self.btn.h() }
}



pub struct MyImgPresenter {
    frame_img: frame::Frame,
    img: Option<Img>,
}

impl MyImgPresenter {
    pub fn new(w: i32, h: i32) -> Self {
        let mut frame_img = frame::Frame::default()
            .with_size(w, h);

        use fltk::enums::{FrameType, Align};

        frame_img.set_frame(FrameType::EmbossedBox);
        frame_img.set_align(Align::Center); 

        let img = None;

        MyImgPresenter {
            frame_img,
            img,
        }
    }

    pub fn clear_image(&mut self) {
        self.img = None;
        self.frame_img.draw(|_| {});
        self.frame_img.redraw(); 
    }

    pub fn set_image(&mut self, img: Img) -> Result<(), MyError> {
        let mut drawable = img.get_drawable_copy()?;

        self.frame_img.draw(move |f| {
            const IMG_PADDING: i32 = 10;

            let x = f.x() + IMG_PADDING;
            let y = f.y() + IMG_PADDING;
            let w = f.w() - IMG_PADDING * 2;
            let h = f.h() - IMG_PADDING * 2;

            drawable.scale(w, h, true, true);
            drawable.draw(x, y, w, h);
        });

        self.img = Some(img);

        self.frame_img.redraw(); 

        Ok(())
    }

    pub fn has_image(&self) -> bool { self.img.is_some() }

    pub fn image<'own>(&'own self) -> Option<&'own Img> {
        match &self.img.borrow() {
            Some(ref img) => Some(img),
            None => None,
        }
    }

    pub fn redraw(&mut self) { self.frame_img.redraw(); }
}

impl Alignable for MyImgPresenter {
    fn resize(&mut self, x: i32, y: i32, w: i32, h: i32) { self.frame_img.resize(x, y, w, h); }

    fn x(&self) -> i32 { self.frame_img.x() }

    fn y(&self) -> i32 { self.frame_img.y() }

    fn w(&self) -> i32 { self.frame_img.w() }

    fn h(&self) -> i32 { self.frame_img.h() }
}



pub struct MyProgressBar {
    bar: misc::Progress
}

impl MyProgressBar {
    pub fn new(w: i32, h: i32) -> Self {
        let mut bar = misc::Progress::default()
            .with_size(w, h);
        bar.set_minimum(0_f64);
        bar.set_maximum(100_f64);
        bar.set_selection_color(fltk::enums::Color::Green);

        MyProgressBar { bar }
    }

    pub fn set_width(&mut self, new_w: i32) {
        self.bar.set_size(new_w, self.bar.h());
    }

    pub fn reset(&mut self) {
        self.set_value(0);
    }

    pub fn set_value(&mut self, progress_percents: usize) {
        self.bar.set_value(progress_percents as f64);
        self.bar.set_label(&format!("{}%", progress_percents));
    }

    pub fn show(&mut self) { self.bar.show(); }
    pub fn hide(&mut self) { self.bar.hide(); }
}

impl Alignable for MyProgressBar {
    fn resize(&mut self, x: i32, y: i32, w: i32, h: i32) { self.bar.resize(x, y, w, h); }

    fn x(&self) -> i32 { self.bar.x() }

    fn y(&self) -> i32 { self.bar.y() }

    fn w(&self) -> i32 { self.bar.w() }

    fn h(&self) -> i32 { self.bar.h() }
}