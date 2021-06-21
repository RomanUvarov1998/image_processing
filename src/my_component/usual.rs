use std::{borrow::{Borrow}};
use fltk::{app::{self, Sender}, button, enums::Shortcut, frame, menu, misc, prelude::{ImageExt, MenuExt, WidgetBase, WidgetExt}};
use crate::{img::Img, my_err::MyError};
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


#[derive(Clone, Copy, Debug)]
enum ImgPresMsg {
    MouseEnter,
    MouseDown { x: i32, y: i32 },
    MouseMove { x: i32, y: i32 },
    MouseUp,
    MouseLeave,
    MouseScroll { delta: f32 },
}

#[derive(Clone, Copy, Debug)]
struct ImgPresRect {
    pos: (i32, i32),
    scale: (f32, f32),
    img_sz: (i32, i32),
    prev_pos: Option<(i32, i32)>
}
impl ImgPresRect {
    fn new(img: &Img) -> Self {
        ImgPresRect { 
            pos: (0, 0), 
            scale: (1_f32, 1_f32),
            img_sz: (img.w() as i32, img.h() as i32),
            prev_pos: None
        }
    }

    fn consume_msg(&mut self, msg: ImgPresMsg) {
        match msg {
            ImgPresMsg::MouseEnter => {},
            ImgPresMsg::MouseDown { x, y } => {
                self.prev_pos = Some((x, y));
            },
            ImgPresMsg::MouseMove { x, y } => {
                if let Some((prev_x, prev_y)) = self.prev_pos {
                    let dx = x - prev_x;
                    let dy = y - prev_y;

                    self.pos.0 += dx;
                    self.pos.1 += dy;

                    self.prev_pos = Some((x, y));
                }
            },
            ImgPresMsg::MouseUp => {
                self.prev_pos = None;
            },
            ImgPresMsg::MouseLeave => {
                self.prev_pos = None;
            },
            ImgPresMsg::MouseScroll { delta } => {
                self.scale.0 += delta;
                self.scale.1 += delta;
            },
        }
    }

    const IMG_PADDING: i32 = 10;

    fn scale_draw(&mut self, img: &mut fltk::image::RgbImage, f: &frame::Frame) {
        let w = (self.scale.0 * self.img_sz.0 as f32) as i32;
        let h = (self.scale.1 * self.img_sz.1 as f32) as i32;

        img.scale(w, h, true, true);

        let x = f.x() + Self::IMG_PADDING + self.pos.0;
        let y = f.y() + Self::IMG_PADDING + self.pos.1;
        
        let right = f.x() + f.w();
        let bottom = f.y() + f.h();
        
        let w = right - x - Self::IMG_PADDING * 2;
        let h = bottom - y - Self::IMG_PADDING * 2;

        img.draw(x, y, w, h);
    }
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

        MyImgPresenter { frame_img, img }
    }

    pub fn clear_image(&mut self) {
        self.img = None;
        self.frame_img.draw(|_| {});
        self.frame_img.redraw(); 
    }

    pub fn set_image(&mut self, img: Img) -> Result<(), MyError> {
        // data to move into closure
        let (sender, receiver) = std::sync::mpsc::channel::<ImgPresMsg>();
        let mut was_mouse_down = false;

        self.frame_img.handle(move |f, ev| {
            let (x, y) = fltk::app::event_coords();

            use app::MouseWheel;

            const SCROLL_DELTA: f32 = 0.2_f32;
            let delta = match fltk::app::event_dy() {
                MouseWheel::None => 0_f32,
                MouseWheel::Down => SCROLL_DELTA,
                MouseWheel::Up => -SCROLL_DELTA,
                MouseWheel::Right | MouseWheel::Left => unreachable!("")
            };

            use fltk::enums::Event;
            let event_handled = match ev {
                Event::Enter => {
                    sender.send(ImgPresMsg::MouseEnter).unwrap();
                    true
                },
                Event::Push => {
                    was_mouse_down = true;
                    sender.send(ImgPresMsg::MouseDown { x, y }).unwrap();
                    true
                },
                Event::Released => {
                    was_mouse_down = false;
                    sender.send(ImgPresMsg::MouseUp).unwrap();
                    true
                },
                Event::Leave => {
                    was_mouse_down = false;
                    sender.send(ImgPresMsg::MouseLeave).unwrap();
                    true
                },
                Event::MouseWheel => {
                    if was_mouse_down {
                        sender.send(ImgPresMsg::MouseScroll { delta }).unwrap();
                        true
                    } else {
                        false
                    }
                },
                Event::Drag => {
                    was_mouse_down = true;
                    sender.send(ImgPresMsg::MouseMove { x, y }).unwrap();
                    true
                },
                _ => return false
            };

            f.redraw();

            event_handled
        });

        // data to move into closure
        let mut drawable = img.get_drawable_copy()?;
        let mut img_pres_rect = ImgPresRect::new(&img);

        self.frame_img.draw(move |f| {
            while let Ok(msg) = receiver.try_recv() {
                img_pres_rect.consume_msg(msg);
            }

            img_pres_rect.scale_draw(&mut drawable, f);
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