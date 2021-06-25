use std::{sync::{Arc, Mutex}};
use fltk::{frame, prelude::{ImageExt, WidgetBase, WidgetExt}};
use crate::{img::{Img}, my_component::{container::{MyColumn, MyRow}, usual::{MyButton, MyToggleButton}}, my_err::MyError, utils::{DragPos, DraggableRect, Pos, RectArea, ScalableRect}};
use super::Alignable;


pub struct MyImgPresenter {
    btn_fit: MyButton,
    btn_toggle_selection: MyToggleButton,
    frame_img: frame::Frame,
    img_pres_rect_arc: Arc<Mutex<Option<ImgPresRect>>>,
    img: Option<Img>,
}

impl MyImgPresenter {
    pub fn new(w: i32, h: i32) -> Self {
        let mut column = MyColumn::new(w, h);

        let mut btns_row = MyRow::new(w, 100);

        let mut btn_fit = MyButton::with_label("Уместить");
        btn_fit.set_active(false);
        
        let mut btn_toggle_selection = MyToggleButton::with_img_and_tooltip("crop.png", "Брать выделенное");
        btn_toggle_selection.set_active(false);

        btns_row.resize(
            btns_row.x(), btns_row.y(), 
            btns_row.w(), 
            std::cmp::max(btn_fit.h(), btn_toggle_selection.h()));
        btns_row.end();

        let mut frame_img = frame::Frame::default()
            .with_size(w, h - btn_fit.h() - btn_toggle_selection.h());
        use fltk::enums::{FrameType, Align};
        frame_img.set_frame(FrameType::EmbossedBox);
        frame_img.set_align(Align::Center); 
        
        column.end();

        let img = None;

        let img_pres_rect_arc = Arc::new(Mutex::new(None));

        MyImgPresenter { btn_fit, btn_toggle_selection, frame_img, img, img_pres_rect_arc }
    }

    pub fn clear_image(&mut self) {
        self.img = None;

        self.btn_fit.set_active(false);
        self.btn_fit.widget().set_callback(move |_| { });

        self.btn_toggle_selection.set_active(false);
        self.btn_toggle_selection.widget_mut().set_callback(move |_| { });

        self.frame_img.handle(|_, _| { false });
        self.frame_img.draw(|_| {});
        self.frame_img.redraw(); 

        {
            let mut rect_locked = self.img_pres_rect_arc.lock().unwrap();
            rect_locked.take();
        }
    }

    pub fn set_image(&mut self, img: Img) -> Result<(), MyError> {
        // data to move into closure
        let (sender, receiver) = std::sync::mpsc::channel::<ImgPresMsg>();

        // ------------------------------------ frame drawing ----------------------------------------
        // data to move into closure
        let mut drawable = img.get_drawable_copy()?;

        {
            let mut mutex = self.img_pres_rect_arc.lock().unwrap();
            mutex.replace(ImgPresRect::new(&img, &mut self.frame_img));
        }

        let img_pres_rect_arc = self.img_pres_rect_arc.clone();

        self.frame_img.draw(move |f| {
            let mut rect_locked = img_pres_rect_arc.lock().unwrap();

            while let Ok(msg) = receiver.try_recv() {
                rect_locked.as_mut().unwrap().consume_msg(msg, f);
            }

			rect_locked.as_mut().unwrap().draw_img(&mut drawable, f);
        });

        // ------------------------------------ btn toggle selection ----------------------------------------
        // data to move into closure
        let sender_for_btn = sender.clone();
        let mut frame_copy = self.frame_img.clone();

        self.btn_toggle_selection.widget_mut().set_callback(move |btn| { 
            let msg = if btn.is_toggled() { ImgPresMsg::SeletionOn } else { ImgPresMsg::SelectionOff };
            sender_for_btn.send(msg).unwrap_or(());
            frame_copy.redraw();
        });
        self.btn_toggle_selection.set_active(true);

        // ------------------------------------ btn fit ----------------------------------------
        // data to move into closure
        let sender_for_btn = sender.clone();
        let mut frame_copy = self.frame_img.clone();  
        let mut btn_toggle_selection_copy = self.btn_toggle_selection.clone();  

        self.btn_fit.widget().set_callback(move |_| {
            sender_for_btn.send(ImgPresMsg::Fit).unwrap_or(());
            sender_for_btn.send(ImgPresMsg::SelectionOff).unwrap_or(());
            btn_toggle_selection_copy.toggle(false);
            frame_copy.redraw();
        });
        self.btn_fit.set_active(true);


        // ------------------------------------ frame handling ----------------------------------------
        // data to move into closure
		let mut was_mouse_down = false;
        self.frame_img.handle(move |f, ev| {
            let mouse_pos = {
                let (x, y) = (fltk::app::event_x(), fltk::app::event_y());
                Pos::new(x, y)
            };

            use fltk::app::MouseWheel;

            const SCROLL_DELTA: f32 = 0.2_f32;
            let factor_delta: f32 = match fltk::app::event_dy() {
                MouseWheel::None => 0_f32,
                MouseWheel::Down => SCROLL_DELTA,
                MouseWheel::Up => -SCROLL_DELTA,
                MouseWheel::Right | MouseWheel::Left => unreachable!("")
            };

            use fltk::enums::Event;
			let event_handled = match ev {
                Event::Push => {
                    was_mouse_down = true;
                    sender.send(ImgPresMsg::MouseDown (mouse_pos)).unwrap_or(());
					true
                },
                Event::Released => {
                    was_mouse_down = false;
                    sender.send(ImgPresMsg::MouseUp).unwrap_or(());
					true
                },
                Event::MouseWheel => {
                    if was_mouse_down {
                        sender.send(ImgPresMsg::MouseScroll { factor_delta, pos: mouse_pos }).unwrap_or(());
						true
                    } else {
						false
                    }
                },
                Event::Drag => {
                    was_mouse_down = true;
                    sender.send(ImgPresMsg::MouseMove (mouse_pos)).unwrap_or(());
                    true
                },
                _ => false
            };

			if event_handled {
            	f.redraw();
			}

            event_handled
        });

        self.img = Some(img);

        self.frame_img.redraw(); 

        Ok(())
    }

