use std::ops::{Add, AddAssign, Sub};

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
        self.frame_img.draw(|_| {});
        self.frame_img.redraw(); 
    }

    pub fn set_image(&mut self, img: Img) -> Result<(), MyError> {
        // data to move into closure
        let (sender, receiver) = std::sync::mpsc::channel::<ImgPresMsg>();
        let mut was_mouse_down = false;

        self.frame_img.handle(move |f, ev| {
            let (x, y) = fltk::app::event_coords();

            use fltk::app::MouseWheel;

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
                    sender.send(ImgPresMsg::MouseDown (Pos::new(x, y))).unwrap();
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
                    sender.send(ImgPresMsg::MouseMove (Pos::new(x, y))).unwrap();
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
                img_pres_rect.consume_msg(msg, f);
            }

            img_pres_rect.scale_draw(&mut drawable, f);
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
    im_sz: Size,
    scale_factor: f32,
    prev_pos: Option<Pos>
}

impl ImgPresRect {
    fn new(img: &Img) -> Self {
        ImgPresRect { 
            im_pos: Pos::new(0, 0), 
            scale_factor: 1_f32,
            im_sz: Size::new(img.w() as i32, img.h() as i32),
            prev_pos: None
        }
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

					// correct position
					let img_bottom_right: Pos = self.im_sz.into();
					let frame_pos = Pos::new(frame.x(), frame.y());
					let frame_size = Pos::new(frame.w(), frame.h());
					
					let pos_min = frame_pos + frame_size - img_bottom_right;
					let pos_max = frame_pos;
			
					self.im_pos.clamp(pos_min, pos_max);
			
					println!("{} <= {} <= {}", pos_min.x, self.im_pos.x, pos_max.x);
					println!("{} <= {} <= {}", pos_min.y, self.im_pos.y, pos_max.y);
                }
            },
            ImgPresMsg::MouseUp => {
                self.prev_pos = None;
            },
            ImgPresMsg::MouseLeave => {
                self.prev_pos = None;
            },
            ImgPresMsg::MouseScroll { delta } => {
                self.scale_factor += delta;

				// let min_w = frame

				let im_w = (self.scale_factor * self.im_sz.w as f32) as i32;
				let im_h = (self.scale_factor * self.im_sz.h as f32) as i32;
				self.im_sz = Size::new(im_w, im_h);
            },
        }
    }

    fn scale_draw(&mut self, img: &mut fltk::image::RgbImage, f: &frame::Frame) {
        img.scale(self.im_sz.w, self.im_sz.h, true, true);

        use fltk::draw;
        draw::push_clip(f.x(), f.y(), f.w(), f.h());
        img.draw(self.im_pos.x, self.im_pos.y, self.im_sz.w, self.im_sz.h);
        draw::pop_clip();
    }
}


#[derive(Clone, Copy, Debug)]
struct Pos { x: i32, y: i32 }

impl Pos {
	fn new(x: i32, y: i32) -> Self {
		Self { x, y }
	}

	fn clamp(&mut self, min: Pos, max: Pos) {
		assert!(min.x <= max.x);
		assert!(min.y <= max.y);

		self.x = if self.x < min.x { min.x } else if self.x > max.x { max.x } else { self.x };
		self.y = if self.y < min.y { min.y } else if self.y > max.y { max.y } else { self.y };
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
    MouseScroll { delta: f32 },
}

