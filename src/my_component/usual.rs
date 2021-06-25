use fltk::{app::{Sender}, button, enums::Shortcut, frame, menu, misc, prelude::{ImageExt, MenuExt, WidgetBase, WidgetExt}};
use super::{Alignable, TEXT_PADDING};


fn set_img_and_tooltip<W: WidgetExt +  WidgetBase>(widget: &mut W, path: &str, tooltip: &str) {
    let bytes = crate::Asset::get(path).unwrap();
    let mut img = fltk::image::PngImage::from_data(&bytes[..]).unwrap();
    
    const IMG_PADDING: i32 = 5;
    
    widget.set_size(img.w() + IMG_PADDING * 2, img.h() + IMG_PADDING * 2);

    widget.draw(move |wid| {
        let (x, y, w, h) = 
            (wid.x() + IMG_PADDING, wid.y() + IMG_PADDING, 
            wid.w() - IMG_PADDING, wid.h() - IMG_PADDING);

        img.draw(x, y, w, h);
    });

    widget.set_tooltip(tooltip);
}


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

    pub fn with_img_and_tooltip(path: &str, tooltip: &str) -> Self {
        let mut btn = button::Button::default();

        set_img_and_tooltip(&mut btn, path, tooltip);

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

    pub fn widget<'own>(&'own mut self) -> &'own mut button::Button {
        &mut self.btn
    }
}

impl Alignable for MyButton {
    fn resize(&mut self, x: i32, y: i32, w: i32, h: i32) { self.btn.resize(x, y, w, h); }

    fn x(&self) -> i32 { self.btn.x() }

    fn y(&self) -> i32 { self.btn.y() }

    fn w(&self) -> i32 { self.btn.w() }

    fn h(&self) -> i32 { self.btn.h() }
}


#[derive(Clone)]
pub struct MyToggleButton {
    btn: button::ToggleButton,
}

#[allow(unused)]
impl MyToggleButton {
    pub fn with_label<'label>(label: &'label str) -> Self {
        let mut btn = button::ToggleButton::default();
        btn.set_label(label);

        let (w, h) = btn.measure_label();
        btn.set_size(w + TEXT_PADDING, h + TEXT_PADDING);

        MyToggleButton { btn }
    }

    pub fn with_img_and_tooltip(path: &str, tooltip: &str) -> Self {
        let mut btn = button::ToggleButton::default();

        let bytes = crate::Asset::get(path).unwrap();
        let mut img = fltk::image::PngImage::from_data(&bytes[..]).unwrap();
        
        const IMG_PADDING: i32 = 5;
        
        btn.set_size(img.w() + IMG_PADDING * 2, img.h() + IMG_PADDING * 2);

        btn.draw(move |b| {
            let (x, y, w, h) = 
                (b.x() + IMG_PADDING, b.y() + IMG_PADDING, 
                b.w() - IMG_PADDING, b.h() - IMG_PADDING);

            if b.is_toggled() {
                use fltk::{draw, enums::{Color}};
                const LINE_PADDING: i32 = 3;
                draw::draw_rect_fill(
                    x - LINE_PADDING, y - LINE_PADDING, 
                    img.w() + LINE_PADDING * 2, img.h() + LINE_PADDING * 2, 
                    Color::Blue);
            }

            img.draw(x, y, w, h);
        });

        btn.set_tooltip(tooltip);

        MyToggleButton { btn }
    }

    pub fn toggle(&mut self, value: bool) {
        self.btn.toggle(value);
    }

    pub fn is_toggled(&self) -> bool {
        self.btn.is_toggled()
    }

    pub fn set_active(&mut self, active: bool) {
        if active {
            self.btn.activate();
        } else {
            self.btn.deactivate();
        }
    }

    pub fn set_emit<TMsg>(&mut self, sender: Sender<TMsg>, msg: TMsg) 
        where TMsg: 'static + Clone + Copy + Send + Sync
    {
        self.btn.emit(sender, msg);
    }

    pub fn widget_mut<'own>(&'own mut self) -> &'own mut button::ToggleButton {
        &mut self.btn
    }

    pub fn widget<'own>(&'own self) -> &'own button::ToggleButton {
        &self.btn
    }
}

impl Alignable for MyToggleButton {
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
    pub fn new(w: i32) -> Self {
        MyMenuBar {
            mb: menu::MenuBar::default().with_size(w, 30)
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


pub struct MyMenuButton {
    btn: menu::MenuButton
}

impl MyMenuButton {
    pub fn with_img_and_tooltip(path: &str, tooltip: &str) -> Self {
        let mut btn = menu::MenuButton::default();

        set_img_and_tooltip(&mut btn, path, tooltip);

        btn.set_tooltip(tooltip);

        MyMenuButton { btn }
    }

    pub fn add_emit<'label, TMsg>(&mut self, label: &'label str, sender: Sender<TMsg>, msg: TMsg)
    where TMsg: 'static + Clone + Copy + Send + Sync
    {
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

impl Alignable for MyMenuButton {
    fn resize(&mut self, x: i32, y: i32, w: i32, h: i32) { self.btn.resize(x, y, w, h); }

    fn x(&self) -> i32 { self.btn.x() }

    fn y(&self) -> i32 { self.btn.y() }

    fn w(&self) -> i32 { self.btn.w() }

    fn h(&self) -> i32 { self.btn.h() }
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