    pub fn has_image(&self) -> bool { self.img.is_some() }

    pub fn image_ref<'own>(&'own self) -> Option<&'own Img> { self.img.as_ref() }

    pub fn image_copy(&self) -> Option<Img> { 
        match self.img {
            Some(ref img) => {
                if self.btn_toggle_selection.is_toggled() {
                    let rect_locked = self.img_pres_rect_arc.lock().unwrap();

                    let scale_rect = rect_locked.as_ref()
                        .expect("Selection mode btn is ON but there is scale_rect is None");
                    let sel_rect = scale_rect.selection_rect.as_ref()
                        .expect("Selection mode btn is ON but there is sel_rect is None");

                    let tl = scale_rect.scale_rect.self_to_pixel(sel_rect.inner.tl());
                    let br = scale_rect.scale_rect.self_to_pixel(sel_rect.inner.br());

                    let cropped = img.croped_copy(tl.to_pixel_pos(), br.to_pixel_pos());

                    Some(cropped)
                } else {
                    Some(img.clone())
                }
            },
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


struct ImgPresRect {
    scale_rect: ScalableRect,
    prev_pos: Option<Pos>,
    selection_rect: Option<SelectionRect>
}

impl ImgPresRect {
    fn new(img: &Img, frame: &frame::Frame) -> Self {
        let mut rect = ScalableRect::new(0, 0, img.w() as i32, img.h() as i32);
        rect.stretch_self_to_area(RectArea::of_widget(frame));
        
        ImgPresRect { 
            scale_rect: rect,
            prev_pos: None,
            selection_rect: None
        }
    }


    fn consume_msg(&mut self, msg: ImgPresMsg, frame: &frame::Frame) {
        match msg {
            ImgPresMsg::MouseDown (pos) => self.start_drag(pos),
            ImgPresMsg::MouseMove (cur) => self.drag(cur),
            ImgPresMsg::MouseUp => self.stop_drag(frame),
            ImgPresMsg::MouseScroll { factor_delta, pos } => 
                self.scale(pos, factor_delta, frame),
            ImgPresMsg::Fit => {
                if let Some(ref sel_rect) = self.selection_rect {
                    self.scale_rect.zoom_area(
                        RectArea::of_draggable_rect(&sel_rect.inner),
                        Pos::center_of(frame));
                } else {
                    self.scale_rect.stretch_self_to_area(RectArea::of_widget(frame));
                }
            },
            ImgPresMsg::SeletionOn => {
                self.selection_rect = Some(SelectionRect::middle_third_of(frame));
            },
            ImgPresMsg::SelectionOff => {
                self.selection_rect = None;
            }
        }
    }


    fn start_drag(&mut self, pos: Pos) {
        self.prev_pos = Some(pos);

        if let Some(ref mut rect) = self.selection_rect {
            rect.start_drag(pos);
        }
    }

    fn drag(&mut self, to: Pos) {
        let prev = match self.prev_pos {
            Some(pos) => pos,
            None => { return; },
        };

        let delta = to - prev;
        self.prev_pos = Some(to);

        if let Some(ref mut sel_rect) = self.selection_rect {
            if !sel_rect.drag(delta)  {
                self.scale_rect.translate(delta);
            }
        } else {
            self.scale_rect.translate(delta);
        }
    }

    fn scale(&mut self, anchor: Pos, delta: f32, frame: &frame::Frame) {
        self.scale_rect.scale_keep_anchor_pos(delta, anchor);
        self.scale_rect.fit_scale(Pos::size_of(frame));
    }

    fn stop_drag(&mut self, frame: &frame::Frame) {
        self.prev_pos = None;

        self.scale_rect.fit_scale(Pos::size_of(frame));
        self.scale_rect.fit_pos(RectArea::of_widget(frame));

        if let Some(ref mut sel_rect) = self.selection_rect {
            sel_rect.stop_drag();
            sel_rect.inner.fit_inside(RectArea::of_widget(frame));
            sel_rect.inner.fit_inside(self.scale_rect.area_scaled());
        }
    }


    fn draw_img(&mut self, img: &mut fltk::image::RgbImage, f: &frame::Frame) {
		let (im_w, im_h) = (self.scale_rect.scaled_w(), self.scale_rect.scaled_h());
        img.scale(im_w, im_h, true, true);

        use fltk::draw;
        draw::push_clip(f.x(), f.y(), f.w(), f.h());
        
        let im_pos = self.scale_rect.tl();
        img.draw(im_pos.x, im_pos.y, im_w, im_h);

        if let Some(ref rect) = self.selection_rect {
            rect.draw();
        }
        
        draw::pop_clip();
    }
}


#[derive(Debug)]
struct SelectionRect {
    inner: DraggableRect,
    drag_pos: Option<DragPos>
}

impl SelectionRect {
    fn middle_third_of(frame: &frame::Frame) -> Self {
        let w = frame.x() + frame.w() / 3;
        let h = frame.y() + frame.h() / 3;
        let x = w;
        let y = h;
        
        SelectionRect { 
            inner: DraggableRect::new(x, y, w, h) ,
            drag_pos: None 
        }
    }

    fn x(&self) -> i32 { self.inner.x() }
    fn y(&self) -> i32 { self.inner.y() }
    fn w(&self) -> i32 { self.inner.w() }
    fn h(&self) -> i32 { self.inner.h() }

    const RECT_SIDE: i32 = 10;

    fn draw(&self) {
        use fltk::{draw, enums::Color};

        draw::draw_rect_with_color(
            self.x(), self.y(), 
            self.w(), self.h(),
            Color::Blue);

        let draw_rect_around = |x: i32, y: i32, fill_color: Color| {
            let (rx, ry) = (x - Self::RECT_SIDE / 2, y - Self::RECT_SIDE / 2);

            draw::draw_rect_fill(
                rx, ry, 
                Self::RECT_SIDE, Self::RECT_SIDE, 
                fill_color);
            draw::draw_rect_with_color(
                rx, ry, 
                Self::RECT_SIDE, Self::RECT_SIDE, 
                Color::Blue);
        };

        let w_half = self.w() / 2;
        let h_half = self.h() / 2;

        for x_step in 0..3 {
            for y_step in 0..3 {
                let fill_color: Color = 
                    if let Some(dp) = self.drag_pos {
                        if dp == DragPos::from(x_step, y_step) {
                            Color::Green 
                        } else {
                            Color::Red
                        }
                    } else {
                        Color::Red
                    };

                draw_rect_around(
                    self.x() + w_half * x_step, 
                    self.y() + h_half * y_step,
                    fill_color);
            }
        }
    }

    fn start_drag(&mut self, pos: Pos) {
        let w_half = self.w() / 2;
        let h_half = self.h() / 2;

        let fits_rect = |rcx: i32, rcy: i32, p: Pos| -> bool {
            p.x >= rcx - Self::RECT_SIDE 
            && p.x <= rcx + Self::RECT_SIDE
            && p.y >= rcy - Self::RECT_SIDE 
            && p.y <= rcy + Self::RECT_SIDE
        };

        self.drag_pos = None;
        'out: for x_step in 0..3 {
            for y_step in 0..3 {
                if fits_rect(self.x() + w_half * x_step, self.y() + h_half * y_step, pos) {
                    self.drag_pos = Some(DragPos::from(x_step, y_step));
                    break 'out;
                }
            }
        }
    }

    fn stop_drag(&mut self) {
        self.drag_pos = None;
    }

    fn drag(&mut self, delta: Pos) -> bool  {
        if let Some(ref mut dt) = self.drag_pos {
            *dt = self.inner.drag(delta, *dt);
            return true;
        }
        return false;
    }
}


#[derive(Clone, Copy, Debug)]
enum ImgPresMsg {
    MouseDown (Pos),
    MouseMove (Pos),
    MouseUp,
    MouseScroll { factor_delta: f32, pos: Pos },
    Fit,
    SeletionOn, SelectionOff
}

