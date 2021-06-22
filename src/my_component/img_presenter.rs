use std::{ops::{AddAssign, Sub}};

use fltk::{frame, prelude::{ImageExt, WidgetBase, WidgetExt}};
use crate::{img::Img, my_err::MyError};
use super::Alignable;


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
        self.frame_img.handle(|_, _| { false });
        self.frame_img.draw(|_| {});
        self.frame_img.redraw(); 
    }

    pub fn set_image(&mut self, img: Img) -> Result<(), MyError> {
        // data to move into closure
        let (sender, receiver) = std::sync::mpsc::channel::<ImgPresMsg>();

        // data to move into closure
        let mut drawable = img.get_drawable_copy()?;
        let mut img_pres_rect = ImgPresRect::new(&img, &self.frame_img);

        self.frame_img.draw(move |f| {
            while let Ok(msg) = receiver.try_recv() {
                img_pres_rect.consume_msg(msg, f);
            }

			img_pres_rect.scale_draw(&mut drawable, f);
        });

        // data to move into closure
		let mut was_mouse_down = false;
        self.frame_img.handle(move |f, ev| {
            let (x, y) = (fltk::app::event_x() - f.x(), fltk::app::event_y() - f.y());

            use fltk::app::MouseWheel;

            const SCROLL_DELTA: i32 = 20;
            let delta_percents = match fltk::app::event_dy() {
                MouseWheel::None => 0,
                MouseWheel::Down => SCROLL_DELTA,
                MouseWheel::Up => -SCROLL_DELTA,
                MouseWheel::Right | MouseWheel::Left => unreachable!("")
            };

            use fltk::enums::Event;
			let event_handled = match ev {
                Event::Enter => {
                    sender.send(ImgPresMsg::MouseEnter).unwrap_or(());
					true
                },
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
                Event::Leave => {
                    was_mouse_down = false;
                    sender.send(ImgPresMsg::MouseLeave).unwrap_or(());
					true
                },
                Event::MouseWheel => {
                    if was_mouse_down {
                        sender.send(ImgPresMsg::MouseScroll { delta_percents }).unwrap_or(());
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


#[derive(Clone, Copy, Debug)]
struct ImgPresRect {
    im_pos: Pos,
    im_sz_initial: Size,
    scale_factor_percents: i32,
    prev_pos: Option<Pos>
}

impl ImgPresRect {
    fn new(img: &Img, frame: &frame::Frame) -> Self {
        let mut rect = ImgPresRect { 
            im_pos: Pos::new(0, 0), 
            scale_factor_percents: 100,
            im_sz_initial: Size::new(img.w() as i32, img.h() as i32),
            prev_pos: None
        };

		rect.correct_pos(frame);
		rect.correct_scale(frame);

		rect
    }

    fn consume_msg(&mut self, msg: ImgPresMsg, frame: &frame::Frame) {
        match msg {
            ImgPresMsg::MouseEnter => {},
            ImgPresMsg::MouseDown (pos) => {
                self.prev_pos = Some(pos);
            },
            ImgPresMsg::MouseMove (cur) => {
                if let Some(prev) = self.prev_pos {
					let delta = cur - prev;
					self.prev_pos = Some(cur);

					self.im_pos += delta;
					
					self.correct_pos(frame);
                }
            },
            ImgPresMsg::MouseUp => {
                self.prev_pos = None;
            },
            ImgPresMsg::MouseLeave => {
                self.prev_pos = None;
            },
            ImgPresMsg::MouseScroll { delta_percents } => {	
				self.scale_factor_percents += delta_percents;

				self.correct_scale(frame);
                self.correct_pos(frame);
            },
        }
    }

	fn correct_pos(&mut self, frame: &frame::Frame) {
		let (im_w, im_h) = self.im_size_scaled();

		// min left
		if self.im_pos.x + im_w < frame.w() { 
			self.im_pos.x = frame.w() - im_w; 
		}

		// max right
		if self.im_pos.x > 0 { 
			self.im_pos.x = 0;
		}

		// min top
		if self.im_pos.y + im_h < frame.h() { 
			self.im_pos.y = frame.h() - im_h; 
		}

		// max bottom
		if self.im_pos.y > 0 { 
			self.im_pos.y = 0;
		}
	}

	fn correct_scale(&mut self, frame: &frame::Frame) {
		const MAX_PERCENTS: i32 = 1500;
		const MIN_PERCENTS: i32 = 1;

		let percents_to_fit_horizontaly = frame.w() * 100 / self.im_sz_initial.w;
		let percents_to_fit_vertically = frame.h() * 100 / self.im_sz_initial.h;

		let min_percents = std::cmp::min(percents_to_fit_horizontaly, percents_to_fit_vertically);

		self.scale_factor_percents = std::cmp::max(
			min_percents,
			self.scale_factor_percents);

		self.scale_factor_percents = std::cmp::max(
			MIN_PERCENTS,
			self.scale_factor_percents);
			
		self.scale_factor_percents = std::cmp::min(
			MAX_PERCENTS,
			self.scale_factor_percents);
	}

	fn im_size_scaled(&self) -> (i32, i32) /* w, h */ {
		(
			self.im_sz_initial.w * self.scale_factor_percents / 100,
			self.im_sz_initial.h * self.scale_factor_percents / 100,
		)
	}

    fn scale_draw(&mut self, img: &mut fltk::image::RgbImage, f: &frame::Frame) {
		let (im_w, im_h) = self.im_size_scaled();
        img.scale(im_w, im_h, true, true);

        use fltk::draw;
        draw::push_clip(f.x(), f.y(), f.w(), f.h());
        img.draw(f.x() + self.im_pos.x, f.y() + self.im_pos.y, im_w, im_h);
        draw::pop_clip();
    }
}


#[derive(Clone, Copy, Debug)]
struct Pos { x: i32, y: i32 }

impl Pos {
	fn new(x: i32, y: i32) -> Self {
		Self { x, y }
	}
}

impl Sub for Pos {
    type Output = Pos;

    fn sub(self, rhs: Self) -> Self::Output {
        Pos::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl AddAssign for Pos {
    fn add_assign(&mut self, rhs: Self) {
		self.x += rhs.x;
		self.y += rhs.y;
    }
}


#[derive(Clone, Copy, Debug)]
struct Size { w: i32, h: i32 }

impl Size {
	fn new(w: i32, h: i32) -> Self {
		Self { w, h }
	}
}

impl Into<Pos> for Size {
    fn into(self) -> Pos {
        Pos { x: self.w, y: self.h }
    }
}


#[derive(Clone, Copy, Debug)]
enum ImgPresMsg {
    MouseEnter,
    MouseDown (Pos),
    MouseMove (Pos),
    MouseUp,
    MouseLeave,
    MouseScroll { delta_percents: i32 },
}

