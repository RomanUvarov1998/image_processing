use fltk::{app::Sender, button, enums::Shortcut, frame, group::{self, PackType}, menu, prelude::{GroupExt, ImageExt, MenuExt, WidgetExt}};
use crate::{img::Img, message::{Message}, my_err::MyError};


pub const TEXT_PADDING: i32 = 10;


pub trait SizedWidget {
    fn w(&self) -> i32;
    fn h(&self) -> i32;
}


pub struct MyButton {
    btn: button::Button,
}

#[allow(unused)]
impl MyButton {
    pub fn new<'label, TMsg>(label: &'label str, sender: Sender<TMsg>, msg: TMsg) -> Self 
        where TMsg: 'static + Clone + Copy + Send + Sync
    {
        let mut my_button = Self::with_label(label);
        my_button.set_emit(sender, msg);

        my_button
    }

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

    pub fn widget<'inner>(&'inner self) -> &'inner button::Button { &self.btn }

    pub fn set_active(&mut self, active: bool) {
        if active { 
            self.btn.activate(); 
        } else {
            self.btn.deactivate();
        }
    }
}

impl SizedWidget for MyButton {
    fn w(&self) -> i32 { self.btn.w() }
    fn h(&self) -> i32 { self.btn.h() }
}


#[allow(unused)]
pub struct MyLabel {
    label: frame::Frame,
}

#[allow(unused)]
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

    pub fn widget<'inner>(&'inner self) -> &'inner frame::Frame { &self.label }
}

impl SizedWidget for MyLabel {
    fn w(&self) -> i32 { self.label.w() }
    fn h(&self) -> i32 { self.label.h() }
}


#[allow(unused)]
pub struct MyMenuBar {
    mb: menu::MenuBar
}

#[allow(unused)]
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

impl SizedWidget for MyMenuBar {
    fn w(&self) -> i32 { self.mb.w() }
    fn h(&self) -> i32 { self.mb.h() }
}


#[allow(unused)]
pub struct MyMenuButton<'label, TMsg> 
    where TMsg: 'static + Clone + Copy + Send + Sync
{
    btn: menu::MenuButton,
    emmits: Vec<(&'label str, Sender<TMsg>, TMsg)>
}

#[allow(unused)]
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

    pub fn widget<'inner>(&'inner self) -> &'inner menu::MenuButton { &self.btn }

    pub fn set_active(&mut self, active: bool) {
        if active { 
            self.btn.activate(); 
        } else {
            self.btn.deactivate();
        }
    }
}

impl<'label, TMsg> SizedWidget for MyMenuButton<'label, TMsg>
    where TMsg: 'static + Clone + Copy + Send + Sync
{
    fn w(&self) -> i32 { self.btn.w() }
    fn h(&self) -> i32 { self.btn.h() }
}


pub enum ImgPresenterContent {
    EmptyNoProcessing,
    Progress { percents: usize },
}

pub struct MyImgPresenter {
    frame_img: frame::Frame,
    img_drawable_processable: Option<Img>,
    w: i32, h: i32
}

impl MyImgPresenter {
    pub fn new(w: i32, h: i32) -> Self {
        let mut frame_img = frame::Frame::default()
            .with_size(w, h);
        frame_img.set_frame(fltk::enums::FrameType::EmbossedFrame);
        frame_img.set_align(fltk::enums::Align::Center); 

        MyImgPresenter {
            frame_img,
            img_drawable_processable: None,
            w, h
        }
    }

    pub fn set_state(&mut self, state: ImgPresenterContent) {
        match state {
            ImgPresenterContent::EmptyNoProcessing => {
                self.img_drawable_processable = None;
                self.frame_img.set_image(Option::<fltk::image::RgbImage>::None);
                self.frame_img.set_label("");
                self.frame_img.redraw_label();
                self.frame_img.redraw(); 
            },
            ImgPresenterContent::Progress { percents } => {
                if let Some(_) = self.img_drawable_processable {
                    self.img_drawable_processable = None;
                }
                if let Some(_) = self.frame_img.image() {
                    self.frame_img.set_image(Option::<fltk::image::RgbImage>::None);
                    self.frame_img.redraw(); 
                }
                self.frame_img.set_label(&format!("{}%", percents));
                self.frame_img.redraw_label();
            },
        }
    }

    pub fn set_image(&mut self, img: Img) -> Result<(), MyError> {
        self.frame_img.set_label("");
        self.frame_img.redraw_label();
        self.set_scaled_img(img, self.w(), self.h())?;
        self.frame_img.redraw(); 

        Ok(())
    }

    pub fn has_image(&self) -> bool { self.img_drawable_processable.is_some() }

    pub fn image<'own>(&'own self) -> Option<&'own Img> {
        match &self.img_drawable_processable {
            Some(ref processable) => Some(processable),
            None => None,
        }
    }

    pub fn resize(&mut self, new_w: i32, new_h: i32) -> Result<(), MyError> {
        if self.has_image() {
            self.set_scaled_img(self.image().unwrap().clone(), new_w, new_h)?;
            self.frame_img.set_size(new_w, new_h);
            self.frame_img.redraw(); 
        }

        Ok(())
    }
    
    fn set_scaled_img(&mut self, img: Img, w: i32, h: i32) -> Result<(), MyError> {
        pub const IMG_PADDING: i32 = 10;

        let mut scaled_drawable = img.get_drawable_copy()?;
        
        self.img_drawable_processable = Some(img);

        scaled_drawable.scale(w - IMG_PADDING, h - IMG_PADDING, 
            true, true);

        self.frame_img.set_image(Some(scaled_drawable));

        Ok(())
    }
}

impl SizedWidget for MyImgPresenter {
    fn w(&self) -> i32 { self.w }

    fn h(&self) -> i32 { self.h }
}




#[allow(unused)]
pub struct MyColumn {
    pack: group::Pack
}

#[allow(unused)]
impl MyColumn {
    pub fn new(w: i32, h: i32) -> Self {
        let mut pack = group::Pack::default()
            .with_size(w, h);
        pack.set_type(PackType::Vertical);
        const PADDING: i32 = 3;
        pack.set_spacing(PADDING);

        MyColumn { pack }
    }

    #[allow(unused)]
    pub fn with_pos(mut self, x: i32, y: i32) -> Self {
        self.pack.set_pos(x, y);
        self
    }

    pub fn end(&mut self) { self.pack.end(); }

    pub fn widget_mut<'own>(&'own mut self) -> &'own mut group::Pack { 
        &mut self.pack 
    }
}

impl SizedWidget for MyColumn {
    fn w(&self) -> i32 { self.pack.w() }
    fn h(&self) -> i32 { self.pack.h() }
}


#[allow(unused)]
pub struct MyRow {
    pack: group::Pack
}

#[allow(unused)]
impl MyRow {
    pub fn new(w: i32, h: i32) -> Self {
        let mut pack = group::Pack::default()
            .with_size(w, h);
        pack.set_type(PackType::Horizontal);
        const PADDING: i32 = 3;
        pack.set_spacing(PADDING);

        MyRow { pack }
    }

    pub fn with_pos(mut self, x: i32, y: i32) -> Self {
        self.pack.set_pos(x, y);
        self
    }

    pub fn end(&mut self) { self.pack.end(); }

    pub fn widget_mut<'own>(&'own mut self) -> &'own mut group::Pack { 
        &mut self.pack 
    }
}

impl SizedWidget for MyRow {
    fn w(&self) -> i32 { self.pack.w() }
    fn h(&self) -> i32 { self.pack.h() }
}