use std::{ops::{Add, AddAssign, Neg, Sub, SubAssign}, sync::{Arc, Mutex}};

use fltk::{frame, prelude::{ImageExt, WidgetBase, WidgetExt}};
use crate::{img::{Img, pixel_pos::PixelPos}, my_component::{container::{MyColumn, MyRow}, usual::{MyButton, MyToggleButton}}, my_err::MyError};
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

        let mut btn_toggle_selection = MyToggleButton::with_label("Выделение");
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
            let (x, y) = (fltk::app::event_x() - f.x(), fltk::app::event_y() - f.y());

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
                    sender.send(ImgPresMsg::MouseDown (Pos::new(x, y))).unwrap_or(());
					true
                },
                Event::Released => {
                    was_mouse_down = false;
                    sender.send(ImgPresMsg::MouseUp).unwrap_or(());
					true
                },
                Event::MouseWheel => {
                    if was_mouse_down {
                        sender.send(ImgPresMsg::MouseScroll { factor_delta, mouse_x: x, mouse_y: y }).unwrap_or(());
						true
                    } else {
						false
                    }
                },
                Event::Drag => {
                    was_mouse_down = true;
                    sender.send(ImgPresMsg::MouseMove (Pos::new(x, y))).unwrap_or(());
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

    pub fn get_selection_rect(&self) -> Result<(PixelPos, PixelPos), MyError> {
        let rect_locked = self.img_pres_rect_arc.lock().unwrap();

        match rect_locked.as_ref() {
            Some(pres_rect) => {
                if let Some(ref sel_rect) = pres_rect.selection_rect {
                    let tl_x = (sel_rect.pos_top_left.x as f32 / pres_rect.scale_factor) as usize;
                    let tl_y = (sel_rect.pos_top_left.y as f32 / pres_rect.scale_factor) as usize;
                    let br_x = (sel_rect.pos_bottom_right.x as f32 / pres_rect.scale_factor) as usize;
                    let br_y = (sel_rect.pos_bottom_right.y as f32 / pres_rect.scale_factor) as usize;

                    Ok((PixelPos::new(tl_x, tl_y), PixelPos::new(br_x, br_y)))
                } else {
                    return Err(MyError::new("Необходимо установить режим выделения".to_string()));
                }
            },
            None => {
                return Err(MyError::new("Необходимо установить режим выделения".to_string()));
            },
        }
    }

    pub fn has_image(&self) -> bool { self.img.is_some() }

    pub fn image<'own>(&'own self) -> Option<&'own Img> {
        match &self.img {
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


struct ImgPresRect {
    im_pos: Pos,
    im_sz_initial: Size,
    scale_factor: f32,
    prev_pos: Option<Pos>,
    selection_rect: Option<SelectionRect>
}

impl ImgPresRect {
    fn new(img: &Img, frame: &frame::Frame) -> Self {
        let im_sz_initial = Size::new(img.w() as i32, img.h() as i32);
        let scale_factor = Self::scale_factor_to_fit(im_sz_initial, Size::of(frame));

        let mut rect = ImgPresRect { 
            im_pos: Pos::new(0, 0), 
            scale_factor,
            im_sz_initial,
            prev_pos: None,
            selection_rect: None
        };

        rect.correct_pos_scale(Size::of(frame));

		rect
    }


    fn consume_msg(&mut self, msg: ImgPresMsg, frame: &frame::Frame) {
        match msg {
            ImgPresMsg::MouseDown (pos) => self.start_drag(pos),
            ImgPresMsg::MouseMove (cur) => self.drag(cur),
            ImgPresMsg::MouseUp => {
                self.stop_drag();
                self.correct_pos_scale(Size::of(frame));
            },
            ImgPresMsg::MouseScroll { factor_delta, mouse_x, mouse_y } => {	
                self.scale(factor_delta, Pos::new(mouse_x, mouse_y), Size::of(frame));
            },
            ImgPresMsg::Fit => {
                let (delta, anchor) = if let Some(ref mut rect) = self.selection_rect {
                    let sf_mul = Self::scale_factor_to_fit(rect.size(), Size::of(frame));

                    ((sf_mul - 1.0) * self.scale_factor, rect.center())
                } else {
                    let new_sf = Self::scale_factor_to_fit(self.im_sz_initial, Size::of(frame));
                    
                    (new_sf - self.scale_factor, Pos::center_of(frame))
                };

                self.scale(delta, anchor, Size::of(frame));

                self.correct_pos_scale(Size::of(frame));
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
            rect.start_drag(pos.x, pos.y);
        }
    }

    fn drag(&mut self, to: Pos) {
        let prev = match self.prev_pos {
            Some(pos) => pos,
            None => { return; },
        };

        let delta = to - prev;
        self.prev_pos = Some(to);

        if let Some(ref mut rect) = self.selection_rect {
            rect.drag(delta);
        } else {
            self.im_pos += delta;
        }
    }

    fn stop_drag(&mut self) {
        self.prev_pos = None;

        if let Some(ref mut rect) = self.selection_rect {
            rect.stop_drag();
        }
    }


	fn correct_pos_scale(&mut self, frame_sz: Size) {
        // --------------------------- correct image scale --------------------------- 
		const MAX_FACTOR: f32 = 15.0_f32;
		const MIN_FACTOR: f32 = 0.01_f32;

        let minimal_to_fit = Self::scale_factor_to_fit(self.im_sz_initial, frame_sz);

        if self.scale_factor < minimal_to_fit {
            self.scale_factor = minimal_to_fit;
        }
        if self.scale_factor > MAX_FACTOR {
            self.scale_factor = MAX_FACTOR;
        }
        if self.scale_factor < MIN_FACTOR {
            self.scale_factor = MIN_FACTOR;
        }

        // --------------------------- correct image position --------------------------- 
		let (im_w, im_h) = self.im_size_scaled();

		// min left
		if self.im_pos.x + im_w < frame_sz.w { 
			self.im_pos.x = frame_sz.w - im_w; 
		}

		// max right
		if self.im_pos.x > 0 { 
			self.im_pos.x = 0;
		}

		// min top
		if self.im_pos.y + im_h < frame_sz.h { 
			self.im_pos.y = frame_sz.h - im_h; 
		}

		// max bottom
		if self.im_pos.y > 0 { 
			self.im_pos.y = 0;
		}

        // --------------------------- correct selection rect position --------------------------- 
        let (im_top_left, im_bottom_right) = {
            let (w, h) = self.im_size_scaled();
            (self.im_pos, Pos::new(w, h))
        };

        let left = std::cmp::max(0, im_top_left.x);
        let top = std::cmp::max(0, im_top_left.y);
        let right = std::cmp::min(frame_sz.w, im_bottom_right.x);
        let bottom = std::cmp::min(frame_sz.h, im_bottom_right.y);
        
        if let Some(ref mut rect) = self.selection_rect {
            if rect.pos_top_left.x < left {
                rect.pos_top_left.x = left;
            }

            if rect.pos_top_left.y < top {
                rect.pos_top_left.y = top;
            }

            if rect.pos_bottom_right.x > right {
                rect.pos_bottom_right.x = right;
            }

            if rect.pos_bottom_right.y > bottom {
                rect.pos_bottom_right.y = bottom;
            }
        }
	}

    fn scale_factor_to_fit(im_sz: Size, rect_sz: Size) -> f32 {
		let to_fit_horizontaly = rect_sz.w as f32 / im_sz.w as f32;
		let to_fit_vertically = rect_sz.h as f32 / im_sz.h as f32;

        if to_fit_vertically < to_fit_horizontaly {
            to_fit_vertically
        } else {
            to_fit_horizontaly
        }
    }

    fn scale(&mut self, delta: f32, anchor: Pos, frame_sz: Size) {
        let scale_factor_prev = self.scale_factor;
        
        self.scale_factor += delta;
        let scale_factor_min = Self::scale_factor_to_fit(self.im_sz_initial, frame_sz);
        if self.scale_factor < scale_factor_min {
            self.scale_factor = scale_factor_min;
        }

        let relative = anchor - self.im_pos;

        let c = self.scale_factor / scale_factor_prev - 1.0;

        let shift = relative.mul_f(c);
        
        self.im_pos -= shift;
    }


    fn im_size_scaled(&self) -> (i32, i32) /* w, h */ {
		(
			(self.im_sz_initial.w as f32 * self.scale_factor) as i32,
			(self.im_sz_initial.h as f32 * self.scale_factor) as i32,
		)
	}

    fn draw_img(&mut self, img: &mut fltk::image::RgbImage, f: &frame::Frame) {
		let (im_w, im_h) = self.im_size_scaled();
        img.scale(im_w, im_h, true, true);

        use fltk::draw;
        draw::push_clip(f.x(), f.y(), f.w(), f.h());
        
        img.draw(f.x() + self.im_pos.x, f.y() + self.im_pos.y, im_w, im_h);

        if let Some(ref rect) = self.selection_rect {
            rect.draw(f);
        }
        
        draw::pop_clip();
    }
}


#[derive(Clone, Copy, Debug)]
enum SelRectDragType {
    TopLeft, TopMiddle, TopRight,
    MiddleLeft, Middle, MiddleRight,
    BottomLeft, BottomMiddle, BottomRight,
}


#[derive(Debug)]
struct SelectionRect {
    pos_top_left: Pos,
    pos_bottom_right: Pos,
    drag_type: Option<SelRectDragType>
}

impl SelectionRect {
    fn middle_third_of(frame: &frame::Frame) -> Self {
        let w = frame.w() / 3;
        let h = frame.h() / 3;
        let x = w;
        let y = h;
        
        SelectionRect { 
            pos_top_left: Pos::new(x, y), 
            pos_bottom_right: Pos::new(x + w, y + h), 
            drag_type: None 
        }
    }

    fn x(&self) -> i32 { self.pos_top_left.x }
    fn y(&self) -> i32 { self.pos_top_left.y }
    fn w(&self) -> i32 { self.pos_bottom_right.x - self.pos_top_left.x }
    fn h(&self) -> i32 { self.pos_bottom_right.y - self.pos_top_left.y }
    fn size(&self) -> Size { Size::new(self.w(), self.h()) }
    fn center(&self) -> Pos { 
        Pos::new(
            (self.pos_bottom_right.x + self.pos_top_left.x ) / 2,
            (self.pos_bottom_right.y + self.pos_top_left.y ) / 2)
    }

    const RECT_SIDE: i32 = 10;

    fn draw(&self, frame: &frame::Frame) {
        use fltk::{draw, enums::Color};

        draw::draw_rect_with_color(
            frame.x() + self.x(), 
            frame.y() + self.y(), 
            self.w(), self.h(),
            Color::Blue);

        let draw_rect_around = |x: i32, y: i32| {

            let (rx, ry) = (frame.x() + x - Self::RECT_SIDE / 2, frame.y() + y - Self::RECT_SIDE / 2);

            draw::draw_rect_fill(rx, ry, Self::RECT_SIDE, Self::RECT_SIDE, Color::Red);
            draw::draw_rect_with_color(rx, ry, Self::RECT_SIDE, Self::RECT_SIDE, Color::Blue);
        };

        draw_rect_around(self.x(), self.y());
        draw_rect_around(self.x() + self.w() / 2, self.y());
        draw_rect_around(self.x() + self.w(), self.y());

        draw_rect_around(self.x() + self.w(), self.y() + self.h() / 2);
        draw_rect_around(self.x() + self.w() / 2, self.y() + self.h() / 2);
        draw_rect_around(self.x(), self.y() + self.h() / 2);

        draw_rect_around(self.x(), self.y() + self.h());
        draw_rect_around(self.x() + self.w() / 2, self.y() + self.h());
        draw_rect_around(self.x() + self.w(), self.y() + self.h());
    }

    fn start_drag(&mut self, px: i32, py: i32) {
        let fits_rect = |rcx: i32, rcy: i32| -> bool {
            px >= rcx - Self::RECT_SIDE 
            && px <= rcx + Self::RECT_SIDE
            && py >= rcy - Self::RECT_SIDE 
            && py <= rcy + Self::RECT_SIDE
        };

        self.drag_type = 
            if fits_rect(self.x(), self.y()) {
                Some(SelRectDragType::TopLeft)
            } else if fits_rect(self.x() + self.w() / 2, self.y()) {
                Some(SelRectDragType::TopMiddle)
            } else if fits_rect(self.x() + self.w(), self.y()) {
                Some(SelRectDragType::TopRight)
            } else if fits_rect(self.x(), self.y() + self.h() / 2) {
                Some(SelRectDragType::MiddleLeft)
            } else if fits_rect(self.x() + self.w() / 2, self.y() + self.h() / 2) {
                Some(SelRectDragType::Middle)
            } else if fits_rect(self.x() + self.w(), self.y() + self.h() / 2) {
                Some(SelRectDragType::MiddleRight)
            } else if fits_rect(self.x(), self.y() + self.h()) {
                Some(SelRectDragType::BottomLeft)
            } else if fits_rect(self.x() + self.w() / 2, self.y() + self.h()) {
                Some(SelRectDragType::BottomMiddle)
            } else if fits_rect(self.x() + self.w(), self.y() + self.h()) {
                Some(SelRectDragType::BottomRight)
            } else {
                None
            };
    }

    fn stop_drag(&mut self) {
        self.drag_type = None;
    }

    fn drag(&mut self, delta: Pos)  {
        if let Some(ref mut dt) = self.drag_type {
            match dt {
                SelRectDragType::TopLeft => {
                    self.pos_top_left += delta;
                },
                SelRectDragType::TopMiddle => {
                    self.pos_top_left.y += delta.y;
                },
                SelRectDragType::TopRight => {
                    self.pos_top_left.y += delta.y;
                    self.pos_bottom_right.x += delta.x;
                },
                SelRectDragType::MiddleLeft => {
                    self.pos_top_left.x += delta.x;
                },
                SelRectDragType::Middle => {
                    self.pos_top_left += delta;
                    self.pos_bottom_right += delta;
                },
                SelRectDragType::MiddleRight => {
                    self.pos_bottom_right.x += delta.x;
                },
                SelRectDragType::BottomLeft => {
                    self.pos_top_left.x += delta.x;
                    self.pos_bottom_right.y += delta.y;
                },
                SelRectDragType::BottomMiddle => {
                    self.pos_bottom_right.y += delta.y;
                },
                SelRectDragType::BottomRight => {
                    self.pos_bottom_right += delta;
                },
            }

            if self.pos_top_left.x > self.pos_bottom_right.x {
                std::mem::swap(&mut self.pos_top_left.x, &mut self.pos_bottom_right.x);
                *dt = match dt {
                    SelRectDragType::TopLeft => SelRectDragType::TopRight,
                    SelRectDragType::TopRight => SelRectDragType::TopLeft,
                    SelRectDragType::MiddleLeft => SelRectDragType::MiddleRight,
                    SelRectDragType::MiddleRight => SelRectDragType::MiddleLeft,
                    SelRectDragType::BottomLeft => SelRectDragType::BottomRight,
                    SelRectDragType::BottomRight => SelRectDragType::BottomLeft,
                    _ => *dt
                };
            }
            if self.pos_top_left.y > self.pos_bottom_right.y {
                std::mem::swap(&mut self.pos_top_left.y, &mut self.pos_bottom_right.y);
                *dt = match dt {
                    SelRectDragType::TopLeft => SelRectDragType::BottomLeft,
                    SelRectDragType::TopMiddle => SelRectDragType::BottomMiddle,
                    SelRectDragType::TopRight => SelRectDragType::BottomRight,
                    SelRectDragType::BottomLeft => SelRectDragType::TopLeft,
                    SelRectDragType::BottomMiddle => SelRectDragType::TopMiddle,
                    SelRectDragType::BottomRight => SelRectDragType::TopRight,
                    _ => *dt
                };
            }
        }
    }
}


#[derive(Clone, Copy, Debug)]
struct Pos { x: i32, y: i32 }

impl Pos {
	fn new(x: i32, y: i32) -> Self {
		Self { x, y }
	}

    fn mul_f(&self, val: f32) -> Self {
        Pos::new(
            (self.x as f32 * val) as i32,
            (self.y as f32 * val) as i32,
        )
    }

    fn center_of<W: WidgetExt>(wid: &W) -> Self {
        Pos {
            x: wid.x() + wid.w() / 2,
            y: wid.y() + wid.h() / 2
        }
    }
}

impl Sub for Pos {
    type Output = Pos;

    fn sub(self, rhs: Self) -> Self::Output {
        Pos::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl Add for Pos {
    type Output = Pos;

    fn add(self, rhs: Self) -> Self::Output {
        Pos::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl AddAssign for Pos {
    fn add_assign(&mut self, rhs: Self) {
		self.x += rhs.x;
		self.y += rhs.y;
    }
}

impl SubAssign for Pos {
    fn sub_assign(&mut self, rhs: Self) {
		self.x -= rhs.x;
		self.y -= rhs.y;
    }
}

impl Neg for Pos {
    type Output = Pos;

    fn neg(self) -> Self::Output {
        Self { x: -self.x, y: -self.y }
    }
}


#[derive(Clone, Copy, Debug)]
struct Size { w: i32, h: i32 }

impl Size {
	fn new(w: i32, h: i32) -> Self {
		Self { w, h }
	}

    fn of<W: WidgetExt>(wid: &W) -> Self {
		Self { w: wid.w(), h: wid.h() }
	}

    fn mul_f(&self, val: f32) -> Self {
        Size::new(
            (self.w as f32 * val) as i32,
            (self.h as f32 * val) as i32,
        )
    }
}

impl AddAssign for Size {
    fn add_assign(&mut self, rhs: Self) {
		self.w += rhs.w;
		self.h += rhs.h;
    }
}


#[derive(Clone, Copy, Debug)]
enum ImgPresMsg {
    MouseDown (Pos),
    MouseMove (Pos),
    MouseUp,
    MouseScroll { factor_delta: f32, mouse_x: i32, mouse_y: i32 },
    Fit,
    SeletionOn, SelectionOff
}

